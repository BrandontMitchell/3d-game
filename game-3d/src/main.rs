use std::borrow::BorrowMut;

use cgmath::{Quaternion, Rotation};

use engine3d::{
    assets::ModelRef,
    collision,
    components::Component,
    events::*,
    geom::*,
    lights::Light,
    render::{InstanceGroups, Rect, Rgba, Vec2i},
    run,
    screen::Screen,
    world::World,
    Engine, DT,
    camera_control::CameraController
};
use pixels::{Pixels, SurfaceTexture};

use rand;
use winit::{self, dpi::PhysicalSize};
//use winit::window::WindowBuilder;

use winit_input_helper::WinitInputHelper;
//use winit::event::{Event, VirtualKeyCode};
// extern crate savefile;

// use savefile::prelude::*;

// extern crate savefile_derive;

//use serde::{Serialize, Deserialize};


const G: f32 = 10.0;
const MAX_PLAYER_VELOCITY: f32 = 20.0;
const PLANE_ROT_SPEED: f32 = 0.6;

const DEPTH: usize = 4;
const WIDTH: usize = 800;
const HEIGHT: usize = 500;

// All components that are "sparse" are stored in hashmaps
// The others are in a vec of options
pub struct BodyPlane(Plane);
impl Component for BodyPlane {
    fn is_sparse(&self) -> bool {
        false
    }
}
pub struct BodySphere(Sphere);
impl Component for BodySphere {
    fn is_sparse(&self) -> bool {
        true
    }
}

pub struct EndSphere(Sphere);
impl Component for EndSphere {
    fn is_sparse(&self) -> bool {
        true
    }
}

pub struct Velocity(Vec3);
impl Component for Velocity {
    fn is_sparse(&self) -> bool {
        true
    }
}
pub struct Rot(Quaternion<f32>);
impl Component for Rot {
    fn is_sparse(&self) -> bool {
        false
    }
}

pub struct Acceleration(Vec3);
impl Component for Acceleration {
    fn is_sparse(&self) -> bool {
        true
    }
}

pub struct Omega(Vec3);
impl Component for Omega {
    fn is_sparse(&self) -> bool {
        false
    }
}

pub struct Control((i8, i8));
impl Component for Control {
    fn is_sparse(&self) -> bool {
        false
    }
}

pub struct Model(engine3d::assets::ModelRef);
impl Component for Model {
    fn is_sparse(&self) -> bool {
        false
    }
}

pub struct LinearMomentum(Vec3);
impl Component for LinearMomentum {
    fn is_sparse(&self) -> bool {
        true
    }
}
pub struct Mass(f32);
impl Component for Mass {
    fn is_sparse(&self) -> bool {
        true
    }
}
/* #[macro_use]
extern crate savefile;
use savefile::prelude::*;
#[macro_use]
use savefile::{WithSchema, Serialize, Deserialize};

#[macro_use]
extern crate savefile_derive;


#[derive(Copy, Serialize, Deserialize)] */
//#[derive(Savefile)]
//#[derive()]
//#[derive(Clone, Copy, Savefile)]
extern crate savefile;
use savefile::prelude::*;
//use savefile::{Serialize, Deserialize};

#[macro_use]
extern crate savefile_derive;
//use savefile_derive::Savefile;
//#[derive(Savefile)] /////////////////////////comment out this line to get rid of error
struct GameSave {
    world: World,
    light: Light,
}
//#[derive(Clone, Serialize, Deserialize)]
//#[repr(C)]
//#[derive(Savefile)]
struct Game {
    gamesave: GameSave,
    pw: Vec<collision::Contact<usize>>,
    mode: Mode,
    camera_controller: CameraController,
}

impl Game {
    fn integrate(&mut self) {
    
    }
}
struct GameData {
    wall_model: engine3d::assets::ModelRef,
    player_model: engine3d::assets::ModelRef,
    end_model: engine3d::assets::ModelRef,
}

#[derive(Debug, Copy, Clone)]
enum Mode {
    Title,
    Play(bool),
    Options,
    EndGame,
}

