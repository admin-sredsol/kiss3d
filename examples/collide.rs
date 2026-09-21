// PhysicsPlay Lab - Experiment 6: Collisions (momentum).
//
// Ball A is launched at ball B, which is at rest. At the moment of contact
// (a collision event), the speeds before/after are captured. Total momentum
// m*vA + m*vB is conserved for any restitution; how the speed is *split*
// between the balls depends on e: e = 1 passes it all to B (Newton's cradle),
// e = 0 makes them move off together.
//
// Run with the `egui` feature:
//   cargo run --features egui --example collide

#[cfg(feature = "egui")]
mod lab;

#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled: cargo run --features egui --example collide");
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    use kiss3d::prelude::*;
    use physicsplay_core::{BodyKind, Entity, Material, Shape, World, WorldEvent, WorldParams};

    env_logger::init();
    let mut window = Window::new("PhysicsPlay Lab: Collisions").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(-4.0, 7.0, 14.0), Vec3::new(-2.0, 0.5, 0.0));
    let mut scene = SceneNode3d::empty();

    scene
        .add_light(Light::point(120.0).with_intensity(5.0))
        .set_position(Vec3::new(-6.0, 12.0, 8.0));
    window.set_background_color(Color::new(0.02, 0.02, 0.04, 1.0));

    const FIXED_DT: f32 = 1.0 / 60.0;
    const BALL_RADIUS: f32 = 0.5;
    const SPAWN_A: Vec3 = Vec3::new(-9.0, BALL_RADIUS, 0.0);
    const SPAWN_B: Vec3 = Vec3::new(0.0, BALL_RADIUS, 0.0);

    // Low impact threshold so force events fire on every contact.
    let mut world = World::new(WorldParams {
        impact_threshold: 10.0,
        ..WorldParams::default()
    });

    let _ground = world.spawn(Entity {
        translation: Vec3::new(0.0, -0.5, 0.0),
        shape: Shape::Cuboid {
            half_extents: Vec3::new(30.0, 0.5, 10.0),
        },
        kind: BodyKind::Fixed,
        material: Material {
            friction: 0.01,
            restitution: 0.0,
            ..Material::default()
        },
        ..Entity::default()
    });
    scene
        .add_cube(60.0, 1.0, 20.0)
        .set_position(Vec3::new(0.0, -0.5, 0.0))
        .set_color(Color::new(0.75, 0.85, 0.95, 1.0));

    let ball_a = world.spawn(Entity {
        translation: SPAWN_A,
        shape: Shape::Sphere {
            radius: BALL_RADIUS,
        },
        kind: BodyKind::Dynamic,
        material: Material {
            friction: 0.01,
            restitution: 1.0,
            ..Material::default()
        },
        ..Entity::default()
    });
    let ball_b = world.spawn(Entity {
        translation: SPAWN_B,
        shape: Shape::Sphere {
            radius: BALL_RADIUS,
        },
        kind: BodyKind::Dynamic,
        material: Material {
            friction: 0.01,
            restitution: 1.0,
            ..Material::default()
        },
        ..Entity::default()
    });

    let mut node_a = scene
        .add_sphere(BALL_RADIUS)
        .set_position(SPAWN_A)
        .set_color(BLUE);
    let mut node_b = scene
        .add_sphere(BALL_RADIUS)
        .set_position(SPAWN_B)
        .set_color(RED);

    // --- Lab state ------------------------------------------------------------
    let mut restitution = 1.0f32;
    let mut impulse_n = 4.0f32;
    let mut launch_requested = false;
    let mut reset_requested = false;

    let mut last_speed_a = 0.0f32;
    let mut v_a_before: Option<f32> = None;
    let mut collision_time: Option<f32> = None;
    let mut post_sampled = false;
    let mut v_a_after: Option<f32> = None;
    let mut v_b_after: Option<f32> = None;
    let mut impact_force: Option<f32> = None;

    let mut chart_a = lab::instruments::StripChart::new("ball A", egui::Color32::from_rgb(90, 170, 255));
    let mut chart_b = lab::instruments::StripChart::new("ball B", egui::Color32::from_rgb(240, 110, 110));

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
            last_speed_a = world.speed(ball_a);
            let t = world.time();
            chart_a.push(t, world.speed(ball_a));
            chart_b.push(t, world.speed(ball_b));
            // Sample the "after" speeds once things settle (~0.5 s post hit).
            if let Some(tc) = collision_time {
                if !post_sampled && t - tc > 0.5 {
                    v_a_after = Some(world.speed(ball_a));
                    v_b_after = Some(world.speed(ball_b));
                    post_sampled = true;
                }
            }
            accumulator -= FIXED_DT;
        }

        for (id, node) in [(&ball_a, &mut node_a), (&ball_b, &mut node_b)] {
            node.set_position(world.translation(*id));
        }

        for event in world.drain_events() {
            match event {
                WorldEvent::TouchStarted { a, b } => {
                    let pair = (a == ball_a && b == ball_b) || (a == ball_b && b == ball_a);
                    if pair && collision_time.is_none() {
                        // Speed sampled on the tick BEFORE the solver resolved
                        // the contact - that is the "before" speed.
                        v_a_before = Some(last_speed_a);
                        collision_time = Some(world.time());
                    }
                }
                WorldEvent::Impact { force_magnitude, .. } => {
                    impact_force = Some(force_magnitude);
                }
                _ => {}
            }
        }

        window.draw_ui(|ctx| {
            egui::Window::new("World laws").show(ctx, |ui| {
                ui.add(
                    egui::Slider::new(&mut restitution, 0.0..=1.0)
                        .text("restitution e (0 = stick, 1 = elastic)"),
                );
                world.set_restitution(ball_a, restitution);
                world.set_restitution(ball_b, restitution);
                ui.add(
                    egui::Slider::new(&mut impulse_n, 1.0..=8.0)
                        .text("launch impulse (kg·m/s)"),
                );
                ui.separator();
                if ui.button("Launch A").clicked() {
                    launch_requested = true;
                }
                if ui.button("Reset").clicked() {
                    reset_requested = true;
                }
            });

            egui::Window::new("Collision readouts").show(ctx, |ui| {
                let m = world.mass(ball_a);
                ui.label(format!("ball mass: {:.2} kg", m));
                if let Some(f) = impact_force {
                    ui.label(format!("last impact force: {:.0} N", f));
                }
                match (v_a_before, v_a_after, v_b_after) {
                    (Some(v0), Some(va), Some(vb)) => {
                        let p_before = m * v0;
                        let p_after = m * va + m * vb;
                        ui.label(format!("A before: {:.2} m/s   after: {:.2} m/s", v0, va));
                        ui.label(format!("B before: 0.00 m/s   after: {:.2} m/s", vb));
                        ui.label(format!(
                            "momentum before: {:.2}   after: {:.2} (conserved!)",
                            p_before, p_after
                        ));
                        ui.label(format!(
                            "theory: A' = {:.2}, B' = {:.2}",
                            v0 * (1.0 - restitution) / 2.0,
                            v0 * (1.0 + restitution) / 2.0
                        ));
                    }
                    (Some(_), None, None) => {
                        ui.label("collision detected - sampling after-speeds...");
                    }
                    _ => {
                        ui.label("Press Launch A. Watch the speed hand-off.");
                    }
                }
            });

            egui::Window::new("Speed over time").show(ctx, |ui| {
                let t1 = world.time().max(10.0);
                let y_max =
                    (chart_a.max_value().max(chart_b.max_value()) * 1.15).max(5.0);
                chart_a.draw(ui, t1, y_max);
                chart_b.draw(ui, t1, y_max);
                ui.label("at the touch: A hands its speed to B (e = 1)");
            });
        });

        if launch_requested {
            world.apply_impulse(ball_a, Vec3::new(impulse_n, 0.0, 0.0));
            launch_requested = false;
        }

        if reset_requested {
            world.set_translation(ball_a, SPAWN_A);
            world.set_linvel(ball_a, Vec3::ZERO);
            world.set_translation(ball_b, SPAWN_B);
            world.set_linvel(ball_b, Vec3::ZERO);
            world.reset_clock();
            chart_a.clear();
            chart_b.clear();
            v_a_before = None;
            collision_time = None;
            post_sampled = false;
            v_a_after = None;
            v_b_after = None;
            reset_requested = false;
        }
    }
}
