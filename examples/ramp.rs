// PhysicsPlay Lab - Experiment 5: Ramps (tilted plane).
//
// A block slides down a ramp. The acceleration along the incline is
// a = g * sin(theta) (minus a little friction). Tilt the ramp live with the
// slider and watch the measured acceleration chase the theory value.
//
// Run with the `egui` feature:
//   cargo run --features egui --example ramp

#[cfg(feature = "egui")]
mod lab;

#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled: cargo run --features egui --example ramp");
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    use kiss3d::prelude::*;
    use physicsplay_core::{BodyKind, Entity, Material, Shape, World, WorldParams};
    use std::f32::consts::PI;

    env_logger::init();
    let mut window = Window::new("PhysicsPlay Lab: Ramps").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(0.0, 10.0, 18.0), Vec3::new(0.0, 3.0, 0.0));
    let mut scene = SceneNode3d::empty();

    scene
        .add_light(Light::point(150.0).with_intensity(5.0))
        .set_position(Vec3::new(4.0, 14.0, 8.0));
    window.set_background_color(Color::new(0.02, 0.02, 0.04, 1.0));

    const FIXED_DT: f32 = 1.0 / 60.0;
    const RAMP_CENTER: Vec3 = Vec3::new(0.0, 4.0, 0.0);
    const RAMP_HALF_LEN: f32 = 6.0;
    const RAMP_HALF_THICK: f32 = 0.3;
    const BLOCK_HALF: f32 = 0.25;

    let mut world = World::new(WorldParams::default());

    // Floor to catch the block after it leaves the ramp.
    let _floor = world.spawn(Entity {
        translation: Vec3::new(0.0, -0.5, 0.0),
        shape: Shape::Cuboid {
            half_extents: Vec3::new(30.0, 0.5, 20.0),
        },
        kind: BodyKind::Fixed,
        material: Material {
            friction: 0.4,
            restitution: 0.0,
            ..Material::default()
        },
        ..Entity::default()
    });
    scene
        .add_cube(60.0, 1.0, 40.0)
        .set_position(Vec3::new(0.0, -0.5, 0.0))
        .set_color(GRAY);

    // The ramp: a rotated cuboid, tilted about the Z axis.
    let ramp = world.spawn(Entity {
        translation: RAMP_CENTER,
        shape: Shape::Cuboid {
            half_extents: Vec3::new(RAMP_HALF_LEN, RAMP_HALF_THICK, 2.0),
        },
        kind: BodyKind::Fixed,
        material: Material {
            friction: 0.02,
            restitution: 0.0,
            ..Material::default()
        },
        ..Entity::default()
    });
    let mut ramp_node = scene
        .add_cube(RAMP_HALF_LEN * 2.0, RAMP_HALF_THICK * 2.0, 4.0)
        .set_position(RAMP_CENTER)
        .set_color(Color::new(0.55, 0.45, 0.35, 1.0));

    // Sliding block (a cube slides; a ball would roll and change the physics).
    let block = world.spawn(Entity {
        translation: RAMP_CENTER,
        shape: Shape::Cuboid {
            half_extents: Vec3::splat(BLOCK_HALF),
        },
        kind: BodyKind::Dynamic,
        material: Material {
            friction: 0.02,
            restitution: 0.0,
            ..Material::default()
        },
        ..Entity::default()
    });
    let mut block_node = scene
        .add_cube(BLOCK_HALF * 2.0, BLOCK_HALF * 2.0, BLOCK_HALF * 2.0)
        .set_color(RED);

    /// World-space spawn point for the block: near the ramp's high end, just
    /// above the tilted surface. The ramp rotates about Z, so its local +X end
    /// rises when theta > 0; we start the block on the high side.
    fn block_spawn(theta_rad: f32) -> Vec3 {
        let q = Quat::from_axis_angle(Vec3::Z, theta_rad);
        let local = Vec3::new(
            RAMP_HALF_LEN - 1.5,
            RAMP_HALF_THICK + BLOCK_HALF + 0.05,
            0.0,
        );
        RAMP_CENTER + q * local
    }

    // --- Lab state ------------------------------------------------------------
    let mut theta_deg = 20.0f32;
    let mut prev_speed = 0.0f32;
    let mut a_measured = 0.0f32; // exponentially smoothed estimate
    let mut speed_chart =
        lab::instruments::StripChart::new("block speed", egui::Color32::from_rgb(240, 110, 110));
    let mut reset_requested = false;

    // Initial placement.
    world.set_rotation(ramp, Quat::from_axis_angle(Vec3::Z, theta_deg * PI / 180.0));
    ramp_node.set_rotation(Quat::from_axis_angle(Vec3::Z, theta_deg * PI / 180.0));
    world.set_translation(block, block_spawn(theta_deg * PI / 180.0));

    // --- Fixed-timestep simulation + render loop ------------------------------
    let mut accumulator = 0.0f32;
    let mut last = web_time::Instant::now();

    while window.render_3d(&mut scene, &mut camera).await {
        let now = web_time::Instant::now();
        accumulator += (now - last).as_secs_f32();
        last = now;
        accumulator = accumulator.min(0.25);

        while accumulator >= FIXED_DT {
            let v = world.speed(block);
            let a_inst = (v - prev_speed) / FIXED_DT;
            prev_speed = v;
            // Smooth: 10% of each new sample.
            a_measured = a_measured * 0.9 + a_inst * 0.1;

            world.step(FIXED_DT);
            speed_chart.push(world.time(), world.speed(block));
            accumulator -= FIXED_DT;
        }

        block_node.set_position(world.translation(block));
        block_node.set_rotation(world.rotation(block));

        window.draw_ui(|ctx| {
            egui::Window::new("World laws").show(ctx, |ui| {
                let before = theta_deg;
                ui.add(
                    egui::Slider::new(&mut theta_deg, 5.0..=40.0).text("ramp angle (degrees)"),
                );
                if (theta_deg - before).abs() > f32::EPSILON {
                    let q = Quat::from_axis_angle(Vec3::Z, theta_deg * PI / 180.0);
                    world.set_rotation(ramp, q);
                    ramp_node.set_rotation(q);
                }
                ui.separator();
                if ui.button("Block to top").clicked() {
                    reset_requested = true;
                }
                ui.small("Tilt the ramp - the block slides harder.");
            });

            egui::Window::new("Accelerometer").show(ctx, |ui| {
                ui.label(format!(
                    "measured a: {:.2} m/s²",
                    a_measured.abs()
                ));
                ui.label(format!(
                    "theory g·sin(θ): {:.2} m/s²",
                    9.81 * (theta_deg * PI / 180.0).sin()
                ));
                ui.label(format!("current speed: {:.2} m/s", world.speed(block)));
                ui.small("Smoothing makes the meter settle on the slope.");
            });

            egui::Window::new("Speed over time").show(ctx, |ui| {
                let t1 = world.time().max(10.0);
                let y_max = (speed_chart.max_value() * 1.15).max(5.0);
                speed_chart.draw(ui, t1, y_max);
                ui.label("the slope of this line IS the acceleration");
            });
        });

        if reset_requested {
            world.set_translation(block, block_spawn(theta_deg * PI / 180.0));
            world.set_linvel(block, Vec3::ZERO);
            world.set_rotation(block, Quat::IDENTITY);
            world.reset_clock();
            speed_chart.clear();
            prev_speed = 0.0;
            a_measured = 0.0;
            reset_requested = false;
        }
    }
}