/* //not fully implemented
impl Mode {
    //update consumes self and yields a new state (which might also just be self)
    fn modeupdate(self, input: &WinitInputHelper, engine: &mut Engine) -> Self {
        match self {
            Mode::Title => {
                if engine.events.key_held(KeyCode::P) {
                    Mode::Play(false)
                }
                else if engine.events.key_held(KeyCode::O) {
                    Mode::Options
                }
                else if engine.events.key_held(KeyCode::Q) {
                    panic!();
                }
                else {
                    self
                }
            }
            //actively playing
            Mode::Play(paused) => {
                //if !paused {
                    //update game
                //}
                if engine.events.key_held(KeyCode::Space) {
                    Mode::Play(!paused)
                }
                else if engine.events.key_held(KeyCode::T) {
                    Mode::Title
                }
                else if engine.events.key_held(KeyCode::Q) {
                    panic!();
                }
                else if engine.events.key_held(KeyCode::O) {
                    Mode::Options
                }
                else {
                    self
                }
            }
            Mode::Options => {
                if engine.events.key_held(KeyCode::T) {
                    Mode::Title
                }
                else if engine.events.key_held(KeyCode::P) {
                    Mode::Play(false)
                }
                else if engine.events.key_held(KeyCode::Q) {
                    panic!();
                }
                else {
                    self
                }
            }
            //on play screen while dead
            Mode::EndGame => {
                if engine.events.key_held(KeyCode::T) {
                    Mode::Title
                }
                else if engine.events.key_held(KeyCode::P) {
                    Mode::Play(false)
                }
                else if engine.events.key_held(KeyCode::T) {
                    Mode::Title
                }
                else if engine.events.key_held(KeyCode::Q) {
                    panic!();
                }
                else if engine.events.key_held(KeyCode::O) {
                    Mode::Options
                }
                else {
                    self
                }
            }
        }
    }
    //screen reference needs to be changed
    fn modedisplay(&self, screen: &mut Screen) {
        match self {
            Mode::Title => {
                //draw a (static?) title
                screen.clear(Rgba(0, 0, 255, 255));
                let display_rect = Rect {
                    x: 0,
                    y: 0,
                    w: 250,
                    h: 51,
                };
            }
            Mode::Play(_paused) => {
                // Call screen's drawing methods to render the game state
                screen.clear(Rgba(80, 80, 80, 255));
            }
            Mode::Options => {
                screen.clear(Rgba(0, 0, 0, 255));
            }
            Mode::EndGame => { // Draw game result?
                screen.clear(Rgba(255, 255, 80, 255));
            }
        }
    }
} */

impl engine3d::Game for Game {
    type StaticData = GameData;

    fn start(engine: &mut Engine) -> (Self, Self::StaticData) {
        let mut world = World::new();

        // wall has body and control
        let wall = world.add_entity();
        world.add_component(
            wall,
            BodyPlane(Plane {
                n: Vec3::new(0.0, 1.0, 0.0),
                d: 0.0,
            }),
        );
        world.add_component(wall, Control((0, 0)));

        // player has body, vel, acc, omega, rot, momentum, and mass
        let player = world.add_entity();
        let r = 0.3;
        world.add_component(
            player,
            BodySphere(Sphere {
                c: Pos3::new(0.0, 3.0, 0.0),
                r,
            }),
        );
        world.add_component(player, Velocity(Vec3::zero()));
        world.add_component(player, Acceleration(Vec3::zero()));
        world.add_component(player, Omega(Vec3::zero()));
        world.add_component(player, Rot(Quat::new(1.0, 0.0, 0.0, 0.0)));
        world.add_component(player, LinearMomentum(Vec3::zero()));
        let mass = (r * 4.0).powi(3);
        world.add_component(player, Mass(mass));

        let end_sphere = world.add_entity();
        let r = 0.8;
        world.add_component(
            end_sphere,
            EndSphere(Sphere {
                c: Pos3::new(3.0, 1.0, 8.0), // todo make random and match plane height
                r,
            }),
        );
        world.add_component(end_sphere, Velocity(Vec3::zero()));
        world.add_component(end_sphere, Acceleration(Vec3::zero()));
        world.add_component(end_sphere, Omega(Vec3::zero()));
        world.add_component(end_sphere, Rot(Quat::new(1.0, 0.0, 0.0, 0.0)));
        world.add_component(end_sphere, LinearMomentum(Vec3::zero()));

        // add models to wall and player
        let wall_model = engine.load_model("floor.obj");
        let player_model = engine.load_model("sphere.obj");
        let end_model = engine.load_model("sphere.obj");
        world.add_component(wall, Model(wall_model));
        world.add_component(player, Model(player_model));
        world.add_component(end_sphere, Model(end_model));

        engine.set_ambient(0.05);
        let light = Light::point(Pos3::new(0.0, 10.0, 0.0), Vec3::new(1.0, 1.0, 1.0));

        let game_save = GameSave{world: world, light: light};
        let camera_controller = CameraController::new(0.2);
        (
/*             Self {
                world,
                pw: vec![],
                light,
                mode: Mode::Title,
            }, */
            Self {
                gamesave: game_save,
                pw: vec![],
                mode: Mode::Title,
                camera_controller
            },
            GameData {
                wall_model,
                player_model,
                end_model,
            },
        )
    }

