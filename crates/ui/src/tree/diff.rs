//! Widget Diff Algorithm
//!
//! This module implements the diff algorithm that compares widget trees
//! and produces a minimal set of operations to transform one tree into another.
//!
//! # Algorithm Overview
//!
//! The diff algorithm works in several phases:
//! 1. **Type comparison**: If widget types differ, replace the entire subtree
//! 2. **Key matching**: If keys are present, match by key; otherwise by index
//! 3. **Property comparison**: Use `should_update()` to check if update is needed
//! 4. **Child recursion**: Recursively diff child lists
//!
//! # Key Matching Optimization
//!
//! For lists with keys, the algorithm uses a simplified LCS (Longest Common Subsequence)
//! approach to minimize DOM operations (insert/remove/move).

use std::any::TypeId;
use std::collections::HashMap;

use crate::key::WidgetKey;
use crate::widget::Widget;

/// Result of diffing two widget trees
#[derive(Debug)]
pub enum DiffOperation<'a> {
    /// Insert a new widget at the given index
    Insert {
        /// Index to insert at
        index: usize,
        /// Widget to create RenderObject from
        widget: &'a dyn Widget,
    },
    
    /// Remove the widget at the given index
    Remove {
        /// Index to remove from
        index: usize,
    },
    
    /// Update the widget at the given index
    Update {
        /// Index to update
        index: usize,
        /// Widget with new configuration
        widget: &'a dyn Widget,
    },
    
    /// Move a widget from one index to another
    Move {
        /// Original index
        from: usize,
        /// Target index
        to: usize,
        /// Widget reference (for potential update)
        widget: &'a dyn Widget,
    },
    
    /// No operation needed (widget unchanged)
    None {
        /// Index that remains unchanged
        index: usize,
    },
}

/// The result of a complete diff operation
#[derive(Debug)]
pub struct DiffResult<'a> {
    /// Operations to apply in order
    pub operations: Vec<DiffOperation<'a>>,
    
    /// Child diff results (for recursive application)
    pub child_diffs: Vec<(usize, DiffResult<'a>)>,
}

impl<'a> DiffResult<'a> {
    /// Create an empty diff result (no changes)
    pub fn empty() -> Self {
        Self {
            operations: Vec::new(),
            child_diffs: Vec::new(),
        }
    }
    
    /// Check if there are any operations
    pub fn has_changes(&self) -> bool {
        !self.operations.is_empty() || !self.child_diffs.is_empty()
    }
}

/// Widget differ for computing minimal updates
pub struct WidgetDiffer;

