// PhysicsPlay Lab - Experiment 4: Bounce decay (restitution).
//
// A ball is dropped onto the floor. Every touch with the floor is a collision
// event; the peak height of each flight shrinks by a factor of e^2 (e =
// restitution). The height chart shows the classic decaying bounce envelope,
// and the readouts list the measured peak heights and their ratios.
//
// Run with the `egui` feature:
//   cargo run --features egui --example bounce

#[cfg(feature = "egui")]
mod lab;

#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled: cargo run --features egui --example bounce");
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    use kiss3d::prelude::*;
    use physicsplay_core::{BodyKind, Entity, Material, Shape, World, WorldEvent, WorldParams};

    env_logger::init();
    let mut window = Window::new("PhysicsPlay Lab: Bounce decay").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(6.0, 5.0, 12.0), Vec3::new(0.0, 4.0, 0.0));
    let mut scene = SceneNode3d::empty();

    scene
        .add_light(Light::point(120.0).with_intensity(5.0))
        .set_position(Vec3::new(4.0, 12.0, 8.0));
    window.set_background_color(Color::new(0.02, 0.02, 0.04, 1.0));

    const FIXED_DT: f32 = 1.0 / 60.0;
    const BALL_RADIUS: f32 = 0.4;
    const DROP_HEIGHT: f32 = 8.0;
    const SPAWN: Vec3 = Vec3::new(0.0, DROP_HEIGHT, 0.0);

    let mut world = World::new(WorldParams::default());

    let ground_id = world.spawn(Entity {
        translation: Vec3::new(0.0, -0.5, 0.0),
        shape: Shape::Cuboid {
            half_extents: Vec3::new(20.0, 0.5, 20.0),
        },
        kind: BodyKind::Fixed,
        material: Material {
            friction: 0.2,
            restitution: 0.7,
            ..Material::default()
        },
        ..Entity::default()
    });
    scene
        .add_cube(40.0, 1.0, 40.0)
        .set_position(Vec3::new(0.0, -0.5, 0.0))
        .set_color(GRAY);

    let ball = world.spawn(Entity {
        translation: SPAWN,
        shape: Shape::Sphere {
            radius: BALL_RADIUS,
        },
        kind: BodyKind::Dynamic,
        material: Material {
            friction: 0.2,
            restitution: 0.7,
            ..Material::default()
        },
        ..Entity::default()
    });
    let mut ball_node = scene
        .add_sphere(BALL_RADIUS)
        .set_position(SPAWN)
        .set_color(RED);

    // --- Lab state ------------------------------------------------------------
    let mut restitution = 0.7f32;
    let mut bounce_count = 0u32;
    let mut flight_peak = DROP_HEIGHT;
    let mut peaks: Vec<f32> = Vec::new();
    let mut height_chart =
        lab::instruments::StripChart::new("ball height", egui::Color32::from_rgb(240, 110, 110));
    let mut reset_requested = false;

    // --- Fixed-timestep simulation + render loop ------------------------------
    let mut accumulator = 0.0f32;
    let mut last = web_time::Instant::now();

    while window.render_3d(&mut scene, &mut camera).await {
        let now = web_time::Instant::now();
        accumulator += (now - last).as_secs_f32();
        last = now;
        accumulator = accumulator.min(0.25);

        while accumulator >= FIXED_DT {
            world.step(FIXED_DT);
            let h = world.translation(ball).y;
            flight_peak = flight_peak.max(h);
            height_chart.push(world.time(), h);
            accumulator -= FIXED_DT;
        }

        ball_node.set_position(world.translation(ball));
        ball_node.set_rotation(world.rotation(ball));

        for event in world.drain_events() {
            if let WorldEvent::TouchStarted { a, b } = event {
                if (a == ground_id && b == ball) || (a == ball && b == ground_id) {
                    bounce_count += 1;
                    peaks.push(flight_peak);
                    flight_peak = world.translation(ball).y;
                }
            }
        }

        window.draw_ui(|ctx| {
            egui::Window::new("World laws").show(ctx, |ui| {
                ui.add(
                    egui::Slider::new(&mut restitution, 0.1..=1.0)
                        .text("restitution (bounciness)"),
                );
                world.set_restitution(ball, restitution);
                world.set_restitution(ground_id, restitution);
                ui.label(format!("bounce count: {}", bounce_count));
                ui.separator();
                if ui.button("Drop again").clicked() {
                    reset_requested = true;
                }
                ui.small("Each flight peak should shrink by e².");
            });

            egui::Window::new("Bounce readouts").show(ctx, |ui| {
                ui.label(format!("current height: {:.2} m", world.translation(ball).y));
                if let Some(peak) = peaks.last() {
                    ui.label(format!("last flight peak: {:.2} m", peak));
                }
                if peaks.len() >= 2 {
                    let ratio = peaks[peaks.len() - 1] / peaks[peaks.len() - 2];
                    ui.label(format!(
                        "peak ratio: {:.2}  (theory e² = {:.2})",
                        ratio,
                        restitution * restitution
                    ));
                }
                if !peaks.is_empty() {
                    ui.label("flight peaks:");
                    for (i, p) in peaks.iter().rev().take(4).enumerate() {
                        ui.label(format!("  #{}: {:.2} m", peaks.len() - i, p));
                    }
                }
                if bounce_count > 3 && peaks.len() >= 2 {
                    let last = *peaks.last().unwrap();
                    if last < 0.08 {
                        ui.colored_label(
                            egui::Color32::from_rgb(120, 230, 120),
                            "Bounces faded out - kinetic energy became heat and sound.",
                        );
                    }
                }
            });

            egui::Window::new("Height over time").show(ctx, |ui| {
                let t1 = world.time().max(10.0);
                let y_max = DROP_HEIGHT * 1.1;
                height_chart.draw(ui, t1, y_max);
                ui.label("the envelope decays by e² per bounce");
            });
        });

        if reset_requested {
            world.set_translation(ball, SPAWN);
            world.set_linvel(ball, Vec3::ZERO);
            world.reset_clock();
            height_chart.clear();
            bounce_count = 0;
            peaks.clear();
            flight_peak = DROP_HEIGHT;
            reset_requested = false;
        }
    }
}
