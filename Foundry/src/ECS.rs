//Imports
use std::any::Any;
use std::any::TypeId;
use std::collections::HashMap;
use std::hash::Hash;

use ash::vk::PipelineLayout;

//Type Aliases
type EntityID = u64;
type ComponentID = u64;
type ArchetypeID = usize;
type ArchetypeSet = Vec<ArchetypeID>;
type ArchetypeSignature = Vec<TypeId>;

//Structs
struct EntityRecord {
    row_index: usize,
    archetype_id: ArchetypeID,
}

struct EntityBuilder {
    id: EntityID,
    pending_components: Vec<TypeId>,
}

impl EntityBuilder {
    fn with<T: Component + 'static>(mut self, component: T) -> Self {
        let id = TypeId::of::<T>();
        self.pending_components.push(id);
        self
    }

    fn build(self, world: &mut World) -> EntityID {
        //Find archetype / create if non existing
        //
        let signature = Archetype::generate_signature(&self.pending_components);
        match world.archetype_index.get(&signature) {
            Some(archetype) => {}
            None => {
                world.build_archetype(&signature);
            }
        }
        let Some(archetype) = world.archetype_index.get(&signature) else {
            panic!("Failed to generate or get archetype");
        };
        //archetype.add_entity WIP

        //Populate Archetype

        //Make an entity record (last?)

        println!("Building");
        self.id
    }
}

//------Components
trait Component {}

struct GameObject {
    name: String,
    tag: String,
}

struct MeshRenderer {}

struct Health {
    current: u32,
    max: u32,
}
struct Position {}

impl Component for Position {}
impl Component for Health {}

//---------------

//Columns will map a TypeId to a box containing the vector of associated components
struct Archetype {
    id: ArchetypeID,
    columns: HashMap<TypeId, Box<dyn Any>>,
    entity_ids: Vec<EntityID>,
}

impl Archetype {
    fn generate_signature(ids: &Vec<TypeId>) -> Vec<TypeId> {
        /* Old code that used Vec<Box<dyn Any>> for components
                let mut ids = Vec::new();
                for component in components {
                    let id = component.as_ref().type_id();
                    ids.push(id);
                }
        */
        let mut ret: Vec<TypeId> = ids.to_owned();
        ret.sort();
        ret
    }
    //Originally had signature instead of 'components'
    //Removed because it creates a mental dependancy for the Archetype API
    //Might bite me in the butt but who knows
    fn new(id: ArchetypeID, components: Vec<TypeId>, registry: &TypeRegister) -> Self {
        //
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

    //TypeId gets lost in Box<dyn Any> so bundle it in a tuple
    fn add_entity(&mut self, components: Vec<(TypeId, Box<dyn Any>)>) -> usize {
        for (id, comp) in components {}
        self.entity_ids.len() as usize
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

    archetypes: Vec<Archetype>, //Outdated?
    archetype_index: HashMap<ArchetypeSignature, Archetype>,

    //Used to find the components of any entity instantly
    entity_index: HashMap<EntityID, EntityRecord>,

    //Used to find the archetypes that contain a component (used for systems)
    component_index: HashMap<ComponentID, ArchetypeSet>,

    registry: TypeRegister,
}

impl World {
    pub fn debug_world_data(&self) {
        for archetype in &self.archetypes {}
    }

    pub fn spawn(&mut self) -> EntityBuilder {
        let new_entity = EntityBuilder {
            id: self.next_available_entity_id,
            pending_components: Vec::new(),
        };
        //Eventually change this to an ID pool?
        self.next_available_entity_id += 1;

        new_entity
    }

    //Example:
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

    fn get_component<T: 'static>(self, entity: EntityID) {
        let Some(record) = self.entity_index.get(&entity) else {
            panic!("No record for entity!");
        };
        let archetype = &self.archetypes[record.archetype_id];
    }
}
