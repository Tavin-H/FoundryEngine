use num::{Float, PrimInt};
use uuid::Uuid;

//----Logic outline----
// 1. Serialize is implemented for types either by default or through a macro
// 2. Serialize trait implements a method that returns a SerializeNode
// 3. An AST is built from values that can be used for various other systems such as the inspector
// 4. Serializer takes the root node and implements methods to parse it into YAML, JSON, whatever
//
// Serializer pipeline:
// - Serializer is given structs to serialize (objects and components) from the scene graph in the
// editor
// - It calls the appropriate serialize method for the object / component
// - This builds a Serializer tree
// - Serializer tree then gets parsed and turned into a yaml file for the scene

// How to turn Nodes into text in a file
pub trait Serializer {
    // How to parse the AST
    fn serialize_int(&mut self, val: i64);
    fn serialize_float(&mut self, val: f64);
    fn serialize_string(&mut self, val: String);
    fn serialize_struct(&mut self);
}

pub struct YamlSerializer {
    ast_roots: Vec<SerializerNode>,
    output_buffer: String, //Later change to a data structure that represents fscn files as an AST?
}
impl YamlSerializer {
    fn new() -> Self {
        YamlSerializer {
            ast_roots: Vec::new(),
            output_buffer: String::new(),
        }
    }
}
impl Serializer for YamlSerializer {
    fn serialize_int(&mut self, val: i64) {}
    fn serialize_float(&mut self, val: f64) {}
    fn serialize_string(&mut self, val: String) {}
    fn serialize_struct(&mut self) {}
}

pub trait Serialize {
    fn serialize(&self, serializer: &mut impl Serializer) -> SerializerNode;
}

pub trait DeSerialize {}
// Serializer AST
pub enum SerializerNode {
    Entity {
        serialized_name: String,
        id: Uuid,
        parent: Uuid,
        components: Vec<SerializerNode>,
    },
    Component(),
    Struct(),
    Float(f64),
    Integer(i64),
    String(String),
    Bool(bool),
}

macro_rules! impl_serialize {
    ($base_type: ty, int) => {
        impl Serialize for $base_type {
            fn serialize(&self, serializer: &mut impl Serializer) -> SerializerNode {
                SerializerNode::Integer(*self as i64)
            }
        }
    };
    ($base_type: ty, float) => {
        impl Serialize for $base_type {
            fn serialize(&self, serializer: &mut impl Serializer) -> SerializerNode {
                SerializerNode::Float(*self as f64)
            }
        }
    };
    ($base_type: ty, string) => {
        impl Serialize for $base_type {
            fn serialize(&self, serializer: &mut impl Serializer) -> SerializerNode {
                SerializerNode::String(self.clone() as String)
            }
        }
    };
    ($base_type: ty, bool) => {
        impl Serialize for $base_type {
            fn serialize(&self, serializer: &mut impl Serializer) -> SerializerNode {
                SerializerNode::Bool(self.clone() as bool)
            }
        }
    };
}
// Integers
impl_serialize!(i8, int);
impl_serialize!(i16, int);
impl_serialize!(i32, int);
impl_serialize!(i128, int);

// Floats
impl_serialize!(f32, float);
impl_serialize!(f64, float);

// Others
impl_serialize!(String, string);
impl_serialize!(bool, bool);

pub fn test() {
    let mut yaml_serializer = YamlSerializer::new();
    //let test: SerializerNode = 4.0.serialize(&mut yaml_serializer);
}
