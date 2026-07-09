use uuid::Uuid;

//----Logic outline----
// 1. Serialize is implemented for types either by default or through a macro
// 2. Serialize trait implements a method that returns a SerializeNode
// 3. An AST is built from values that can be used for various other systems such as the inspector
// 4. Serializer takes the root node and implements methods to parse it into YAML, JSON, whatever

pub trait Serializer {
    // How to parse the AST
    fn serialize_int(&mut self);
    fn serialize_string(&mut self);
    fn serialize_struct(&mut self);
}

pub trait Serialize {
    fn serialize(&self, serializer: &mut impl Serializer) -> SerializerNode;
}

pub trait DeSerialize {}

pub struct YamlSerializer {
    output_ast_root: SerializerNode,
    output_buffer: String, //Later change to a data structure that represents fscn files as an AST?
}

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
    Float(),
    Integer(),
    Bool(),
}
