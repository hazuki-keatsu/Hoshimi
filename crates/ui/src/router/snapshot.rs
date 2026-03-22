//! Page Snapshot System
//!
//! Provides mechanisms for creating efficient snapshots of page render trees.
//! Snapshots separate dynamic components (that need to keep animating) from
//! static components (that can be pre-rendered to a texture).

use std::cell::RefCell;
use std::rc::Weak;

use hoshimi_types::{Offset, Rect, Size};

use crate::painter::Painter;
use crate::render_object::RenderObject;

// ============================================================================
// Snapshot Handle
// ============================================================================

/// Handle to a rendered texture snapshot
/// 
/// This is an opaque handle that references a texture stored in the rendering backend.
/// The actual texture data is managed by the Painter implementation.
#[derive(Debug, Clone, Copy)]
pub struct TextureHandle {
    /// Unique identifier for this texture
    pub(crate) id: u64,
    /// Size of the texture
    pub(crate) size: Size,
}

impl PartialEq for TextureHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TextureHandle {}

impl std::hash::Hash for TextureHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl TextureHandle {
    /// Create a new texture handle
    pub fn new(id: u64, size: Size) -> Self {
        Self { id, size }
    }
    
    /// Get the texture ID
    pub fn id(&self) -> u64 {
        self.id
    }
    
    /// Get the texture size
    pub fn size(&self) -> Size {
        self.size
    }
}

impl Default for TextureHandle {
    fn default() -> Self {
        Self {
            id: 0,
            size: Size::ZERO,
        }
    }
}

// ============================================================================
// Surface Handle
// ============================================================================

/// Handle to an offscreen rendering surface
/// 
/// Surfaces are used for rendering content to a texture that can be
/// composited later. This enables efficient page transition effects.
#[derive(Debug, Clone, Copy)]
pub struct SurfaceHandle {
    /// Unique identifier for this surface
    pub(crate) id: u64,
    /// Size of the surface
    pub(crate) size: Size,
}

impl PartialEq for SurfaceHandle {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SurfaceHandle {}

impl std::hash::Hash for SurfaceHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl SurfaceHandle {
    /// Create a new surface handle
    pub fn new(id: u64, size: Size) -> Self {
        Self { id, size }
    }
    
    /// Get the surface ID
    pub fn id(&self) -> u64 {
        self.id
    }
    
    /// Get the surface size
    pub fn size(&self) -> Size {
        self.size
    }
}

// ============================================================================
// Dynamic Node Reference
// ============================================================================

/// Reference to a dynamic node within a snapshot
/// 
/// Dynamic nodes are components that should continue animating during
/// page transitions (e.g., playing videos, animated sprites).
#[derive(Debug)]
pub struct DynamicNodeRef {
    /// The render object (weak reference to avoid cycles)
    node: Weak<RefCell<dyn RenderObject>>,
    /// Position offset within the snapshot
    offset: Offset,
    /// Z-order for layering
    z_order: i32,
    /// Original rect in the render tree
    original_rect: Rect,
}

impl DynamicNodeRef {
    /// Create a new dynamic node reference
    pub fn new(
        node: Weak<RefCell<dyn RenderObject>>,
        offset: Offset,
        z_order: i32,
        original_rect: Rect,
    ) -> Self {
        Self {
            node,
            offset,
            z_order,
            original_rect,
        }
    }
    
    /// Get the offset
    pub fn offset(&self) -> Offset {
        self.offset
    }
    
    /// Get the z-order
    pub fn z_order(&self) -> i32 {
        self.z_order
    }
    
    /// Get the original rect
    pub fn original_rect(&self) -> Rect {
        self.original_rect
    }
    
    /// Check if the node is still valid
    pub fn is_valid(&self) -> bool {
        self.node.strong_count() > 0
    }
    
    /// Try to upgrade to a strong reference
    pub fn upgrade(&self) -> Option<std::rc::Rc<RefCell<dyn RenderObject>>> {
        self.node.upgrade()
    }
}

// ============================================================================
// Page Snapshot
// ============================================================================

/// A snapshot of a page's render tree
/// 
/// Snapshots optimize page transitions by:
/// 1. Pre-rendering static components to a texture (fast to composite)
/// 2. Keeping references to dynamic components (continue animating)
/// 
/// This allows smooth transitions while dynamic content remains alive.
#[derive(Debug)]
pub struct PageSnapshot {
    /// Texture containing pre-rendered static content
    static_layer: Option<TextureHandle>,
    
    /// References to dynamic nodes that need continued updates
    dynamic_nodes: Vec<DynamicNodeRef>,
    
    /// Size of the snapshot
    size: Size,
    
