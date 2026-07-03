use crate::commands::*;
use crate::delegator::RuntimeContext;
use crate::ecs::EntityBuilder;
use glam::Vec3;
use mlua::prelude::*;
use mlua::{MetaMethod, UserData};
use std::collections::HashMap;
use std::iter::Zip;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
// Execution
// Re-designing the structure...
// Make Lua instance per thread,
// Set lua bindings for each thread. Accessing it's own chunk

use uuid::Uuid;
type EntityId = Uuid;

pub struct LuaEngine {
    pub command_buffer_storage: Vec<CommandBuffer>,
    pub worker_cluster: Vec<LuaWorker>,
    pub worker_count: usize,
}

pub struct LuaWorker {
    lua_instance: Lua,
    update_functions: Vec<mlua::Function>,
}
unsafe impl Send for LuaWorker {}
impl LuaWorker {
    pub fn new() -> Self {
        let lua = Lua::new();
        let command_buffer = CommandBuffer::new();
        lua.set_app_data(command_buffer);
        //init bindings for lua passing in command_buffer
        let mut lua_worker = LuaWorker {
            lua_instance: lua,
            update_functions: Vec::new(),
        };
        lua_worker.init_bindings();
        lua_worker
    }

    pub fn bind_context(&mut self, ctx: &RuntimeContext) -> Result<(), mlua::Error> {
        let lua = &self.lua_instance;
        lua.globals().set("input", ctx.input_buffer_ref.clone());
        lua.globals().set("id", ctx.id_allocator_ref.clone());
        Ok(())
    }

    pub fn run_update_functions(
        &self,
        command_buffer_chunk: CommandBufferChunkRef,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.lua_instance.set_app_data(command_buffer_chunk);
        for func in self.update_functions.iter() {
            func.call::<()>(())?;
        }
        Ok(())
    }

    pub fn add_function(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let lua_program: String = std::fs::read_to_string(path).expect("Could not read lua file");
        self.lua_instance.load(lua_program).exec()?;
        let update_function: mlua::Function = self.lua_instance.globals().get("update")?;

        self.update_functions.push(update_function);
        Ok(())
    }

    pub fn init_bindings(&mut self) -> Result<(), mlua::Error> {
        let lua_instance = &mut self.lua_instance;
        lua_instance.globals().set(
            "Vec3",
            lua_instance
                .create_function(|_, (x, y, z): (f32, f32, f32)| Ok(LuaVec3 { x, y, z }))?,
        );
        let transform = lua_instance.create_table()?;
        transform.set(
            "Translate",
            lua_instance
                .create_function_mut(|lua, (id, lua_vec): (u128, mlua::Value)| {
                    let mut app_data = lua.app_data_mut::<CommandBufferChunkRef>().unwrap();
                    let entity_id = Uuid::from_u128(id);
                    let vec = lua_vec
                        .as_userdata()
                        .ok_or_else(|| mlua::Error::runtime("Bad conversion"))?
                        .borrow::<LuaVec3>()?;
                    println!("Moving {}, {}, {}", vec.x, vec.y, vec.z);
                    unsafe {
                        let command_buffer = &mut (*app_data.0)[0];
                        command_buffer.push(Command::Entity(
                            entity_id,
                            EntityCommand::Translate(Vec3::new(vec.x, vec.y, vec.z)),
                        ));
                    }
                    Ok(())
                })
                .unwrap(),
        );
        lua_instance.globals().set("transform", transform)?;

        let broadcaster = lua_instance.create_table()?;
        broadcaster.set(
            "BroadcastMessage",
            lua_instance
                .create_function_mut(move |lua, (message): (String)| {
                    let mut app_data = lua.app_data_mut::<CommandBufferChunkRef>().unwrap();
                    let index_id: u128 = lua.globals().get("index_id").expect("No index_id set");
                    unsafe {
                        let command_buffer = &mut (*app_data.0)[0];
                        command_buffer
                            .push(Command::Message(MessageCommand::BroadcastMessage(message)));
                    }
                    Ok(())
                })
                .unwrap(),
        );
        lua_instance.globals().set("broadcaster", broadcaster)?;

        Ok(())
    }
    //Run once at the start of every frame
    pub fn test(&mut self) {}
}

pub struct CommandBufferChunkRef(*mut [CommandBuffer]);
unsafe impl Send for CommandBufferChunkRef {}

impl LuaEngine {
    pub fn init(worker_count: usize) -> Result<Self, LuaError> {
        let worker_cluster: Vec<LuaWorker> = (0..worker_count).map(|_| LuaWorker::new()).collect();
        let command_buffer_storage: Vec<CommandBuffer> =
            (0..worker_count).map(|_| CommandBuffer::new()).collect();

        Ok(LuaEngine {
            worker_cluster,
            command_buffer_storage,
            worker_count,
        })
    }

    pub fn add_update_function(&mut self, path: &Path) {
        //JUST FOR TESTING, REPLACE LATER
        let res = self.worker_cluster[0].add_function(path);
        if let Err(e) = res {
            panic!("{}", e);
        }
    }

    pub fn run_update_cycle(&mut self, ctx: &RuntimeContext) {
        //Scope all threads to not outlive the function
        thread::scope(|s| {
            //Create an iterator of all lua instances
            let mut workers = self.worker_cluster.iter_mut();

            //I will need to change this to be N amount of chunks
            println!("cmd length: {}", self.command_buffer_storage.len());
            let command_buffer_chunks = self.command_buffer_storage.chunks_mut(1);

            //Batch workers and their allocated command_buffer_chunk
            let pairs = std::iter::zip(workers, command_buffer_chunks);

            for (worker_chunk, command_buffer_chunk) in pairs {
                //Make raw pointer to send across the thread
                let command_buffer_chunk_ref = CommandBufferChunkRef(command_buffer_chunk);

                //Spawn a new thread and have it capture the raw pointer
                s.spawn(move || {
                    println!("Spawned a thread!");
                    worker_chunk.bind_context(ctx);
                    //Run each respective lua script and fill the command_buffer
                    let res = worker_chunk.run_update_functions(command_buffer_chunk_ref);
                    if let Err(e) = res {
                        panic!("Lua script failed: {e}");
                    }
                });
            }
        });
    }

    /*
    pub fn execute_lua_behaviour(&self, id: u64, path: &Path) -> Result<&'static str, LuaError> {
        let lua = &self.lua_instance;

        let lua_program: String = std::fs::read_to_string(path).expect("Could not read lua file");
        lua.load(lua_program).exec()?;
        let update: mlua::Function = lua.globals().get("update")?;
        update.call::<()>(())?;
        Ok("")
    }
    */
}

//Further extraction can be made into a foundry types file
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