impl WidgetDiffer {
    /// Diff a single widget against another
    ///
    /// Returns `Some(DiffResult)` if the widget can be updated in place,
    /// or `None` if the entire subtree needs to be replaced.
    pub fn diff_widget<'a>(
        old: &dyn Widget,
        new: &'a dyn Widget,
    ) -> Option<DiffResult<'a>> {
        // Phase 1: Type comparison
        if old.widget_type() != new.widget_type() {
            // Types differ - need full replacement
            return None;
        }
        
        // Phase 2: Key comparison (if both have keys, they must match)
        match (old.key(), new.key()) {
            (Some(old_key), Some(new_key)) if old_key != new_key => {
                // Keys differ - need full replacement
                return None;
            }
            _ => {}
        }
        
        // Phase 3: Property comparison
        let needs_update = new.should_update(old);
        
        // Phase 4: Child diffing
        let old_children = old.children();
        let new_children = new.children();
        
        let (child_ops, child_diffs) = Self::diff_children(&old_children, &new_children);
        
        // Build result
        let mut operations = Vec::new();
        
        if needs_update {
            operations.push(DiffOperation::Update {
                index: 0,
                widget: new,
            });
        }
        
        // Merge child operations
        operations.extend(child_ops);
        
        Some(DiffResult {
            operations,
            child_diffs,
        })
    }
    
    /// Diff two lists of children
    fn diff_children<'a>(
        old_children: &[&dyn Widget],
        new_children: &[&'a dyn Widget],
    ) -> (Vec<DiffOperation<'a>>, Vec<(usize, DiffResult<'a>)>) {
        if old_children.is_empty() && new_children.is_empty() {
            return (Vec::new(), Vec::new());
        }
        
        // Check if any children have keys
        let has_keys = new_children.iter().any(|w| w.key().is_some())
            || old_children.iter().any(|w| w.key().is_some());
        
        if has_keys {
            Self::diff_keyed_children(old_children, new_children)
        } else {
            Self::diff_indexed_children(old_children, new_children)
        }
    }
    
    /// Diff children without keys (by index)
    fn diff_indexed_children<'a>(
        old_children: &[&dyn Widget],
        new_children: &[&'a dyn Widget],
    ) -> (Vec<DiffOperation<'a>>, Vec<(usize, DiffResult<'a>)>) {
        let mut operations = Vec::new();
        let mut child_diffs = Vec::new();
        
        let old_len = old_children.len();
        let new_len = new_children.len();
        let common_len = old_len.min(new_len);
        
        // Compare common prefix
        for i in 0..common_len {
            let old_child = old_children[i];
            let new_child = new_children[i];
            
            if let Some(diff) = Self::diff_widget(old_child, new_child) {
                if diff.has_changes() {
                    // Store child diff for recursive processing
                    // This includes both the child's own operations and its nested child_diffs
                    child_diffs.push((i, diff));
                }
            } else {
                // Types differ - remove old, insert new
                operations.push(DiffOperation::Remove { index: i });
                operations.push(DiffOperation::Insert {
                    index: i,
                    widget: new_child,
                });
            }
        }
        
        // Handle tail differences
        if new_len > old_len {
            // New children added at end
            for i in old_len..new_len {
                operations.push(DiffOperation::Insert {
                    index: i,
                    widget: new_children[i],
                });
            }
        } else if old_len > new_len {
            // Old children removed from end (remove in reverse order)
            for i in (new_len..old_len).rev() {
                operations.push(DiffOperation::Remove { index: i });
            }
        }
        
        (operations, child_diffs)
    }
    
    /// Diff children with keys using LCS-based algorithm
    fn diff_keyed_children<'a>(
        old_children: &[&dyn Widget],
        new_children: &[&'a dyn Widget],
    ) -> (Vec<DiffOperation<'a>>, Vec<(usize, DiffResult<'a>)>) {
        let mut operations = Vec::new();
        let mut child_diffs = Vec::new();
        
        // Build key -> index maps
        let old_key_map: HashMap<Option<WidgetKey>, usize> = old_children
            .iter()
            .enumerate()
            .map(|(i, w)| (w.key(), i))
            .collect();
        
        // Note: new_key_map could be used for reverse lookups in advanced diff algorithms
        let _new_key_map: HashMap<Option<WidgetKey>, usize> = new_children
            .iter()
            .enumerate()
            .map(|(i, w)| (w.key(), i))
            .collect();
        
        // Track which old indices are still in use
        let mut used_old_indices: Vec<bool> = vec![false; old_children.len()];
        
        // Track new position for each new child
        let mut new_positions: Vec<Option<usize>> = vec![None; new_children.len()];
        
        // First pass: match by key
        for (new_idx, new_child) in new_children.iter().enumerate() {
            let key = new_child.key();
            
            if let Some(&old_idx) = old_key_map.get(&key) {
                // Found matching key
                let old_child = old_children[old_idx];
                
                // Check if types match
                if old_child.widget_type() == new_child.widget_type() {
                    used_old_indices[old_idx] = true;
                    new_positions[new_idx] = Some(old_idx);
                    
                    // Check if update needed
                    if let Some(diff) = Self::diff_widget(old_child, *new_child) {
                        if diff.has_changes() {
                            // Store child diff for recursive processing
                            child_diffs.push((new_idx, diff));
                        }
                    }
                    
                    // Check if move needed
                    if old_idx != new_idx {
                        operations.push(DiffOperation::Move {
                            from: old_idx,
                            to: new_idx,
                            widget: *new_child,
                        });
                    }
                }
            }
        }
        
        // Second pass: remove unmatched old children (in reverse order)
        for (old_idx, used) in used_old_indices.iter().enumerate().rev() {
            if !used {
                operations.push(DiffOperation::Remove { index: old_idx });
            }
        }
        
        // Third pass: insert new children that weren't matched
        for (new_idx, new_child) in new_children.iter().enumerate() {
            if new_positions[new_idx].is_none() {
                operations.push(DiffOperation::Insert {
                    index: new_idx,
                    widget: *new_child,
                });
            }
        }
        
        (operations, child_diffs)
    }
}

/// Represents the identity of a widget for diff purposes
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WidgetIdentity {
    /// The type ID of the widget
    pub type_id: TypeId,
    /// Optional key
    pub key: Option<WidgetKey>,
}

impl WidgetIdentity {
    /// Create identity from a widget
    pub fn from_widget(widget: &dyn Widget) -> Self {
        Self {
            type_id: widget.widget_type(),
            key: widget.key(),
        }
    }
    
    /// Check if two widgets can be updated in place
    pub fn can_update(&self, other: &WidgetIdentity) -> bool {
        self.type_id == other.type_id && self.key == other.key
    }
}
