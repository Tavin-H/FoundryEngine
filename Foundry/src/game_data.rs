use std::{collections::VecDeque, time};

use nalgebra_glm::{self as glm};

#[derive(Default)]
pub struct MeshAllocation {
    pub index_count: u32,
    pub first_index: u32,
    pub first_vertex: i32,
}

#[derive(Default, Clone, Copy)]
pub struct Transform {
    pub position: [f32; 3],
}

#[derive(Default)]
pub struct GameObject {
    pub id: u32,
    pub name: String,
    pub _mesh: MeshAllocation,
    pub transform: Transform,
}

pub struct GameContext {
    pub delta_time_previous_frame: std::time::Instant,
    pub previous_delta_times: VecDeque<f32>,
    pub delta_time: f32,
}

impl Default for GameContext {
    fn default() -> Self {
        Self {
            delta_time_previous_frame: time::Instant::now(),
            previous_delta_times: VecDeque::from([0.0, 0.0, 0.0]),
            delta_time: 0.0,
        }
    }
}

impl GameContext {
    pub fn calculate_delta_time(&mut self) -> f32 {
        let mut delta_time = (time::Instant::now() - self.delta_time_previous_frame).as_nanos()
            as f32
            / 1000000000 as f32;
        //self.game_context.delta_time
        let mut avg_delta_time: f32 = 0.0;
        for i in 0..5 {
            match self.previous_delta_times.get(i) {
                Some(past_delta_time) => {
                    avg_delta_time += past_delta_time;
                }
                None => (),
            }
        }
        avg_delta_time /= 5.0;
        self.delta_time_previous_frame = time::Instant::now();
        self.previous_delta_times.push_back(delta_time);
        self.previous_delta_times.pop_front();
        avg_delta_time
    }
}
