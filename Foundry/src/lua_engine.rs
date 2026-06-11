use crate::commands::*;
use glam::Vec3;
use mlua::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct LuaEngine {
    pub lua_instance: Lua,
    pub command_buffer_index: Arc<Mutex<HashMap<u64, CommandBuffer>>>,
}

impl LuaEngine {
    pub fn init() -> Result<Self, LuaError> {
        let lua = Lua::new();
        Ok(LuaEngine {
            lua_instance: lua,
            command_buffer_index: Arc::new(Mutex::new(HashMap::new())),
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

    pub fn init_lua_globals(&mut self) -> Result<&'static str, mlua::Error> {
        let lua = &self.lua_instance;
        let mut command_buffer_index = Arc::clone(&self.command_buffer_index);

        let transform = lua.create_table()?;
        transform.set(
            "translate",
            lua.create_function_mut(move |_, id: u64| {
                let mut map = command_buffer_index.lock().unwrap();
                let Some(command_buffer) = map.get_mut(&id) else {
                    panic!("")
                };
                command_buffer.push(Command::Entity(
                    id,
                    EntityCommand::Translate(Vec3::new(-1.0, 0.0, 0.0)),
                ));

                Ok(())
            })
            .unwrap(),
        );
        lua.globals().set("transform", transform)?;
        Ok("")
    }

    pub fn excecute_lua_behaviour(&self, id: u64, path: &Path) -> Result<&'static str, LuaError> {
        let lua = &self.lua_instance;

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
        println!("EXECUTING LUA");
        lua.load(lua_program).exec()?;
        Ok("")
    }
}
