//! Reconciler - Applies diff operations to the RenderObject tree
//!
//! The reconciler takes the diff result from the diff algorithm and
//! applies the necessary changes to the RenderObject tree, maintaining
//! proper lifecycle hooks (mount/unmount/update).
//!
//! # Operation Order
//!
//! Operations are applied in a specific order to maintain consistency:
//! 1. Unmount and remove nodes scheduled for deletion
//! 2. Replace nodes (atomic unmount + mount)
//! 3. Move nodes to new positions
//! 4. Create and mount new nodes
//! 5. Update existing nodes with new configuration
//! 6. Recursively reconcile children

use tracing::{debug, trace};

use crate::render_object::RenderObject;
use crate::tree::diff::{DiffOperation, DiffResult};
use crate::widget::Widget;

/// Reconciler for applying diff results to the RenderObject tree
pub struct Reconciler;

impl Reconciler {
    /// Apply a diff result to update a RenderObject
    ///
    /// This method applies all operations from the diff result to transform
    /// the old RenderObject tree into the new configuration.
    pub fn reconcile(
        render_object: &mut dyn RenderObject,
        widget: &dyn Widget,
        diff: &DiffResult<'_>,
    ) {
        trace!("Reconciling render object: {:?}", std::any::type_name_of_val(render_object));
        
        // Process operations in the correct order
        Self::apply_operations(render_object, widget, &diff.operations);
        
        // Recursively reconcile children
        Self::reconcile_children(render_object, &diff.child_diffs);
    }
    
