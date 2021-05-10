use pixels::{Pixels, SurfaceTexture};
use std::path::Path;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    platform::run_return::EventLoopExtRunReturn,
};
#[macro_use]
extern crate savefile_derive;

pub mod camera;
pub mod collision;
pub mod events;
pub mod geom;
pub mod model;
pub mod texture;
use events::Events;
pub mod render;
use render::{InstanceGroups, Render};
pub mod assets;
use assets::Assets;
pub mod camera_control;
pub mod components;
pub mod lights;
pub mod screen;
pub mod text;
pub mod world;

pub const DT: f32 = 1.0 / 60.0;

pub struct Engine {
    pub frame: usize,
    pub assets: Assets,
    render: Render,
    pub events: Events,
    pub pixels: (Pixels, PhysicalSize<u32>),
}

impl Engine {
    pub fn load_model(&mut self, model: impl AsRef<Path>) -> assets::ModelRef {
        self.assets.load_model(
            &self.render.device,
            &self.render.queue,
            &self.render.texture_layout,
            model,
        )
    }
    pub fn camera_mut(&mut self) -> &mut camera::Camera {
        &mut self.render.camera
    }
    pub fn set_ambient(&mut self, amb: f32) {
        self.render.set_ambient(amb);
    }
    pub fn set_lights(&mut self, lights: impl IntoIterator<Item = lights::Light>) {
        self.render.set_lights(lights.into_iter().collect());
    }
}

pub trait Game: Sized {
    type StaticData;
    fn start(engine: &mut Engine) -> (Self, Self::StaticData);
    fn update(&mut self, engine: &mut Engine);
    fn render(
        &mut self,
        igs: &mut InstanceGroups,
        pixels: &mut (Pixels, PhysicalSize<u32>),
    ) -> bool;
}

pub fn run<R, G: Game<StaticData = R>>(
    window_builder: winit::window::WindowBuilder,
    asset_root: &Path,
) {
    use std::time::Instant;
    let mut event_loop = EventLoop::new();
    let window = window_builder.build(&event_loop).unwrap();
    let assets = Assets::new(asset_root);
    use futures::executor::block_on;
    let render = block_on(Render::new(&window));
    let events = Events::default();

    let window_size = window.inner_size();
    let mut pixels = (
        {
            let surface_texture =
                SurfaceTexture::new(window_size.width, window_size.height, &window);
            Pixels::new(
                window_size.width as u32,
                window_size.height as u32,
                surface_texture,
            )
            .unwrap()
        },
        window_size,
    );

    let mut engine = Engine {
        assets,
        render,
        events,
        frame: 0,
        pixels,
    };
    let (mut game, rules) = G::start(&mut engine);
    // How many unsimulated frames have we saved up?
    let mut available_time: f32 = 0.0;
    let mut since = Instant::now();

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent { ref event, .. } => engine.events.device_event(event),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                engine.events.window_event(event);
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        engine.render.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        engine.render.resize(**new_inner_size);
                    }
                    _ => {}
                }
            }
            Event::RedrawRequested(_) => {
                match engine.render.render(
                    &mut game,
                    &rules,
                    &mut engine.assets,
                    &mut engine.pixels,
                ) {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => engine.render.resize(engine.render.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
                // The renderer "produces" time...
                available_time += since.elapsed().as_secs_f32();
                since = Instant::now();
            }
            _ => {}
        }
        // And the simulation "consumes" it
        while available_time >= DT {
            // Eat up one frame worth of time
            available_time -= DT;

            game.update(&mut engine);

            engine.events.next_frame();
            engine.frame += 1;
        }
    });
}
