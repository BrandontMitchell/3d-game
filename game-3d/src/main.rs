use std::borrow::BorrowMut;

use cgmath::{Quaternion, Rotation};
use engine3d::{//screen,
    assets::ModelRef, collision, events::*, geom::*, render::*, //render::InstanceGroups, 
    run, world::World,
    Engine, DT
};
use rand;
use winit;
//use winit::window::WindowBuilder;

use winit_input_helper::WinitInputHelper;
//use winit::event::{Event, VirtualKeyCode};
extern crate savefile;

use savefile::prelude::*;
//use pixels::{Pixels, SurfaceTexture};



extern crate savefile_derive;

const G: f32 = 10.0;
const MAX_PLAYER_VELOCITY: f32 = 20.0;
const PLANE_ROT_SPEED: f32 = 0.6;

const DEPTH: usize = 4;
const WIDTH: usize = 800;
const HEIGHT: usize = 500;

// leaving all "sparse" as false for now
pub struct BodyPlane(Plane);
pub struct BodySphere(Sphere);
// impl ComponentType for Body {
//     const sparse: bool = false;
// }
pub struct Velocity(Vec3);
// impl ComponentType for Velocity {
//     const sparse: bool = false;
// }
pub struct Rot(Quaternion<f32>);

// impl ComponentType for Rotation {
//     const sparse: bool = false;
// }

pub struct Acceleration(Vec3);
// impl ComponentType for Acceleration {
//     const sparse: bool = false;
// }

pub struct Omega(Vec3);
// impl ComponentType for Omega {
//     const sparse: bool = false;
// }

pub struct Control((i8, i8));
// impl ComponentType for Control {
//     const sparse: bool = false;
// }

pub struct Model(engine3d::assets::ModelRef);
// impl ComponentType for Model {
//     const sparse: bool = false;
// }

pub struct Linear_Momentum(Vec3);
pub struct Mass(f32);

#[derive(Debug, Copy, Clone)]
enum Mode {
    Title,
    Play(bool),
    Options,
    EndGame,
}

struct Game {
    world: World,
    pw: Vec<collision::Contact<usize>>,
}
struct GameData {
    wall_model: engine3d::assets::ModelRef,
    player_model: engine3d::assets::ModelRef,
}