    /// Apply a list of diff operations
    fn apply_operations(
        render_object: &mut dyn RenderObject,
        widget: &dyn Widget,
        operations: &[DiffOperation<'_>],
    ) {
        // Sort operations by type for proper ordering:
        // 1. First collect removes (will be processed in reverse order)
        // 2. Then replaces (atomic unmount + mount)
        // 3. Then moves
        // 4. Then inserts
        // 5. Finally updates
        
        let mut removes: Vec<usize> = Vec::new();
        let mut replaces: Vec<(usize, &dyn Widget)> = Vec::new();
        let mut inserts: Vec<(usize, &dyn Widget)> = Vec::new();
        let mut moves: Vec<(usize, usize, &dyn Widget)> = Vec::new();
        let mut updates: Vec<(usize, &dyn Widget)> = Vec::new();
        
        for op in operations {
            match op {
                DiffOperation::Remove { index } => {
                    removes.push(*index);
                }
                DiffOperation::Replace { index, widget } => {
                    replaces.push((*index, *widget));
                }
                DiffOperation::Insert { index, widget } => {
                    inserts.push((*index, *widget));
                }
                DiffOperation::Move { from, to, widget } => {
                    moves.push((*from, *to, *widget));
                }
                DiffOperation::Update { index, widget } => {
                    updates.push((*index, *widget));
                }
                DiffOperation::None { .. } => {
                    // No action needed
                }
            }
        }
        
        // Apply removes first (in reverse order to maintain indices)
        removes.sort_by(|a, b| b.cmp(a));
        for index in removes {
            Self::remove_child(render_object, index);
        }
        
        // Apply replaces (atomic unmount + mount)
        // Sort by index in reverse order to maintain correct indices during replacement
        replaces.sort_by(|a, b| b.0.cmp(&a.0));
        for (index, widget) in replaces {
            Self::replace_child(render_object, index, widget);
        }
        
        // Apply moves (complex - may need temporary storage)
        Self::apply_moves(render_object, &moves);
        
        // Apply inserts (in forward order)
        inserts.sort_by_key(|(idx, _)| *idx);
        for (index, widget) in inserts {
            Self::insert_child(render_object, index, widget);
        }
        
        // Apply updates
        for (index, widget_ref) in updates {
            if index == 0 {
                // Update the root render object itself
                Self::update_render_object(render_object, widget);
            } else {
                // Update a child (index is 1-based relative to operations context)
                let mut children = render_object.children_mut();
                if let Some(child) = children.get_mut(index - 1) {
                    Self::update_render_object(*child, widget_ref);
                }
            }
        }
    }
    
    /// Remove a child at the given index
    fn remove_child(render_object: &mut dyn RenderObject, index: usize) {
        debug!("Removing child at index {}", index);
        
        // First call unmount on the child
        {
            let mut children = render_object.children_mut();
            if let Some(child) = children.get_mut(index) {
                Self::unmount_recursive(*child);
            }
        }
        
        // Then remove from parent
        if let Some(removed) = render_object.remove_child(index) {
            drop(removed);
        }
    }
    
    /// Replace a child at the given index (atomic remove + insert)
    fn replace_child(render_object: &mut dyn RenderObject, index: usize, widget: &dyn Widget) {
        debug!("Replacing child at index {}", index);
        
        // First unmount the old child
        {
            let mut children = render_object.children_mut();
            if let Some(child) = children.get_mut(index) {
                Self::unmount_recursive(*child);
            }
        }
        
        // Create new RenderObject from widget
        let mut new_child = widget.create_render_object();
        
        // Mount the new child
        Self::mount_recursive(new_child.as_mut());
        
        // Replace in parent (this is more efficient than remove + insert)
        render_object.replace_child(index, new_child);
    }
    
    /// Insert a new child at the given index
    fn insert_child(render_object: &mut dyn RenderObject, index: usize, widget: &dyn Widget) {
        debug!("Inserting child at index {}", index);
        
        // Create new RenderObject from widget
        let mut new_child = widget.create_render_object();
        
        // Mount the new child
        Self::mount_recursive(new_child.as_mut());
        
        // Insert into parent
        render_object.insert_child(index, new_child);
    }
    
    /// Apply move operations
    fn apply_moves(
        render_object: &mut dyn RenderObject,
        moves: &[(usize, usize, &dyn Widget)],
    ) {
        if moves.is_empty() {
            return;
        }
        
        debug!("Applying {} move operations", moves.len());
        
        // For simplicity, we use a two-phase approach:
        // 1. Remove all moving children (in reverse order)
        // 2. Re-insert them at their new positions (in forward order)
        
        let mut sorted_moves = moves.to_vec();
        
        // Phase 1: Remove (reverse order by 'from' index)
        sorted_moves.sort_by(|a, b| b.0.cmp(&a.0));
        
        let mut removed: Vec<(usize, Box<dyn RenderObject>)> = Vec::new();
        for (from, to, _widget) in &sorted_moves {
            if let Some(child) = render_object.remove_child(*from) {
                removed.push((*to, child));
            }
        }
        
        // Phase 2: Insert (forward order by 'to' index)
        removed.sort_by_key(|(to, _)| *to);
        for (to, child) in removed {
            render_object.insert_child(to, child);
        }
    }
    
    /// Update a RenderObject with new widget configuration
    fn update_render_object(render_object: &mut dyn RenderObject, widget: &dyn Widget) {
        trace!("Updating render object with widget: {:?}", std::any::type_name_of_val(widget));
        
        // Apply widget configuration
        widget.update_render_object(render_object);
        
        // Call update hook
        render_object.on_update();
        
        // Mark for relayout/repaint
        render_object.mark_needs_layout();
        render_object.mark_needs_paint();
    }
    
    /// Recursively reconcile child diffs
    fn reconcile_children(
        render_object: &mut dyn RenderObject,
        child_diffs: &[(usize, DiffResult<'_>)],
    ) {
        if child_diffs.is_empty() {
            return;
        }
        
        for (index, diff) in child_diffs {
            let mut children = render_object.children_mut();
            if let Some(child) = children.get_mut(*index) {
                // Apply operations for this child
                for op in &diff.operations {
                    match op {
                        DiffOperation::Update { index: op_idx, widget } => {
                            if *op_idx == 0 {
                                // index 0 means update the child itself
                                Self::update_render_object(*child, *widget);
                            } else {
                                // Update a nested child (1-based index)
                                let mut nested_children = child.children_mut();
                                if let Some(nested_child) = nested_children.get_mut(*op_idx - 1) {
                                    Self::update_render_object(*nested_child, *widget);
                                }
                            }
                        }
                        DiffOperation::Remove { index: child_idx } => {
                            Self::remove_child(*child, *child_idx);
                        }
                        DiffOperation::Insert { index: child_idx, widget } => {
                            Self::insert_child(*child, *child_idx, *widget);
                        }
                        DiffOperation::Replace { index: child_idx, widget } => {
                            Self::replace_child(*child, *child_idx, *widget);
                        }
                        DiffOperation::Move { from, to, widget } => {
                            Self::apply_moves(*child, &[(*from, *to, *widget)]);
                        }
                        DiffOperation::None { .. } => {}
                    }
                }
                
                // Recursively handle nested child_diffs
                Self::reconcile_children(*child, &diff.child_diffs);
            }
        }
    }
    
    /// Mount a RenderObject and all its children
    pub fn mount_recursive(render_object: &mut dyn RenderObject) {
        trace!("Mounting: {:?}", std::any::type_name_of_val(render_object));
        
        // Mount self
        render_object.on_mount();
        
        // Mount children
        for child in render_object.children_mut() {
            Self::mount_recursive(child);
        }
    }
    
    /// Unmount a RenderObject and all its children
    pub fn unmount_recursive(render_object: &mut dyn RenderObject) {
        trace!("Unmounting: {:?}", std::any::type_name_of_val(render_object));
        
        // Unmount children first (reverse order)
        let children: Vec<_> = render_object.children_mut();
        for child in children.into_iter().rev() {
            Self::unmount_recursive(child);
        }
        
        // Unmount self
        render_object.on_unmount();
    }
    
    /// Build a RenderObject tree from a Widget tree
    ///
    /// This is used for initial construction (no diffing needed).
    /// 
    /// Note: The widget's `create_render_object` method is responsible for
    /// recursively creating child RenderObjects. This method just handles
    /// mounting the tree.
    pub fn build_tree(widget: &dyn Widget) -> Box<dyn RenderObject> {
        debug!("Building render tree from widget: {:?}", std::any::type_name_of_val(widget));
        
        // Create the RenderObject tree
        // Note: create_render_object recursively creates child RenderObjects,
        // so we don't need to manually iterate over widget.children() here.
        let mut render_object = widget.create_render_object();
        
        // Mount the entire tree
        Self::mount_recursive(render_object.as_mut());
        
        render_object
    }
    
    /// Replace an entire subtree
    ///
    /// Used when widget types don't match and we can't update in place.
    pub fn replace_subtree(
        old_render_object: &mut dyn RenderObject,
        new_widget: &dyn Widget,
    ) -> Box<dyn RenderObject> {
        debug!("Replacing subtree");
        
        // Unmount old tree
        Self::unmount_recursive(old_render_object);
        
        // Build new tree
        Self::build_tree(new_widget)
    }
}

/// Result of a reconciliation operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconcileResult {
    /// No changes were made
    Unchanged,
    /// Existing nodes were updated
    Updated,
    /// The tree structure changed (nodes added/removed/moved)
    StructureChanged,
    /// The entire subtree was replaced
    Replaced,
}

impl ReconcileResult {
    /// Check if layout is needed after this reconciliation
    pub fn needs_layout(&self) -> bool {
        matches!(self, ReconcileResult::Updated | ReconcileResult::StructureChanged | ReconcileResult::Replaced)
    }
    
    /// Check if paint is needed after this reconciliation
    pub fn needs_paint(&self) -> bool {
        *self != ReconcileResult::Unchanged
    }
}
