use std::path::Path;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};

pub mod geom;

pub const DT: f32 = 1.0 / 60.0;
