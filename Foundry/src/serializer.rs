use foundry_derive::Serialize;
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
    fn serialize_struct(&mut self, node: &SerializerNode);
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
    fn write(&self, write_path: &'static str) {
        let raw_path = format!("scenes/{write_path}.yaml");
        let full_path = std::path::Path::new(&raw_path);
        println!("writing: {}", self.output_buffer);
        std::fs::write(&full_path, &self.output_buffer);
    }
}
impl Serializer for YamlSerializer {
    fn serialize_int(&mut self, val: i64) {}
    fn serialize_float(&mut self, val: f64) {}
    fn serialize_string(&mut self, val: String) {}
    fn serialize_struct(&mut self, node: &SerializerNode) {
        let SerializerNode::Struct { f_name, data } = node else {
            panic!("");
        };
        //self.output_buffer += "\n";
        self.output_buffer.push_str(&format!(
            "
Component: 
    id: 1
    type: {f_name}
    data:
"
        ));
        for (field_name, node) in data {
            let value = match node {
                SerializerNode::Float(name, val) => format!("{:.4}", val),
                SerializerNode::Integer(name, val) => val.to_string(),
                SerializerNode::String(name, val) => val.to_string(),
                SerializerNode::Bool(name, val) => val.to_string(),
                _ => panic!("Node not supported as data field"),
            };
            self.output_buffer
                .push_str(&format!("        {}, {}\n", field_name, value));
        }
        println!("Serializing struct {}", self.output_buffer.len())
    }
}

pub trait Serialize {
    fn serialize(&self, name: &'static str, serializer: &mut impl Serializer) -> SerializerNode;
}

pub trait DeSerialize {}
// Serializer AST
#[derive(Debug)]
pub enum SerializerNode {
    // Documents
    Object {
        foundry_type_id: u64,
        local_file_id: Uuid,
        name: String,
        tags: Vec<String>,
        components: Vec<uuid::Uuid>,
    },
    Component {
        foundry_type_id: u64,
        local_file_id: Uuid,
        name: String,                     // Mainly for manual scene file editing
        data_fields: Vec<SerializerNode>, // A Struct node
    },

    // Custom data types
    Struct {
        f_name: &'static str,
        data: Vec<(&'static str, SerializerNode)>,
    },

    // Primatives
    Float(&'static str, f64),
    Integer(&'static str, i64),
    String(&'static str, String),
    Bool(&'static str, bool),
}

// Serialize Macros
macro_rules! impl_serialize {
    ($base_type: ty, int) => {
        impl Serialize for $base_type {
            fn serialize(
                &self,
                name: &'static str,
                serializer: &mut impl Serializer,
            ) -> SerializerNode {
                SerializerNode::Integer(name, *self as i64)
            }
        }
    };
    ($base_type: ty, float) => {
        impl Serialize for $base_type {
            fn serialize(
                &self,
                name: &'static str,
                serializer: &mut impl Serializer,
            ) -> SerializerNode {
                SerializerNode::Float(name, *self as f64)
            }
        }
    };
    ($base_type: ty, string) => {
        impl Serialize for $base_type {
            fn serialize(
                &self,
                name: &'static str,
                serializer: &mut impl Serializer,
            ) -> SerializerNode {
                SerializerNode::String(name, self.clone() as String)
            }
        }
    };
    ($base_type: ty, bool) => {
        impl Serialize for $base_type {
            fn serialize(
                &self,
                name: &'static str,
                serializer: &mut impl Serializer,
            ) -> SerializerNode {
                SerializerNode::Bool(name, self.clone() as bool)
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

#[derive(Serialize)]
pub struct TestStruct {
    thing: f32,
    test: i32,
}

#[derive(Serialize)]
struct Transform {
    parent_id: i32,
}

pub fn test() {
    let test = TestStruct {
        thing: 1.111,
        test: 3,
    };
    let transform = Transform { parent_id: 1 };
    let mut yaml_serializer = YamlSerializer::new();
    test.serialize("", &mut yaml_serializer);
    transform.serialize("", &mut yaml_serializer);
    yaml_serializer.write("test_scene");

    //let test: SerializerNode = 4.0.serialize(&mut yaml_serializer);
}
