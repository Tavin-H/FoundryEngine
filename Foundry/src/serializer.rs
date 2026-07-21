use std::fs;

use foundry_derive::SerializeComponent;
use num::{Float, PrimInt};
use uuid::Uuid;

use crate::serializer;
use crate::{commands::UICommand::ShowUI, components::Component};
use yaml_rust2::{Event, parser::Parser};

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
    fn serialize_component(&mut self, node: &SerializerNode);
}

pub trait Deserializer {
    fn deserialize(&mut self, read_path: &'static str) {}
}

pub struct YamlSerializer {
    ast_roots: Vec<SerializerNode>,
    output_buffer: String, //Later change to a data structure that represents fscn files as an AST?
}

enum DocumentType {
    GameObject,
    Component,
}
impl Default for DocumentType {
    fn default() -> Self {
        Self::GameObject
    }
}

#[derive(Default)]
struct DocumentBuilder {
    //Shared fields
    doc_type: DocumentType,
    type_id: Option<u64>,
    local_file_id: Option<uuid::Uuid>,
    f_name: Option<String>,

    //Object only
    components: Option<Vec<uuid::Uuid>>,
    //Component only
    data_fields: Option<Vec<(String, SerializerNode)>>,
}
impl DocumentBuilder {
    pub fn new(doc_type: DocumentType) -> DocumentBuilder {
        DocumentBuilder {
            doc_type,
            ..Default::default()
        }
    }
    pub fn set_type_id(&mut self, type_id: u64) {
        self.type_id = Some(type_id)
    }
    pub fn set_local_file_id(&mut self, local_id: uuid::Uuid) {
        self.local_file_id = Some(local_id);
    }
    pub fn set_f_name(&mut self, f_name: String) {
        self.f_name = Some(f_name)
    }
    pub fn push_component(&mut self, comp_id: uuid::Uuid) {
        //Check if it's the right type
        if matches!(self.doc_type, DocumentType::Component) {
            panic!("Tried to add a component reference onto a component");
        }
        self.components.get_or_insert(Vec::new()).push(comp_id);
    }
    pub fn push_data_field(&mut self, data_field: (String, SerializerNode)) {
        if matches!(self.doc_type, DocumentType::GameObject) {
            panic!("Tried to add a component data_field to GameObject");
        }
        self.data_fields.get_or_insert(Vec::new()).push(data_field);
    }
}

impl Deserializer for YamlSerializer {
    fn deserialize(&mut self, read_path: &'static str) {
        let raw_path = format!("scenes/{read_path}.yaml");
        let full_path = std::path::Path::new(&raw_path);
        let Ok(contents) = fs::read_to_string(full_path) else {
            panic!("Uh oh")
        };
        let mut parser = Parser::new(contents.chars());
        let mut parsing_object: Option<SerializerNode> = None;
        while let Ok((event, marker)) = parser.next_token() {
            if event == Event::StreamEnd {
                panic!("end of file");
                break;
            }
            if event == Event::DocumentStart {
                println!("Found doc start");
                parsing_object = None;
                //Start component / Object
                //break;
                continue;
            }
            if let Event::Scalar(name, style, _, optional_tag) = event {
                println!("{}", name);
            } else if let Event::MappingStart(anchor_id, optional_tag) = event {
                if let Some(tag) = optional_tag {
                    parsing_object = Some(SerializerNode::Component {
                        foundry_type_id: 0,
                        local_file_id: Uuid::nil(),
                        f_name: "".to_string(),
                        data_fields: Vec::new(),
                    });
                    //Component
                    println!("Found object mapping");
                    println!("found tag: {}", tag.suffix);
                }
                println!("Found data mapping");
                continue;
            }
        }
    }
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
    fn serialize_component(&mut self, node: &SerializerNode) {
        let SerializerNode::Component {
            f_name,
            foundry_type_id,
            local_file_id,
            data_fields,
        } = node
        else {
            panic!("");
        };
        //self.output_buffer += "\n";
        self.output_buffer.push_str(&format!(
            "
---Component: &{foundry_type_id}
    id: 1
    type: {f_name}
    data:"
        ));
        for (field_name, node) in data_fields {
            let value = match node {
                SerializerNode::Float(val) => format!("{:.4}", val),
                SerializerNode::Integer(val) => val.to_string(),
                SerializerNode::String(val) => val.to_string(),
                SerializerNode::Bool(val) => val.to_string(),
                _ => panic!("Node not supported as data field"),
            };
            self.output_buffer
                .push_str(&format!("        {}: {}\n", field_name, value));
        }
        self.output_buffer.push_str("...");
        println!("Serializing struct {}", self.output_buffer.len())
    }
}

pub trait Serialize {
    fn serialize(&self, serializer: &mut impl Serializer) -> SerializerNode;
}

pub trait DeSerialize {}
// Serializer AST
#[derive(Debug)]
pub enum SerializerNode {
    // Documents
    Object {
        foundry_type_id: u64,
        local_file_id: Uuid,
        f_name: String,
        tags: Vec<String>,
        components: Vec<uuid::Uuid>,
    },

    Component {
        foundry_type_id: u64,
        local_file_id: Uuid,
        f_name: String, // Mainly for manual scene file editing
        data_fields: Vec<(String, SerializerNode)>, // A Struct node
    },

    // Custom data types
    Struct {
        f_name: String,
        data: Vec<(String, SerializerNode)>,
    },

    // Primatives
    Float(f64),
    Integer(i64),
    String(String),
    Bool(bool),
}

// Serialize Macros
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
impl_serialize!(i64, int);

impl_serialize!(u8, int);
impl_serialize!(u16, int);
impl_serialize!(u32, int);
impl_serialize!(u64, int);

// Floats
impl_serialize!(f32, float);
impl_serialize!(f64, float);

// Others
impl_serialize!(String, string);
impl_serialize!(bool, bool);

#[derive(SerializeComponent)]
pub struct TestStruct {
    thing: f32,
    test: i32,
}

pub fn test() {
    let test = TestStruct {
        thing: 1.111,
        test: 3,
    };
    let mut yaml_serializer = YamlSerializer::new();
    test.serialize(&mut yaml_serializer);
    //yaml_serializer.write("test_scene");
    yaml_serializer.deserialize("test_scene");

    //let test: SerializerNode = 4.0.serialize(&mut yaml_serializer);
}

pub struct GameObject {
    name: String,
    tags: Vec<String>,
    components: Vec<Box<dyn Component>>,
}

impl Serialize for GameObject {
    fn serialize(&self, serializer: &mut impl Serializer) -> SerializerNode {
        SerializerNode::Object {
            foundry_type_id: 1,
            local_file_id: uuid::Uuid::new_v4(),
            f_name: self.name.clone(),
            tags: self.tags.clone(),
            components: Vec::new(),
        }
    }
}