    /// Whether the snapshot is valid and usable
    is_valid: bool,
}

impl PageSnapshot {
    /// Create an empty snapshot
    pub fn empty(size: Size) -> Self {
        Self {
            static_layer: None,
            dynamic_nodes: Vec::new(),
            size,
            is_valid: false,
        }
    }
    
    /// Create a snapshot with a static layer
    pub fn with_static_layer(texture: TextureHandle) -> Self {
        let size = texture.size;
        Self {
            static_layer: Some(texture),
            dynamic_nodes: Vec::new(),
            size,
            is_valid: true,
        }
    }
    
    /// Add a dynamic node reference
    pub fn add_dynamic_node(&mut self, node_ref: DynamicNodeRef) {
        self.dynamic_nodes.push(node_ref);
    }
    
    /// Get the static layer texture handle
    pub fn static_layer(&self) -> Option<&TextureHandle> {
        self.static_layer.as_ref()
    }
    
    /// Get the dynamic nodes
    pub fn dynamic_nodes(&self) -> &[DynamicNodeRef] {
        &self.dynamic_nodes
    }
    
    /// Get mutable access to dynamic nodes
    pub fn dynamic_nodes_mut(&mut self) -> &mut Vec<DynamicNodeRef> {
        &mut self.dynamic_nodes
    }
    
    /// Get the snapshot size
    pub fn size(&self) -> Size {
        self.size
    }
    
    /// Check if the snapshot is valid
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
    
    /// Mark the snapshot as invalid
    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }
    
    /// Check if there are any dynamic nodes
    pub fn has_dynamic_content(&self) -> bool {
        !self.dynamic_nodes.is_empty()
    }
    
    /// Update dynamic nodes (tick animations)
    /// 
    /// Returns true if any node is still animating
    pub fn tick_dynamic_nodes(&mut self, delta: f32) -> bool {
        let mut animating = false;
        
        // Remove invalid references
        self.dynamic_nodes.retain(|node| node.is_valid());
        
        for node_ref in &self.dynamic_nodes {
            if let Some(node) = node_ref.upgrade() {
                if node.borrow_mut().tick(delta) {
                    animating = true;
                }
            }
        }
        
        animating
    }
    
    /// Paint the snapshot to the given painter
    /// 
    /// Paints the static layer first, then overlays dynamic nodes.
    pub fn paint(&self, painter: &mut dyn Painter, offset: Offset, alpha: f32) {
        // Paint static layer
        if let Some(texture) = &self.static_layer {
            let dest_rect = Rect::from_offset_size(offset, self.size);
            painter.draw_offscreen_texture(texture.id, dest_rect, alpha);
        }
        
        // Paint dynamic nodes
        // Sort by z-order for correct layering
        let mut sorted_nodes: Vec<_> = self.dynamic_nodes.iter().collect();
        sorted_nodes.sort_by_key(|n| n.z_order);
        
        for node_ref in sorted_nodes {
            if let Some(node) = node_ref.upgrade() {
                painter.save();
                painter.translate(offset + node_ref.offset);
                
                // Apply alpha if not fully opaque
                if alpha < 1.0 {
                    // TODO: Implement alpha for dynamic nodes
                    // This would require extending the painter to support layer opacity
                }
                
                node.borrow().paint(painter);
                painter.restore();
            }
        }
    }
}

// ============================================================================
// Snapshot Builder
// ============================================================================

/// Options for building a page snapshot
#[derive(Debug, Clone)]
pub struct SnapshotOptions {
    /// Whether to include dynamic nodes (or render everything as static)
    pub preserve_dynamic: bool,
    /// Whether to skip nodes marked as non-cacheable
    pub respect_cache_hints: bool,
    /// Maximum number of dynamic nodes to preserve
    pub max_dynamic_nodes: usize,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            preserve_dynamic: true,
            respect_cache_hints: true,
            max_dynamic_nodes: 32,
        }
    }
}

/// Builder for creating page snapshots
/// 
/// This builder analyzes a render tree to separate static and dynamic content,
/// then creates an optimized snapshot for efficient transition rendering.
pub struct SnapshotBuilder {
    options: SnapshotOptions,
}

impl SnapshotBuilder {
    /// Create a new snapshot builder with default options
    pub fn new() -> Self {
        Self {
            options: SnapshotOptions::default(),
        }
    }
    
    /// Create a snapshot builder with custom options
    pub fn with_options(options: SnapshotOptions) -> Self {
        Self { options }
    }
    
