// Falling spheres and cubes on a ground plane, simulated by `physicsplay-core`
// (Rapier underneath) and rendered with kiss3d.
//
// This is the PhysicsPlay layering in miniature:
//
//     physicsplay-core  ->  World, entities, laws, events, sensors
//     kiss3d            ->  reads poses from the World, draws meshes
//
// The example never touches rapier directly: it talks to `World`, steps it with
// a fixed timestep, and copies poses into scene nodes (types match directly:
// rapier 0.35 uses glamx math, the same `Vec3` / `Quat` that kiss3d re-exports).
//
// Native: `cargo run --example physics3d`
// WASM:   `cargo build --target wasm32-unknown-unknown --example physics3d`
// Browser: cd examples/physics3d-web && trunk serve

use kiss3d::prelude::*;
use physicsplay_core::{BodyKind, Entity, Material, Shape, World, WorldParams};
use rand::RngExt;

const FIXED_DT: f32 = 1.0 / 60.0;
const SPHERE_RADIUS: f32 = 0.6;

#[kiss3d::main]
async fn main() {
    env_logger::init();
    let mut window = Window::new("PhysicsPlay: falling bodies").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(12.0, 9.0, 18.0), Vec3::new(0.0, 2.0, 0.0));
    let mut scene = SceneNode3d::empty();

    scene
        .add_light(Light::point(120.0).with_intensity(5.0))
        .set_position(Vec3::new(6.0, 10.0, 6.0));
    window.set_background_color(Color::new(0.02, 0.02, 0.04, 1.0));

    // --- Physics world (physicsplay-core) ----------------------------------
    let mut world = World::new(WorldParams {
        gravity: Vec3::new(0.0, -9.81, 0.0),
        ..WorldParams::default()
    });

    // Ground: fixed entity with half-extent (25, 0.5, 25), drawn as a 50x1x50
    // cube.
    let _ground_id = world.spawn(Entity {
        translation: Vec3::new(0.0, -0.5, 0.0),
        shape: Shape::Cuboid {
            half_extents: Vec3::new(25.0, 0.5, 25.0),
        },
        kind: BodyKind::Fixed,
        material: Material {
            friction: 0.8,
            restitution: 0.0,
            ..Material::default()
        },
        ..Entity::default()
    });
    scene
        .add_cube(50.0, 1.0, 50.0)
        .set_position(Vec3::new(0.0, -0.5, 0.0))
        .set_color(GRAY);

    // Dynamic bodies: 6 spheres + 3 cubes, each with a matching mesh.
    // (entity id, scene node) pairs, synced every frame.
    let mut rng = rand::rng();
    let palette = [RED, YELLOW, LIME, CYAN, MAGENTA, BLUE];
    let mut objects: Vec<(usize, SceneNode3d)> = Vec::new();

    for i in 0..6 {
        let x = rng.random_range(-4.0..4.0);
        let z = rng.random_range(-4.0..4.0);
        let y = 4.0 + i as f32 * 2.0;
        let mut entity = Entity::new(
            Shape::Sphere {
                radius: SPHERE_RADIUS,
            },
            BodyKind::Dynamic,
            Vec3::new(x, y, z),
        );
        entity.angvel = Vec3::new(
            rng.random_range(-2.0..2.0),
            rng.random_range(-2.0..2.0),
            rng.random_range(-2.0..2.0),
        );
        entity.material.restitution = 0.6;
        entity.material.friction = 0.5;
        let id = world.spawn(entity);

        let node = scene
            .add_sphere(SPHERE_RADIUS)
            .set_color(palette[i % palette.len()]);
        objects.push((id, node));
    }

    for i in 0..3 {
        let hx = 0.5 + rng.random_range(0.0..0.4);
        let hy = 0.5 + rng.random_range(0.0..0.4);
        let hz = 0.5 + rng.random_range(0.0..0.4);
        let x = rng.random_range(-4.0..4.0);
        let z = rng.random_range(-4.0..4.0);
        let y = 6.0 + i as f32 * 3.0;
        let mut entity = Entity::new(
            Shape::Cuboid {
                half_extents: Vec3::new(hx, hy, hz),
            },
            BodyKind::Dynamic,
            Vec3::new(x, y, z),
        );
        entity.angvel = Vec3::new(
            rng.random_range(-3.0..3.0),
            rng.random_range(-3.0..3.0),
            rng.random_range(-3.0..3.0),
        );
        entity.material.restitution = 0.4;
        let id = world.spawn(entity);

        let node = scene
            .add_cube(hx * 2.0, hy * 2.0, hz * 2.0)
            .set_color(palette[(i + 3) % palette.len()]);
        objects.push((id, node));
    }

    // --- Fixed-timestep simulation + render loop ---------------------------
    let mut accumulator = 0.0f32;
    let mut last = web_time::Instant::now();

    while window.render_3d(&mut scene, &mut camera).await {
        let now = web_time::Instant::now();
        accumulator += (now - last).as_secs_f32();
        last = now;
        // Clamp long pauses (e.g. window dragged to another workspace) so the
        // simulation does not explode catching up.
        accumulator = accumulator.min(0.25);

        while accumulator >= FIXED_DT {
            world.step(FIXED_DT);
            accumulator -= FIXED_DT;
        }

        // Sync physics transforms to the scene. physicsplay-core uses the same
        // glamx types as kiss3d, so no conversion is needed.
        for (id, node) in &mut objects {
            node.set_position(world.translation(*id));
            node.set_rotation(world.rotation(*id));
        }

        // Events are produced by the world and drained here. The measurement
        // and block layers (M1/M3) will consume these; for now we just keep
        // the queue empty.
        let _events = world.drain_events();
    }
}