//not fully implemented
impl Mode {
    // update consumes self and yields a new state (which might also just be self)
    fn update(self, input: &WinitInputHelper, engine: &mut Engine) -> Self {
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
    fn display(&self, screen: &mut Screen) {
        match self {
            Mode::Title => {
                //draw a (static?) title
                screen.clear(Rgba(0, 0, 0, 255));
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
}
impl Game {
    fn integrate(&mut self) {
    
    }
}

impl engine3d::Game for Game {
    type StaticData = GameData;
    
    fn start(engine: &mut Engine) -> (Self, Self::StaticData) {
        use rand::Rng;
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
        world.add_component(
            player,
            BodySphere(Sphere {
                c: Pos3::new(0.0, 3.0, 0.0),
                r: 0.3,
            }),
        );
        world.add_component(player, Velocity(Vec3::zero()));
        world.add_component(player, Acceleration(Vec3::zero()));
        world.add_component(player, Omega(Vec3::zero()));
        world.add_component(player, Rot(Quat::new(1.0, 0.0, 0.0, 0.0)));
        world.add_component(player, Linear_Momentum(Vec3::zero()));
        let mut mass = 0.0;
        if let Some(body) = &world.borrow_component_vec_mut::<BodySphere>().unwrap()[player] {
            mass = (body.0.r * 4.0).powi(3);
        }
        world.add_component(player, Mass(mass));

        // add models to wall and player
        let wall_model = engine.load_model("floor.obj");
        let player_model = engine.load_model("sphere.obj");
        world.add_component(wall, Model(wall_model));
        world.add_component(player, Model(player_model));
        (
            Self { world, pw: vec![] },
            GameData {
                wall_model,
                player_model,
            },
        )
    }
    fn render(&self, igs: &mut InstanceGroups) {
        // need shapes, their rotations, and their models
        let spheres = self
            .world
            .borrow_component_vec_mut::<BodySphere>()
            .unwrap();
        let planes = self.world.borrow_component_vec_mut::<BodyPlane>().unwrap();
        let models = self.world.borrow_component_vec_mut::<Model>().unwrap();
        let rots = self.world.borrow_component_vec_mut::<Rot>().unwrap();

        // render spheres
        for (id, body) in spheres.iter().enumerate() {
            if let Some(body) = body {
                if let Some(rot) = &rots[id] {
                    let ir = engine3d::render::InstanceRaw {
                        model: (Mat4::from_translation(body.0.c.to_vec())
                            * Mat4::from_scale(body.0.r)
                            * Mat4::from(rot.0))
                        .into(),
                    };
                    if let Some(model) = &models[id] {
                        igs.render(model.0, ir);
                    }
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

    fn update(&mut self, engine: &mut Engine) {
        let mut accs = self
            .world
            .borrow_component_vec_mut::<Acceleration>()
            .unwrap();
        let mut omegas = self.world.borrow_component_vec_mut::<Omega>().unwrap();

        // IF WE WANT PLAYER CONTROLS INSTEAD OF PLANE CONTROLS
        //
        // // player is currently the only thing w acceleration, so search by that for now
        // for (id, a) in accs.iter_mut().enumerate() {
        //     if let Some(a) = a {
        //         a.0 = Vec3::zero();
        //         if engine.events.key_held(KeyCode::W) {
        //             a.0.z = 1.0;
        //         } else if engine.events.key_held(KeyCode::S) {
        //             a.0.z = -1.0;
        //         }

        //         if engine.events.key_held(KeyCode::A) {
        //             a.0.x = 1.0;
        //         } else if engine.events.key_held(KeyCode::D) {
        //             a.0.x = -1.0;
        //         }
        //         if a.0.magnitude2() > 1.0 {
        //             a.0 = a.0.normalize();
        //         }
        //         if let Some(om) = &mut omegas[id] {
        //             if engine.events.key_held(KeyCode::Q) {
        //                 om.0 = Vec3::unit_y();
        //             } else if engine.events.key_held(KeyCode::E) {
        //                 om.0 = -Vec3::unit_y();
        //             } else {
        //                 om.0 = Vec3::zero();
        //             }
        //         }
        //     }
        // }

        // only one thing is controllable, so its fine for now
        let mut controls = self.world.borrow_component_vec_mut::<Control>().unwrap();
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
        let mut planes = self.world.borrow_component_vec_mut::<BodyPlane>().unwrap();

        for (id, body) in planes.iter_mut().enumerate() {
            if let Some(body) = body {
                if let Some(c) = &controls[id] {
                    body.0.n += Vec3::new(c.0.0 as f32 * PLANE_ROT_SPEED * DT, 0.0, c.0.1 as f32 * PLANE_ROT_SPEED * DT);
                    body.0.n = body.0.n.normalize();
                }
            }
        }

        // integrate spheres
        let mut spheres = self
            .world
            .borrow_component_vec_mut::<BodySphere>()
            .unwrap();
        let mut vels = self.world.borrow_component_vec_mut::<Velocity>().unwrap();
        let mut rots = self.world.borrow_component_vec_mut::<Rot>().unwrap();
        let mut ps = self.world.borrow_component_vec_mut::<Linear_Momentum>().unwrap();
        let mut masses = self.world.borrow_component_vec_mut::<Mass>().unwrap();

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

        for (id, s) in spheres.iter().enumerate() {
            if let Some(s) = s {
                player_id = id;
                pb.push(s.0);
                if let Some(v) = &vels[id] {
                    pv.push(v.0);
                }
                if let Some(p) = &ps[id] {
                    pp.push(p.0);
                }
                if let Some(m) = &masses[id] {
                    pm.push(m.0);
                }
            }
        }

        collision::gather_contacts_ab(&pb, &walls, &mut self.pw);
        collision::restitute_dyn_stat(&mut pb, &pv, &mut pp, &pm, &walls, &mut self.pw);
        if let Some(body) = &mut spheres[player_id] {
            body.0 = pb[0];
        }
        if let Some(p) = &mut ps[player_id] {
            p.0 = pp[0];
        }

        for collision::Contact { a: pa, .. } in self.pw.iter() {
            // apply "friction" to players on the ground
            assert_eq!(*pa, 0);
            if let Some(v) = &mut vels[player_id] {
                v.0 *= 0.98;
            }
        }

        // update spheres (apply gravity, momentum, etc)
        // hopefully this can be less gross if i do the sparse thing
        for (id, body) in spheres.iter_mut().enumerate() {
            // control sphere (includes the player)
            if let Some(body) = body {
                if let Some(p) = &mut ps[id] {
                    if let Some(v) = &mut vels[id] {
                        if let Some(m) = &masses[id] {
                            if let Some(r) = &mut rots[id] {
                                if let Some(a) = &accs[id] {
                                    if let Some(o) = &omegas[id] {
                                        p.0 += ((r.0 * a.0) + Vec3::new(0.0, -G, 0.0)) * DT;
                                        v.0 = p.0 / m.0;
                                        if v.0.magnitude() > MAX_PLAYER_VELOCITY {
                                            v.0 = v.0.normalize_to(MAX_PLAYER_VELOCITY);
                                        }
                                        body.0.c += v.0 * DT;
                                        r.0 += 0.5 * DT * Quat::new(0.0, o.0.x, o.0.y, o.0.z) * r.0;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    
    

    let mut mode = Mode::Title;
    let camera_position = Vec2i(0,0);


    //pixels.get_frame() needs to be replaced with framebuffer that works with render & its buffers
    //let mut screen = Screen::wrap(pixels.get_frame(), WIDTH, HEIGHT, DEPTH, camera_position);
    //screen.clear(CLEAR_COL);
    mode.display(&mut state, &mut data, &mut screen); 
    
    

    run::<GameData, Game>(window, std::path::Path::new("content"));
}