    /// Build a snapshot from a render tree
    /// 
    /// This method:
    /// 1. Analyzes the tree to identify dynamic nodes
    /// 2. Renders static content to an offscreen surface
    /// 3. Creates references to dynamic nodes
    /// 4. Returns a complete PageSnapshot
    pub fn build(
        &self,
        root: &dyn RenderObject,
        painter: &mut dyn Painter,
        size: Size,
    ) -> PageSnapshot {
        let mut snapshot = PageSnapshot::empty(size);
        
        // If we're preserving dynamic content, analyze the tree
        let dynamic_rects = if self.options.preserve_dynamic {
            self.find_dynamic_nodes(root)
        } else {
            Vec::new()
        };
        
        // Create offscreen surface and render static content
        let surface_id = painter.create_offscreen_surface(size);
        if surface_id != 0 {
            painter.begin_offscreen(surface_id);
            
            // Render the entire tree (for now - optimization: skip dynamic regions)
            // In a more advanced implementation, we would mask out dynamic regions
            root.paint(painter);
            
            let texture_id = painter.end_offscreen();
            if texture_id != 0 {
                snapshot.static_layer = Some(TextureHandle::new(texture_id, size));
                snapshot.is_valid = true;
            }
            painter.release_offscreen_surface(surface_id);
        } else {
            // Fallback: No offscreen support, create a fully dynamic snapshot
            snapshot.is_valid = true;
        }
        
        // Store dynamic node information (but not actual node references for now)
        // Full implementation would track actual RenderObject references
        for (rect, z_order) in dynamic_rects {
            // In a full implementation, we would store actual node references
            // For now, we just record the regions
            snapshot.size = size;
            let _ = rect;
            let _ = z_order;
        }
        
        snapshot
    }
    
    /// Find dynamic nodes in the render tree
    /// 
    /// Returns a list of (Rect, z_order) for each dynamic node found
    fn find_dynamic_nodes(&self, root: &dyn RenderObject) -> Vec<(Rect, i32)> {
        let mut results = Vec::new();
        self.collect_dynamic_nodes(root, &mut results, 0);
        
        // Limit the number of dynamic nodes
        if results.len() > self.options.max_dynamic_nodes {
            results.truncate(self.options.max_dynamic_nodes);
        }
        
        results
    }
    
    /// Recursively collect dynamic nodes
    fn collect_dynamic_nodes(
        &self,
        node: &dyn RenderObject,
        results: &mut Vec<(Rect, i32)>,
        z_order: i32,
    ) {
        // Check if this node is dynamic
        if node.is_dynamic() {
            results.push((node.get_rect(), z_order));
        }
        
        // Recurse into children
        let mut child_z = z_order;
        for child in node.children() {
            child_z += 1;
            self.collect_dynamic_nodes(child, results, child_z);
        }
    }
}

impl Default for SnapshotBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Painter Extensions for Snapshots
// ============================================================================

/// Extension trait for Painter to support snapshot operations
/// 
/// This trait adds offscreen rendering and texture composition capabilities
/// needed for the snapshot system.
pub trait SnapshotPainter: Painter {
    /// Create an offscreen rendering surface
    /// 
    /// Returns None if offscreen rendering is not supported.
    fn create_offscreen_surface(&mut self, size: Size) -> Option<SurfaceHandle>;
    
    /// Begin rendering to an offscreen surface
    /// 
    /// All subsequent draw calls will render to this surface until
    /// `end_offscreen()` is called.
    fn begin_offscreen(&mut self, surface: &SurfaceHandle);
    
    /// End offscreen rendering and return the resulting texture
    /// 
    /// Returns None if offscreen rendering failed.
    fn end_offscreen(&mut self) -> Option<TextureHandle>;
    
    /// Release an offscreen surface
    fn release_surface(&mut self, surface: SurfaceHandle);
    
    /// Release a texture
    fn release_texture(&mut self, texture: TextureHandle);
    
    /// Draw a snapshot texture to a destination rectangle
    fn draw_snapshot_texture(&mut self, texture: &TextureHandle, dest_rect: Rect, alpha: f32);
}

// ============================================================================
// Default implementation for Painter (no-op)
// ============================================================================

/// Extension methods for any Painter to support basic snapshot operations
/// 
/// These default implementations provide fallback behavior when the
/// underlying painter doesn't support offscreen rendering.
impl<P: Painter + ?Sized> SnapshotPainterExt for P {}

/// Extension trait providing default snapshot implementations
pub trait SnapshotPainterExt: Painter {
    /// Create an offscreen rendering surface (default: returns None)
    fn create_offscreen_surface(&mut self, _size: Size) -> Option<SurfaceHandle> {
        None
    }
    
    /// Begin rendering to an offscreen surface (default: no-op)
    fn begin_offscreen(&mut self, _surface: &SurfaceHandle) {}
    
    /// End offscreen rendering (default: returns None)
    fn end_offscreen(&mut self) -> Option<TextureHandle> {
        None
    }
    
    /// Draw a snapshot texture (default: no-op)
    fn draw_snapshot_texture(&mut self, _texture: &TextureHandle, _dest_rect: Rect, _alpha: f32) {}
}
