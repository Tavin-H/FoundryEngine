//Imports
use std::any::Any;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

//Built-in Components
use crate::components::{Component, MeshAllocation, Transform};

use ash::vk::PipelineLayout;

//Type Aliases
type EntityID = u64;
type ComponentID = u64;
type ArchetypeID = usize;
type ArchetypeSet = Vec<ArchetypeID>;
type ArchetypeSignature = Vec<TypeId>;

//Structs
pub struct EntityRecord {
    row_index: usize,
    archetype_signature: ArchetypeSignature,
}

pub struct EntityBuilder {
    id: EntityID,
    signature: Vec<TypeId>,
    //Box because Size of dyn Fn is not known at compile time
    push_component_functions: Vec<Box<dyn FnOnce(&mut Archetype)>>,
    ensure_functions: Vec<Box<dyn Fn(&mut World)>>,
}

impl EntityBuilder {
    pub fn with<T: Component + 'static>(mut self, component: T) -> Self {
        let id = TypeId::of::<T>();

        let push_function = |archetype: &mut Archetype| {
            let id = TypeId::of::<T>();
            let column = archetype
                .columns
                .get_mut(&id)
                .expect("No matching column in archetype");
            let downcast_column = column
                .downcast_mut::<Vec<T>>()
                .expect("Failed to downcast in push");
            downcast_column.push(component);
        };
        let ensure_function = |world: &mut World| {
            world.ensure_registered::<T>();
        };
        self.ensure_functions.push(Box::new(ensure_function));
        self.push_component_functions.push(Box::new(push_function));
        self.signature.push(id);
        self
    }

    pub fn build(self, world: &mut World) -> EntityID {
        //Find archetype / create if non existing
        //Populate Archetype
        //Make an entity record (last?)

        //Ensure Initialized
        for ensure in &self.ensure_functions {
            ensure(world);
        }
        let signature = Archetype::generate_signature(&self.signature);
        match world.archetype_index.get(&signature) {
            Some(archetype) => {}
            None => {
                world.build_archetype(&signature);
            }
        }
        let Some(archetype) = world.archetype_index.get_mut(&signature) else {
            panic!("Failed to generate or get archetype");
        };
        let row_index = archetype.add_entity(self.id, self.push_component_functions);

        let record = EntityRecord {
            archetype_signature: signature,
            row_index: row_index,
        };
        world.entity_index.insert(self.id, record);

        println!("Building");
        self.id
    }
}

//------Components

struct GameObject {
    name: String,
    tag: String,
}

struct MeshRenderer {}

pub struct Health {
    pub current: u32,
    pub max: u32,
}
impl Health {
    pub fn new(current: u32, max: u32) -> Self {
        Health { current, max }
    }
}

struct Position {}

impl Component for Position {}
impl Component for Health {}

//---------------

//Columns will map a TypeId to a box containing the vector of associated components
pub struct Archetype {
    id: ArchetypeID,
    columns: HashMap<TypeId, Box<dyn Any>>,
    entity_ids: Vec<EntityID>,
}

impl Archetype {
    fn has_component<T: 'static>(&self) -> bool {
        let id = TypeId::of::<T>();
        self.columns.contains_key(&id)
    }
    fn has_id(&self, id: &TypeId) -> bool {
        self.columns.contains_key(id)
    }
    fn get_components_as_slice<T: 'static>(&self) -> &[T] {
        let id = TypeId::of::<T>();

        let raw_column = self
            .columns
            .get(&id)
            .expect("Filed to find column in get_components_as_slice");
        let downcast_column = raw_column
            .downcast_ref::<Vec<T>>()
            .expect("Failed to downcast column in get_components_as_slice");
        downcast_column.as_slice()
    }

    fn generate_signature(ids: &Vec<TypeId>) -> Vec<TypeId> {
        let mut ret: Vec<TypeId> = ids.to_owned();
        ret.sort();
        ret
    }
    //Originally had signature instead of 'components'
    //Removed because it creates a mental dependancy for the Archetype API
    //Might bite me in the butt but who knows
    fn new(id: ArchetypeID, components: Vec<TypeId>, registry: &TypeRegister) -> Self {
        //Use the reference before moving each TypeID into columns
        let signature = Archetype::generate_signature(&components);
        let mut columns = HashMap::new();
        for comp in components {
            if let Some(column_creator) = registry.column_creators.get(&comp) {
                let empty_column = column_creator();
                columns.insert(comp, empty_column);
            };
        }
        Archetype {
            id: id,
            columns: columns,
            entity_ids: Vec::new(),
        }
    }

    fn add_entity(
        &mut self,
        id: EntityID,
        push_components: Vec<Box<dyn FnOnce(&mut Archetype)>>,
    ) -> usize {
        for push_fn in push_components {
            push_fn(self);
        }
        let row_index = self.entity_ids.len() as usize;
        self.entity_ids.push(id as u64);
        row_index
    }
}

