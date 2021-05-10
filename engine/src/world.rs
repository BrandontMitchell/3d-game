use crate::components::*;
use std::{
    cell::{RefCell, RefMut},
    collections::HashMap,
};

// Implementation based on:
// https://ianjk.com/ecs-in-rust/

pub struct World {
    num_entities: usize,
    components: Vec<Box<dyn ComponentStorage>>, // VECS ONLY
    components_sparse: Vec<Box<dyn ComponentStorage>>, // HASHMAPS ONLY
}

impl World {
    pub fn new() -> Self {
        Self {
            num_entities: 0,
            components: Vec::new(),
            components_sparse: Vec::new(),
        }
    }

    // add an entity with no components
    pub fn add_entity(&mut self) -> usize {
        let entity_id = self.num_entities;
        self.num_entities += 1;

        // add new space in every component vec
        for component_vec in self.components.iter_mut() {
            component_vec.push_none();
        }
        entity_id
    }

    // adds a single component to an entity
    pub fn add_component<ComponentType: 'static + Component>(
        &mut self,
        id: usize,
        c: ComponentType,
    ) {
        match c.is_sparse() {
            true => {
                for component_map in self.components_sparse.iter_mut() {
                    if let Some(component_map) = component_map
                        .as_any_mut()
                        .downcast_mut::<RefCell<HashMap<usize, ComponentType>>>()
                    {
                        component_map.get_mut().insert(id, c);
                        return;
                    }
                }

                // if component map doesn't exist, create it
                let mut new_component_map: HashMap<usize, ComponentType> = HashMap::new();
                new_component_map.insert(id, c);
                self.components_sparse
                    .push(Box::new(RefCell::new(new_component_map)));
            }
            false => {
                for component_vec in self.components.iter_mut() {
                    if let Some(component_vec) = component_vec
                        .as_any_mut()
                        .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
                    {
                        component_vec.get_mut()[id] = Some(c);
                        return;
                    }
                }

                // if component vec doesn't exist, create it
                // have to create spots for all existing entities
                let mut new_component_vec: Vec<Option<ComponentType>> =
                    Vec::with_capacity(self.num_entities);
                for _ in 0..self.num_entities {
                    new_component_vec.push(None);
                }
                new_component_vec[id] = Some(c);
                self.components
                    .push(Box::new(RefCell::new(new_component_vec)));
            }
        }
    }

    // remove an entity from the world
    pub fn remove_entity<ComponentType: 'static>(&mut self, id: usize) {
        for component_vec in self.components.iter_mut() {
            if let Some(component_vec) = component_vec
                .as_any_mut()
                .downcast_mut::<RefCell<Vec<Option<ComponentType>>>>()
            {
                component_vec.get_mut()[id] = None;
            }
        }
        for component_map in self.components_sparse.iter_mut() {
            if let Some(component_map) = component_map
                .as_any_mut()
                .downcast_mut::<RefCell<HashMap<usize, ComponentType>>>()
            {
                component_map.get_mut().remove(&id);
            }
        }
        self.num_entities -= 1;
    }

    // get a component vec
    pub fn borrow_components_mut<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<Vec<Option<ComponentType>>>> {
        for component_vec in self.components.iter() {
            if let Some(component_vec) = component_vec
                .as_any()
                .downcast_ref::<RefCell<Vec<Option<ComponentType>>>>()
            {
                return Some(component_vec.borrow_mut());
            }
        }
        None
    }

    // get a component map
    pub fn borrow_components_sparse_mut<ComponentType: 'static>(
        &self,
    ) -> Option<RefMut<HashMap<usize, ComponentType>>> {
        for component_map in self.components_sparse.iter() {
            if let Some(component_map) = component_map
                .as_any()
                .downcast_ref::<RefCell<HashMap<usize, ComponentType>>>()
            {
                return Some(component_map.borrow_mut());
            }
        }
        None
    }

    // remove all entities
    pub fn clear(&mut self) {
        self.components.clear();
        self.components_sparse.clear();
        self.num_entities = 0;
    }
}
