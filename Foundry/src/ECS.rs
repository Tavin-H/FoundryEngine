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

    fn build(self) -> EntityID {
        //Find archetype / create if non existing
        //Populate Archetype
        //Make an entity record (last?)
        println!("Building");
        self.id
    }
}

//------Components
trait Component {}
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
    fn new(id: ArchetypeID, signature: Vec<TypeId>, registry: &TypeRegister) -> Self {
        let mut columns = HashMap::new();
        for component in signature {
            if let Some(column_creator) = registry.column_creators.get(&component) {
                let empty_column = column_creator();
                columns.insert(component, empty_column);
            };
        }
        Archetype {
            id: id,
            columns: columns,
            entity_ids: Vec::new(),
        }
    }
}

//World (Core logic)
struct World {
    //Used to create a new entity
    next_available_entity_id: EntityID,
    next_available_archtype_id: ArchetypeID,

    archetypes: Vec<Archetype>,

    //Used to find the components of any entity instantly
    entity_index: HashMap<EntityID, EntityRecord>,

    //Used to find the archetypes that contain a component (used for systems)
    component_index: HashMap<ComponentID, ArchetypeSet>,

    registry: TypeRegister,
}

type ColumnCreator = fn() -> Box<dyn Any>;

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

impl World {
    fn spawn(&mut self) -> EntityBuilder {
        let new_entity = EntityBuilder {
            id: self.next_available_entity_id,
            pending_components: Vec::new(),
        };
        //Eventually change this to an ID pool?
        self.next_available_entity_id += 1;

        new_entity
    }

    //Example:
    fn spawn_player(mut self) {
        let player = self
            .spawn()
            .with::<Position>(Position {})
            .with::<Health>(Health {
                current: 20,
                max: 20,
            })
            .build();
    }

    fn locate_archetype(components: Vec<Box<dyn Any>>) {
        for component in components {}
    }

    fn ensure_registered<T: 'static>(&mut self) {
        let id: TypeId = TypeId::of::<T>();
        if (!self.registry.column_creators.contains_key(&id)) {
            self.registry.register::<T>();
        }
    }

    fn generate_archetype_signature(components: Vec<Box<dyn Any>>) -> Vec<TypeId> {
        let mut ids = Vec::new();
        for component in components {
            let id = component.as_ref().type_id();
            ids.push(id);
        }
        ids.sort();
        ids
    }

    fn get_component<T: 'static>(self, entity: EntityID) {
        let Some(record) = self.entity_index.get(&entity) else {
            panic!("No record for entity!");
        };
        let archetype = &self.archetypes[record.archetype_id];
    }
}
