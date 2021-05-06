use std::borrow::BorrowMut;

use cgmath::{Quaternion, Rotation};
use engine3d::{
    assets::ModelRef, collision, events::*, geom::*, render::InstanceGroups, run, world::World,
    Engine, DT,
};
use rand;
use winit;

const NUM_MARBLES: usize = 10;
const G: f32 = 1.0;

// leaving all "sparse" as false for now
pub struct Body_Plane(Plane);
pub struct Body_Sphere(Sphere);
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

struct Game {
    world: World,
    pw: Vec<collision::Contact<usize>>,
}
struct GameData {
    wall_model: engine3d::assets::ModelRef,
    player_model: engine3d::assets::ModelRef,
}

impl Game {
    fn integrate(&mut self) {
        // needs body, vels, accs, rots, omegas, and controls
        
        let mut planes = self.world.borrow_component_vec_mut::<Body_Plane>().unwrap();
        let controls = self.world.borrow_component_vec_mut::<Control>().unwrap();

        for (id, body) in planes.iter_mut().enumerate() {
            if let Some(body) = body {
                if let Some(c) = &controls[id] {
                    body.0.n += Vec3::new(
                        c.0.0 as f32 * 0.4 * DT,
                        0.0,
                        c.0.1 as f32 * 0.4 * DT,
                    );
                    body.0.n = body.0.n.normalize();
                }
            }
        }

        let mut spheres = self
            .world
            .borrow_component_vec_mut::<Body_Sphere>()
            .unwrap();
        let mut vels = self.world.borrow_component_vec_mut::<Velocity>().unwrap();
        let mut accs = self
            .world
            .borrow_component_vec_mut::<Acceleration>()
            .unwrap();
        let mut rots = self.world.borrow_component_vec_mut::<Rot>().unwrap();
        let mut omegas = self.world.borrow_component_vec_mut::<Omega>().unwrap();
        let zip = spheres
            .iter_mut()
            .zip(vels.iter_mut())
            .zip(accs.iter_mut())
            .zip(rots.iter_mut())
            .zip(omegas.iter_mut());
        let iter = zip.filter_map(|((((body, v), a), r), o)| {
            Some((
                body.as_mut()?,
                v.as_mut()?,
                a.as_mut()?,
                r.as_mut()?,
                o.as_mut()?,
            ))
        });

        for (body, v, a, r, o) in iter {
            // control sphere (includes the player)
            v.0 += ((r.0 * a.0) + Vec3::new(0.0, -G, 0.0)) * DT;
            body.0.c += v.0 * DT;
            r.0 += 0.5 * DT * Quat::new(0.0, o.0.x, o.0.y, o.0.z) * r.0;
        }
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
            Body_Plane(Plane {
                n: Vec3::new(0.0, 1.0, 0.0),
                d: 0.0,
            }),
        );
        world.add_component(wall, Control((0, 0)));

        // player has body, vel, acc, omega, and rot
        let player = world.add_entity();
        world.add_component(
            player,
            Body_Sphere(Sphere {
                c: Pos3::new(0.0, 3.0, 0.0),
                r: 0.3,
            }),
        );
        world.add_component(player, Velocity(Vec3::zero()));
        world.add_component(player, Acceleration(Vec3::zero()));
        world.add_component(player, Omega(Vec3::zero()));
        world.add_component(player, Rot(Quat::new(1.0, 0.0, 0.0, 0.0)));

        // add models to wall and player
        let wall_model = engine.load_model("floor.obj");
        let player_model = engine.load_model("sphere.obj");
        world.add_component(wall, Model(wall_model));
        world.add_component(player, Model(player_model));
        (
            Self {
                world,
                pw: vec![],
            },
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
            .borrow_component_vec_mut::<Body_Sphere>()
            .unwrap();
        let planes = self.world.borrow_component_vec_mut::<Body_Plane>().unwrap();
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

        let mut accs = self.world.borrow_component_vec_mut::<Acceleration>().unwrap();
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
        let mut planes = self.world.borrow_component_vec_mut::<Body_Plane>().unwrap();

        for (id, body) in planes.iter_mut().enumerate() {
            if let Some(body) = body {
                if let Some(c) = &controls[id] {
                    body.0.n += Vec3::new(
                        c.0.0 as f32 * 0.4 * DT,
                        0.0,
                        c.0.1 as f32 * 0.4 * DT,
                    );
                    body.0.n = body.0.n.normalize();
                }
            }
        }

        // integrate spheres
        // also sorry this is so gross
        let mut spheres = self
            .world
            .borrow_component_vec_mut::<Body_Sphere>()
            .unwrap();
        let mut vels = self.world.borrow_component_vec_mut::<Velocity>().unwrap();
        let mut rots = self.world.borrow_component_vec_mut::<Rot>().unwrap();
        let zip = spheres
            .iter_mut()
            .zip(vels.iter_mut())
            .zip(accs.iter_mut())
            .zip(rots.iter_mut())
            .zip(omegas.iter_mut());
        let iter = zip.filter_map(|((((body, v), a), r), o)| {
            Some((
                body.as_mut()?,
                v.as_mut()?,
                a.as_mut()?,
                r.as_mut()?,
                o.as_mut()?,
            ))
        });

        for (body, v, a, r, o) in iter {
            // control sphere (includes the player)
            v.0 += ((r.0 * a.0) + Vec3::new(0.0, -G, 0.0)) * DT;
            body.0.c += v.0 * DT;
            r.0 += 0.5 * DT * Quat::new(0.0, o.0.x, o.0.y, o.0.z) * r.0;
        }

        // collisions between player and floor
        self.pw.clear();

        let mut walls = vec![];
        for w in planes.iter() {
            if let Some(w) = w {
                walls.push(w.0);
            }
        }

        let mut pb = vec![];
        let mut pv = vec![];
        let mut player_id = 0;

        for (id, s) in spheres.iter().enumerate() {
            if let Some(s) = s {
                player_id = id;
                pb.push(s.0);
                if let Some(v) = &vels[id] {
                    pv.push(v.0);
                }
            }
        }

        collision::gather_contacts_ab(&pb, &walls, &mut self.pw);
        collision::restitute_dyn_stat(&mut pb, &mut pv, &walls, &mut self.pw);
        if let Some(body) = &mut spheres[player_id] {
            body.0 = pb[0];
        }
        if let Some(v) = &mut vels[player_id] {
            v.0 = pv[0];
        }

        for collision::Contact { a: pa, .. } in self.pw.iter() {
            // apply "friction" to players on the ground
            assert_eq!(*pa, 0);
            if let Some(v) = &mut vels[player_id] {
                v.0 *= 0.98;
            }
        }
    }
}

fn main() {
    env_logger::init();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new().with_title(title);
    run::<GameData, Game>(window, std::path::Path::new("content"));
}
