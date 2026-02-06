//! Widget Key for list diffing
//!
//! Widget keys are used to identify widgets across rebuilds,
//! enabling efficient incremental updates.

use std::any::TypeId;
use std::hash::{Hash, Hasher};

/// A key that uniquely identifies a widget instance
/// 
/// Keys are used by the diff algorithm to match widgets across rebuilds.
/// Without keys, widgets are matched by their index in the parent's child list.
/// With keys, widgets can be matched even if their order changes.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum WidgetKey {
    /// A string-based key
    String(String),
    
    /// An integer-based key (useful for indexed lists)
    Int(i64),
    
    /// A value-based key (for any hashable value)
    Value(u64),
    
    /// A local key unique within the parent widget
    Local(u64),
    
    /// A global key unique across the entire tree
    Global(String),
}

impl WidgetKey {
    /// Create a string key
    pub fn string(s: impl Into<String>) -> Self {
        WidgetKey::String(s.into())
    }
    
    /// Create an integer key
    pub fn int(i: i64) -> Self {
        WidgetKey::Int(i)
    }
    
    /// Create a key from any hashable value
    pub fn from_value<T: Hash>(value: &T) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        WidgetKey::Value(hasher.finish())
    }
    
    /// Create a local key (unique within parent)
    pub fn local(id: u64) -> Self {
        WidgetKey::Local(id)
    }
    
    /// Create a global key (unique across entire tree)
    pub fn global(id: impl Into<String>) -> Self {
        WidgetKey::Global(id.into())
    }
}

impl From<&str> for WidgetKey {
    fn from(s: &str) -> Self {
        WidgetKey::String(s.to_string())
    }
}

impl From<String> for WidgetKey {
    fn from(s: String) -> Self {
        WidgetKey::String(s)
    }
}

impl From<i64> for WidgetKey {
    fn from(i: i64) -> Self {
        WidgetKey::Int(i)
    }
}

impl From<i32> for WidgetKey {
    fn from(i: i32) -> Self {
        WidgetKey::Int(i as i64)
    }
}

impl From<usize> for WidgetKey {
    fn from(i: usize) -> Self {
        WidgetKey::Int(i as i64)
    }
}

/// Widget identity combining type and optional key
#[derive(Debug, Clone)]
pub struct WidgetIdentity {
    /// The TypeId of the widget
    pub type_id: TypeId,
    
    /// Optional key for disambiguation
    pub key: Option<WidgetKey>,
}

impl WidgetIdentity {
    /// Create a new WidgetIdentity
    pub fn new(type_id: TypeId, key: Option<WidgetKey>) -> Self {
        Self { type_id, key }
    }
    
    /// Check if two identities match
    pub fn matches(&self, other: &WidgetIdentity) -> bool {
        if self.type_id != other.type_id {
            return false;
        }
        
        match (&self.key, &other.key) {
            (Some(k1), Some(k2)) => k1 == k2,
            (None, None) => true,
            _ => false,
        }
    }
}

impl PartialEq for WidgetIdentity {
    fn eq(&self, other: &Self) -> bool {
        self.matches(other)
    }
}

impl Eq for WidgetIdentity {}

impl Hash for WidgetIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
        self.key.hash(state);
    }
}