type ColumnCreator = fn() -> Box<dyn Any>;

#[derive(Default)]
struct TypeRegister {
    column_creators: HashMap<TypeId, ColumnCreator>,
}
impl TypeRegister {
    fn register<T: 'static>(&mut self) {
        let id = TypeId::of::<T>();
        self.column_creators.insert(id, || {
            let v: Vec<T> = Vec::new();
            Box::new(v)
        });
    }
}

//World (Core logic)
#[derive(Default)]
pub struct World {
    //Used to create a new entity
    next_available_entity_id: EntityID,
    next_available_archtype_id: ArchetypeID,

    pub archetype_index: HashMap<ArchetypeSignature, Archetype>,

    //Used to find the components of any entity instantly
    pub entity_index: HashMap<EntityID, EntityRecord>,

    //Used to find the archetypes that contain a component (used for systems)
    component_index: HashMap<ComponentID, ArchetypeSet>,

    registry: TypeRegister,
}

impl World {
    pub fn new() -> World {
        World {
            ..Default::default()
        }
    }

    //FIXME - THIS WOULD BE GOOD TO MAKE
    pub fn debug_world_data(&self) {
        for i in 0..self.next_available_entity_id {
            println!();
        }
    }

    pub fn spawn(&mut self) -> EntityBuilder {
        let new_entity = EntityBuilder {
            id: self.next_available_entity_id,
            //pending_components: HashMap::new(),
            ensure_functions: Vec::new(),
            push_component_functions: Vec::new(),
            signature: Vec::new(),
        };
        //FIXME Eventually change this to an ID pool?
        self.next_available_entity_id += 1;

        new_entity
    }

    //Example usage of builder pattern:
    pub fn spawn_player(&mut self) {
        let player = self
            .spawn()
            .with::<Position>(Position {})
            .with::<Health>(Health {
                current: 20,
                max: 20,
            })
            .build(self);
    }

    fn ensure_registered<T: 'static>(&mut self) {
        let id: TypeId = TypeId::of::<T>();
        if (!self.registry.column_creators.contains_key(&id)) {
            self.registry.register::<T>();
        }
    }

    fn build_archetype(&mut self, signature: &ArchetypeSignature) {
        match self.archetype_index.get(signature) {
            Some(archetype) => (),
            None => {
                let new_archetype = Archetype::new(
                    self.next_available_archtype_id,
                    signature.clone(),
                    &self.registry,
                );
                self.archetype_index
                    .insert(signature.clone(), new_archetype);

                //ID pool this later
                self.next_available_archtype_id += 1;
            }
        }
    }

    pub fn get_component<T: 'static>(&mut self, entity: EntityID) -> &T {
        let component_id = TypeId::of::<T>();
        let Some(record) = self.entity_index.get(&entity) else {
            panic!("No record for entity!");
        };
        let archetype = self
            .archetype_index
            .get_mut(&record.archetype_signature)
            .expect("Failed to get archetpye");
        let raw_vec = archetype
            .columns
            .get_mut(&component_id)
            .expect("Failed to get column");

        let downcast_vec = raw_vec
            .downcast_mut::<Vec<T>>()
            .expect("Failed to downcast column in get_component");
        &downcast_vec[record.row_index]
    }

    pub fn get_archetypes_by_ids(&self, ids: &Vec<TypeId>) -> Vec<&Archetype> {
        //Gets all archetypes with component T
        self.archetype_index
            .values() // Iterate over the archetypes
            .filter(|archetype| {
                // Check if EVERY search_id exists in this archetype's columns
                // .all() stops early (short-circuits) if it finds a missing ID
                ids.iter().all(|id| archetype.has_id(id))
            })
            .collect()
    }
    pub fn get_render_batches(&mut self) -> Vec<(&[Transform], &[MeshAllocation])> {
        //Get all the components of type MeshAllocation and Transform
        //Uses get_archetypes and sees overlap
        //Passes to vulkan.draw_frame(List<(transform, meshallocation))
        let mut render_batches: Vec<(&[Transform], &[MeshAllocation])> = Vec::new();
        let mut archetypes = self.get_archetypes_by_ids(&vec![
            TypeId::of::<Transform>(),
            TypeId::of::<MeshAllocation>(),
        ]);
        println!("RENDER ARCHETYPES FOUND {}", archetypes.len());

        for archetype in archetypes.iter_mut() {
            //Get components
            //Call vulkan to record stuff
            let transforms = archetype.get_components_as_slice::<Transform>();
            let mesh_allocation = archetype.get_components_as_slice::<MeshAllocation>();
            render_batches.push((transforms, mesh_allocation));
        }
        render_batches
        //Use render_batches in draw_frame()
    }
}
