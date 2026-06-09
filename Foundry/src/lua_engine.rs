use mlua::prelude::*;
use std::path::Path;

pub struct LuaEngine {
    lua_instance: Lua,
}

impl LuaEngine {
    pub fn init() -> Result<Self, LuaError> {
        let lua = Lua::new();
        /*
                let map_table = lua.create_table()?;
                map_table.set(1, "one")?;
                map_table.set("two", 2)?;

                lua.globals().set("map_table", map_table)?;

                lua.load("for k,v in pairs(map_table) do print(k,v) end")
                    .exec()?;
        */

        Ok(LuaEngine { lua_instance: lua })
    }

    //Give a path to a lua file and it will execute it
    pub fn excecute_lua(&self, path: &Path) -> Result<&'static str, LuaError> {
        let lua = &self.lua_instance;
        let lua_program: String = std::fs::read_to_string(path).expect("Could not read lua file");
        println!("EXECUTING LUA");
        lua.load(lua_program).exec()?;
        println!("OK");
        Ok("")
    }
}
