use crate::commands::*;
use crate::components::RuntimeContext;
use crate::delegator::{InputBuffer, LuaInputBuffer};
use glam::Vec3;
use mlua::prelude::*;
use mlua::{MetaMethod, UserData};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct LuaEngine {
    pub lua_instance: Lua,
    pub command_buffer_index: Arc<Mutex<HashMap<u64, CommandBuffer>>>,
}

impl LuaEngine {
    pub fn init() -> Result<Self, LuaError> {
        let mut lua = Lua::new();
        let command_buffer_index = Arc::new(Mutex::new(HashMap::new()));
        LuaEngine::init_lua_globals(&mut lua, &command_buffer_index);

        Ok(LuaEngine {
            lua_instance: lua,
            command_buffer_index,
        })
    }

    //Give a path to a lua file and it will execute it
    pub fn excecute_lua_old(&self, path: &Path) -> Result<&'static str, LuaError> {
        let lua = &self.lua_instance;
        let lua_program: String = std::fs::read_to_string(path).expect("Could not read lua file");
        println!("EXECUTING LUA");
        lua.load(lua_program).exec()?;
        println!("OK");
        Ok("")
    }

    pub fn init_lua_globals(
        lua_instance: &mut Lua,
        command_buffer_index: &Arc<Mutex<HashMap<u64, CommandBuffer>>>,
    ) -> Result<&'static str, mlua::Error> {
        let mut command_buffer_index = Arc::clone(command_buffer_index);

        lua_instance.globals().set(
            "Vec3",
            lua_instance
                .create_function(|_, (x, y, z): (f32, f32, f32)| Ok(LuaVec3 { x, y, z }))?,
        );

        let transform = lua_instance.create_table()?;
        transform.set(
            "Translate",
            lua_instance
                .create_function_mut(move |_, (id, lua_vec): (u64, mlua::Value)| {
                    let vec = lua_vec
                        .as_userdata()
                        .ok_or_else(|| mlua::Error::runtime("Bad conversion"))?
                        .borrow::<LuaVec3>()?;
                    let mut map = command_buffer_index.lock().unwrap();
                    let command_buffer = map.entry(id).or_insert(CommandBuffer::new());
                    println!("Moving {}, {}, {}", vec.x, vec.y, vec.z);
                    command_buffer.push(Command::Entity(
                        id,
                        EntityCommand::Translate(Vec3::new(vec.x, vec.y, vec.z)),
                    ));
                    Ok(())
                })
                .unwrap(),
        );
        lua_instance.globals().set("transform", transform)?;
        Ok("")
    }
    pub fn batch_context(&mut self, ctx: &RuntimeContext) {}

    pub fn execute_lua_behaviour(
        &self,
        id: u64,
        path: &Path,
        ctx: &LuaInputBuffer, // move to batch
    ) -> Result<&'static str, LuaError> {
        let lua = &self.lua_instance;

        lua.globals().set("input", ctx.clone());

        let engine = lua.create_table()?;
        engine.set(
            "test",
            lua.create_function(|_, name: String| {
                println!("Testing rust binding");
                Ok(())
            })
            .unwrap(),
        );
        lua.globals().set("engine", engine)?;

        let lua_program: String = std::fs::read_to_string(path).expect("Could not read lua file");
        lua.load(lua_program).exec()?;
        Ok("")
    }
}

struct LuaVec3 {
    //Make this the central Vec3 for foundry?
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl UserData for LuaVec3 {
    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method(MetaMethod::Mul, |_, this, scalar: f32| {
            Ok(LuaVec3 {
                x: this.x * scalar,
                y: this.y * scalar,
                z: this.z * scalar,
            })
        });
    }
}