    // Returns true if rendering in 2d, false otherwise
    fn render(&self, igs: &mut InstanceGroups, pixels: &mut (Pixels, PhysicalSize<u32>)) -> bool {
        match self.mode {
            Mode::Title => {
                let mut screen = Screen::wrap(
                    pixels.0.get_frame(),
                    pixels.1.width as usize,
                    pixels.1.height as usize,
                    DEPTH,
                    Vec2i(0, 0),
                );
                screen.clear(Rgba(0, 0, 255, 0));

/*                 let w = WIDTH as i32;
                let h = HEIGHT as i32;
                let menu_rect = Rect {
                    x: w / 6,
                    y: h / 8,
                    w: (2 * w as u16) / 3,
                    h: (h as u16) / 2,
                };

                screen.rect(menu_rect, Rgba(20, 0, 100, 255));
                screen.empty_rect(menu_rect, 4, Rgba(200, 220, 255, 255)); */
                pixels.0.render().unwrap();
                return true;
            }
            Mode::EndGame => {}
            Mode::Options => {}
            Mode::Play(live) => {
                // need shapes, their rotations, and their models
                let spheres = self.gamesave.world
                    .borrow_components_sparse_mut::<BodySphere>()
                    .unwrap();
                let end_objects = self.gamesave.world.borrow_components_sparse_mut::<EndSphere>().unwrap();
                let planes = self.gamesave.world.borrow_components_mut::<BodyPlane>().unwrap();
                let models = self.gamesave.world.borrow_components_mut::<Model>().unwrap();
                let rots = self.gamesave.world.borrow_components_mut::<Rot>().unwrap();

                // render spheres
                for (id, body) in spheres.iter() {
                    if let Some(rot) = &rots[*id] {
                        let ir = engine3d::render::InstanceRaw {
                            model: (Mat4::from_translation(body.0.c.to_vec())
                                * Mat4::from_scale(body.0.r)
                                * Mat4::from(rot.0))
                            .into(),
                        };
                        if let Some(model) = &models[*id] {
                            igs.render(model.0, ir);
                        }
                    }
                }

                // end spheres
                for (id, body) in end_objects.iter() {
                    if let Some(rot) = &rots[*id] {
                        let ir = engine3d::render::InstanceRaw {
                            model: (Mat4::from_translation(body.0.c.to_vec())
                                * Mat4::from_scale(body.0.r)
                                * Mat4::from(rot.0))
                            .into(),
                        };
                        if let Some(model) = &models[*id] {
                            igs.render(model.0, ir);
                        }
                    }
                }


                // render planes
                for (id, body) in planes.iter().enumerate() {
                    if let Some(body) = body {
                        let ir = engine3d::render::InstanceRaw {
                            model: (Mat4::from(cgmath::Quaternion::between_vectors(
                                Vec3::new(0.0, 1.0, 0.0),
                                body.0.n,
                            )) * Mat4::from_translation(Vec3::new(0.0, -0.025, 0.0))
                                * Mat4::from_nonuniform_scale(0.5, 0.05, 0.5))
                            .into(),
                        };
                        if let Some(model) = &models[id] {
                            igs.render(model.0, ir);
                        }
                    }
                }
            }
        }
        false
    }

