use std::{cell::RefCell, collections::HashMap};

// Components can be stored in vecs or hashmaps
// All components should have an bool "sparse"
pub trait ComponentType {
    const sparse: bool;
}

pub trait ComponentStorage {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
    fn push_none(&mut self);
}

impl<T: 'static> ComponentStorage for RefCell<Vec<Option<T>>> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
    fn push_none(&mut self) {
        self.get_mut().push(None)
    }
}

impl<T: 'static> ComponentStorage for HashMap<usize, T> {
    fn as_any(&self) -> &dyn std::any::Any {
        self as &dyn std::any::Any
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self as &mut dyn std::any::Any
    }
    fn push_none(&mut self) {
        // does nothing; defeats the point of using a Hashmap
    }
}
