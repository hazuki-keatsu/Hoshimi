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
//! For lists with keys, the algorithm uses LIS (Longest Increasing Subsequence)
//! to minimize move operations. This is the same approach used by React and Flutter.
//!
//! # Performance
//!
//! - Time complexity: O(n log n) for LIS computation
//! - Space complexity: O(n) for key maps and position arrays

use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use crate::key::WidgetKey;
use crate::widget::Widget;

/// Result of diffing two widget trees
#[derive(Debug, Clone)]
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
    
    /// Replace the widget at the given index (atomic remove + insert)
    /// This is more efficient than separate Remove + Insert operations
    Replace {
        /// Index to replace at
        index: usize,
        /// Widget with new configuration
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
    
    /// Optimize operations by merging consecutive Remove + Insert into Replace
    pub fn optimize(&mut self) {
        if self.operations.len() < 2 {
            return;
        }
        
        let mut optimized = Vec::with_capacity(self.operations.len());
        let mut i = 0;
        
        while i < self.operations.len() {
            match (&self.operations[i], self.operations.get(i + 1)) {
                (
                    DiffOperation::Remove { index: r_idx },
                    Some(DiffOperation::Insert { index: i_idx, widget })
                ) if r_idx == i_idx => {
                    optimized.push(DiffOperation::Replace {
                        index: *r_idx,
                        widget: *widget,
                    });
                    i += 2;
                }
                _ => {
                    optimized.push(self.operations[i].clone());
                    i += 1;
                }
            }
        }
        
        self.operations = optimized;
        
        // Recursively optimize child diffs
        for (_, child_diff) in &mut self.child_diffs {
            child_diff.optimize();
        }
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
        
        let (mut child_ops, child_diffs) = Self::diff_children(&old_children, &new_children);
        
        // Build result
        let mut operations = Vec::new();
        
        if needs_update {
            operations.push(DiffOperation::Update {
                index: 0,
                widget: new,
            });
        }
        
        // Merge child operations
        operations.append(&mut child_ops);
        
        let mut result = DiffResult {
            operations,
            child_diffs,
        };
        
        // Optimize operations (merge Remove+Insert into Replace)
        result.optimize();
        
        Some(result)
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
                    child_diffs.push((i, diff));
                }
            } else {
                // Types differ - use Replace instead of Remove + Insert
                operations.push(DiffOperation::Replace {
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
    
    /// Diff children with keys using LIS-based algorithm
    ///
    /// This algorithm minimizes move operations by:
    /// 1. Matching children by key
    /// 2. Computing the Longest Increasing Subsequence of matched positions
    /// 3. Only moving children not in the LIS
    fn diff_keyed_children<'a>(
        old_children: &[&dyn Widget],
        new_children: &[&'a dyn Widget],
    ) -> (Vec<DiffOperation<'a>>, Vec<(usize, DiffResult<'a>)>) {
        let mut operations = Vec::new();
        let mut child_diffs = Vec::new();
        
        // Build key -> old_index map
        let old_key_map: HashMap<WidgetKey, usize> = old_children
            .iter()
            .enumerate()
            .filter_map(|(i, w)| w.key().map(|k| (k, i)))
            .collect();
        
        // Track mapping: new_index -> old_index (for matched children)
        let mut new_to_old: Vec<Option<usize>> = vec![None; new_children.len()];
        
        // Track which old children are matched
        let mut matched_old: HashSet<usize> = HashSet::new();
        
        // First pass: match by key and type
        for (new_idx, new_child) in new_children.iter().enumerate() {
            if let Some(key) = new_child.key() {
                if let Some(&old_idx) = old_key_map.get(&key) {
                    let old_child = old_children[old_idx];
                    
                    // Check if types match
                    if old_child.widget_type() == new_child.widget_type() {
                        new_to_old[new_idx] = Some(old_idx);
                        matched_old.insert(old_idx);
                        
                        // Check if update needed
                        if let Some(diff) = Self::diff_widget(old_child, *new_child) {
                            if diff.has_changes() {
                                child_diffs.push((new_idx, diff));
                            }
                        }
                    }
                }
            }
        }
        
        // Build array of old indices for matched new children (in new order)
        let matched_old_indices: Vec<usize> = new_to_old
            .iter()
            .filter_map(|&old_idx| old_idx)
            .collect();
        
        // Compute LIS on matched positions
        let lis_indices = Self::compute_lis(&matched_old_indices);
        
        // Convert LIS indices to a set of old indices that should NOT be moved
        let lis_old_indices: HashSet<usize> = lis_indices
            .iter()
            .map(|&lis_idx| matched_old_indices[lis_idx])
            .collect();
        
        // Generate operations
        
        // 1. Remove unmatched old children (in reverse order to maintain indices)
        let mut unmatched_old: Vec<usize> = (0..old_children.len())
            .filter(|i| !matched_old.contains(i))
            .collect();
        unmatched_old.sort_by(|a, b| b.cmp(a));
        
        for old_idx in unmatched_old {
            operations.push(DiffOperation::Remove { index: old_idx });
        }
        
        // 2. Process new children in order
        for (new_idx, new_child) in new_children.iter().enumerate() {
            if let Some(old_idx) = new_to_old[new_idx] {
                // This child exists in old tree
                if lis_old_indices.contains(&old_idx) {
                    // Part of LIS - no move needed, already in correct relative position
                } else {
                    // Not in LIS - needs to be moved
                    operations.push(DiffOperation::Move {
                        from: old_idx,
                        to: new_idx,
                        widget: *new_child,
                    });
                }
            } else {
                // New child - insert
                operations.push(DiffOperation::Insert {
                    index: new_idx,
                    widget: *new_child,
                });
            }
        }
        
        (operations, child_diffs)
    }
    
    /// Compute the Longest Increasing Subsequence (LIS)
    ///
    /// Returns the indices of elements that form the LIS.
    /// Uses O(n log n) algorithm with binary search.
    ///
    /// # Example
    ///
    /// ```
    /// // Input: [3, 1, 2, 4]
    /// // One LIS is [1, 2, 4] at indices [1, 2, 3]
    /// // Returns: [1, 2, 3]
    /// ```
    fn compute_lis(arr: &[usize]) -> Vec<usize> {
        if arr.is_empty() {
            return Vec::new();
        }
        
        let n = arr.len();
        
        // tails[i] = smallest tail element for LIS of length i+1
        let mut tails: Vec<usize> = Vec::with_capacity(n);
        
        // tails_idx[i] = index in arr where tails[i] came from
        let mut tails_idx: Vec<usize> = Vec::with_capacity(n);
        
        // prev[i] = index of previous element in LIS ending at arr[i]
        let mut prev: Vec<Option<usize>> = vec![None; n];
        
        for i in 0..n {
            let val = arr[i];
            
            // Binary search for the first tail >= val
            let pos = tails.partition_point(|&x| x < val);
            
            if pos == tails.len() {
                // Extend the LIS
                tails.push(val);
                tails_idx.push(i);
            } else {
                // Update existing tail
                tails[pos] = val;
                tails_idx[pos] = i;
            }
            
            // Set previous pointer
            if pos > 0 {
                prev[i] = Some(tails_idx[pos - 1]);
            }
        }
        
        // Reconstruct LIS indices
        let lis_len = tails.len();
        let mut result = Vec::with_capacity(lis_len);
        
        // Start from the last element of LIS
        let mut current = tails_idx.last().copied();
        
        // Build result in reverse order
        let mut temp = Vec::with_capacity(lis_len);
        while let Some(idx) = current {
            temp.push(idx);
            current = prev[idx];
        }
        
        // Reverse to get correct order
        result.extend(temp.into_iter().rev());
        
        result
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_lis_empty() {
        assert_eq!(WidgetDiffer::compute_lis(&[]), Vec::<usize>::new());
    }
    
    #[test]
    fn test_lis_single() {
        assert_eq!(WidgetDiffer::compute_lis(&[5]), vec![0]);
    }
    
    #[test]
    fn test_lis_increasing() {
        // [1, 2, 3, 4] - entire array is LIS
        assert_eq!(WidgetDiffer::compute_lis(&[1, 2, 3, 4]), vec![0, 1, 2, 3]);
    }
    
    #[test]
    fn test_lis_decreasing() {
        // [4, 3, 2, 1] - any single element is LIS
        let result = WidgetDiffer::compute_lis(&[4, 3, 2, 1]);
        assert_eq!(result.len(), 1);
        assert!(result[0] < 4);
    }
    
    #[test]
    fn test_lis_mixed() {
        // [3, 1, 2, 4] - LIS is [1, 2, 4] at indices [1, 2, 3]
        let result = WidgetDiffer::compute_lis(&[3, 1, 2, 4]);
        assert_eq!(result.len(), 3);
        
        // Verify the result forms a valid increasing subsequence
        let arr = [3, 1, 2, 4];
        for i in 1..result.len() {
            assert!(arr[result[i - 1]] < arr[result[i]]);
        }
    }
    
    #[test]
    fn test_lis_complex() {
        // [10, 9, 2, 5, 3, 7, 101, 18]
        // LIS is [2, 3, 7, 18] or [2, 5, 7, 101] etc.
        let arr = [10, 9, 2, 5, 3, 7, 101, 18];
        let result = WidgetDiffer::compute_lis(&arr);
        
        // Length should be 4
        assert_eq!(result.len(), 4);
        
        // Verify it's a valid increasing subsequence
        for i in 1..result.len() {
            assert!(arr[result[i - 1]] < arr[result[i]]);
        }
    }
    
    #[test]
    fn test_lis_with_duplicates() {
        // [2, 2, 2] - strictly increasing means only one element
        let result = WidgetDiffer::compute_lis(&[2, 2, 2]);
        assert_eq!(result.len(), 1);
    }
    
    #[test]
    fn test_diff_result_optimize() {
        // Test that Remove + Insert at same index becomes Replace
        use std::any::Any;
        
        #[derive(Debug, Clone)]
        struct TestWidget;
        
        impl Widget for TestWidget {
            fn create_render_object(&self) -> Box<dyn crate::render_object::RenderObject> {
                unimplemented!()
            }
            fn update_render_object(&self, _render_object: &mut dyn crate::render_object::RenderObject) {}
            fn as_any(&self) -> &dyn Any { self }
            fn clone_boxed(&self) -> Box<dyn Widget> { Box::new(self.clone()) }
        }
        
        let widget = TestWidget;
        
        let mut result = DiffResult {
            operations: vec![
                DiffOperation::Remove { index: 0 },
                DiffOperation::Insert { index: 0, widget: &widget },
            ],
            child_diffs: Vec::new(),
        };
        
        result.optimize();
        
        assert_eq!(result.operations.len(), 1);
        assert!(matches!(result.operations[0], DiffOperation::Replace { index: 0, .. }));
    }
}