    fn update(&mut self, engine: &mut Engine) {
        match self.mode {
            Mode::Title => {
                if engine.events.key_held(KeyCode::Return) || engine.events.key_held(KeyCode::P){
                    self.mode = Mode::Play(true);
                } else if engine.events.key_held(KeyCode::O) {
                    self.mode = Mode::Options;
                } else if engine.events.key_held(KeyCode::Q) {
                    panic!();
                }
            }
            Mode::Options => {
                if engine.events.key_held(KeyCode::P) {
                    self.mode = Mode::Play(true);
                } else if engine.events.key_held(KeyCode::Q) {
                    panic!();
                }
            }
            Mode::EndGame => {}
            Mode::Play(live) => {
                if engine.events.key_held(KeyCode::O) {
                    self.mode = Mode::Options;
                } else if engine.events.key_held(KeyCode::Q) {
                    panic!();
                }
                self.camera_controller.update(engine);
                let mut accs = self.gamesave.world
                    .borrow_components_sparse_mut::<Acceleration>()
                    .unwrap();
                let mut omegas = self.gamesave.world.borrow_components_mut::<Omega>().unwrap();

                // only one thing is controllable, so its fine for now
                let mut controls = self.gamesave.world.borrow_components_mut::<Control>().unwrap();
                for c in controls.iter_mut() {
                    if let Some(cont) = c {
                        cont.0 .0 = if engine.events.key_held(KeyCode::A) {
                            -1
                        } else if engine.events.key_held(KeyCode::D) {
                            1
                        } else {
                            0
                        };
                        cont.0 .1 = if engine.events.key_held(KeyCode::W) {
                            -1
                        } else if engine.events.key_held(KeyCode::S) {
                            1
                        } else {
                            0
                        };
                    }
                }

                // integrate planes
                let mut planes = self.gamesave.world.borrow_components_mut::<BodyPlane>().unwrap();

                for (id, body) in planes.iter_mut().enumerate() {
                    if let Some(body) = body {
                        if let Some(c) = &controls[id] {
                            body.0.n += Vec3::new(
                                c.0 .0 as f32 * PLANE_ROT_SPEED * DT,
                                0.0,
                                c.0 .1 as f32 * PLANE_ROT_SPEED * DT,
                            );
                            body.0.n = body.0.n.normalize();
                        }
                    }
                }

                // integrate spheres
                let mut spheres = self.gamesave
                    .world
                    .borrow_components_sparse_mut::<BodySphere>()
                    .unwrap();
                let mut vels = self.gamesave
                    .world
                    .borrow_components_sparse_mut::<Velocity>()
                    .unwrap();
                let mut rots = self.gamesave.world.borrow_components_mut::<Rot>().unwrap();
                let mut ps = self.gamesave
                    .world
                    .borrow_components_sparse_mut::<LinearMomentum>()
                    .unwrap();
                let mut masses = self.gamesave.world.borrow_components_sparse_mut::<Mass>().unwrap();

                // collisions between player and floor
                self.pw.clear();

                let mut walls = vec![];
                for w in planes.iter() {
                    if let Some(w) = w {
                        walls.push(w.0);
                    }
                }

                // get values for bodies, velocities, momentums, and masses for collision
                let mut pb = vec![];
                let mut pv = vec![];
                let mut pp = vec![];
                let mut pm = vec![];
                let mut player_id = 0;

                for (id, s) in spheres.iter() {
                    player_id = *id;
                    pb.push(s.0);
                    pv.push(vels[&id].0);
                    pp.push(ps[&id].0);
                    pm.push(masses[&id].0);
                }

                collision::gather_contacts_ab(&pb, &walls, &mut self.pw);
                collision::restitute_dyn_stat(&mut pb, &pv, &mut pp, &pm, &walls, &mut self.pw);
                spheres.get_mut(&player_id).unwrap().0 = pb[0];
                ps.get_mut(&player_id).unwrap().0 = pp[0];

                for collision::Contact { a: pa, .. } in self.pw.iter() {
                    // apply "friction" to players on the ground
                    assert_eq!(*pa, 0);
                    vels.get_mut(&player_id).unwrap().0 *= 0.98;
                }

                // update spheres (apply gravity, momentum, etc)
                for (id, body) in spheres.iter_mut() {
                    // control sphere (includes the player)
                    let m = &masses[&id];
                    let a = &accs[&id];
                    if let Some(r) = &mut rots[*id] {
                        if let Some(o) = &omegas[*id] {
                            ps.get_mut(&id).unwrap().0 +=
                                ((r.0 * a.0) + Vec3::new(0.0, -G, 0.0)) * DT;
                            vels.get_mut(&id).unwrap().0 = ps[&id].0 / m.0;
                            if vels[&id].0.magnitude() > MAX_PLAYER_VELOCITY {
                                vels.get_mut(&id).unwrap().0 =
                                    vels[&id].0.normalize_to(MAX_PLAYER_VELOCITY);
                            }
                            body.0.c += vels[&id].0 * DT;
                            r.0 += 0.5 * DT * Quat::new(0.0, o.0.x, o.0.y, o.0.z) * r.0;
                        }
                    }
                }

                // lights
                let light_pos = self.gamesave.light.position();
                let light_pos = if engine.events.key_held(KeyCode::A) {
                    Quat::from(cgmath::Euler::new(
                        cgmath::Deg(0.0),
                        cgmath::Deg(-90.0 * DT),
                        cgmath::Deg(0.0),
                    ))
                    .rotate_point(light_pos)
                } else if engine.events.key_held(KeyCode::D) {
                    Quat::from(cgmath::Euler::new(
                        cgmath::Deg(0.0),
                        cgmath::Deg(90.0 * DT),
                        cgmath::Deg(0.0),
                    ))
                    .rotate_point(light_pos)
                } else {
                    light_pos
                };
                self.gamesave.light = Light::point(light_pos, self.gamesave.light.color());
                engine.set_lights(vec![self.gamesave.light]);
            }
        }
    }
}

/* fn save_game(game:&Game) {
    save_file("save_marble.bin", 0, game).unwrap();
}

fn load_game() -> Game {
    load_file("save_marble.bin", 0).unwrap()
}  */

fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);

    let mut mode = Mode::Title;
    //let camera_position = Vec2i(0,0);

    //pixels.get_frame() needs to be replaced with framebuffer that works with render & its buffers
    //let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, camera_position);
    //screen.clear(CLEAR_COL);

    //mode.display(&mut screen); 


    run::<GameData, Game>(window, std::path::Path::new("content"));
}
