struct World {
    num_entities: usize,
    components: Vec<Box<dyn ComponentStorage>>,     // VECS ONLY
    components_sparse: Vec<Box<dyn ComponentStorage>>,      // HASHMAPS ONLY
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
    pub fn add_entity (&mut self) -> usize {
        let entity_id = self.num_entities;
        self.num_entities += 1;

        // add new space in every component vec
        for component_vec in self.components.iter_mut() {
            component_vec.push_none();
        }

    }

    // adds a single component to an entity
    pub fn add_component<ComponentType: 'static> (&mut self, c: ComponentType, id: usize) {
        match c.sparse {
            true => {
                // unsure if this works for hashmap
                for component_map in self.components_sparse.iter_mut() {
                    if let Some(component_map) = component_map
                        .as_any_mut()
                        .downcast_mut::<Vec<Option<ComponentType>>>()
                    {
                        component_map.get_mut(id).unwrap() = c;
                        break;
                    }
                }

                // if component storage doesn't exist, create it
                let mut new_hashmap: Hashmap<usize, ComponentType> = Hashmap::new();
                new_hashmap.insert(entity_id, c);
                self.components_sparse.push(Box::new(new_hashmap));
            },
            false => {
                for component_vec in self.components.iter_mut() {
                    if let Some(component_vec) = component_vec
                        .as_any_mut()
                        .downcast_mut::<Vec<Option<ComponentType>>>()
                    {
                        component_vec[id] = Some(component);
                        break;
                    }
                }

                // if component vec doesn't exist, create it
                // have to create spots for all existing entities
                let mut new_component_vec: Vec<Option<ComponentType>> = Vec::with_capacity(self.num_entities);
                for _ in 0..self.entities_count {
                    new_component_vec.push(None);
                }
                new_component_vec[entity_id] = Some(c);
                self.components.push(Box::new(new_component_vec));
            },
        }
    }

    // remove an entity from the world
    pub fn remove_entity(&mut self, id: usize) {
        for component_vec in self.components.iter_mut() {
            component_vec[id] = None;
        }
        for component_map in self.components_sparse.iter_mut() {
            component_map.remove(id);
        }
    }

    // remove all entities
    pub fn clear(&mut self) {
        self.components.clear();
        self.components_sparse.clear();
        self.num_entities = 0;
    }
}