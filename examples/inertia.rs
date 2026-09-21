// PhysicsPlay Lab - Experiment 2: Inertia (F = ma).
//
// Two cubes with different masses sit on ice (almost no friction). Pushing
// both with the same force makes the light one accelerate faster: same push,
// different response. The speed chart shows two ramps with different slopes,
// then two plateaus - the accelerations are the slopes.
//
// Run with the `egui` feature:
//   cargo run --features egui --example inertia

#[cfg(feature = "egui")]
mod lab;

#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled: cargo run --features egui --example inertia");
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    use kiss3d::prelude::*;
    use physicsplay_core::{BodyKind, Entity, Material, Shape, World, WorldParams};

    env_logger::init();
    let mut window = Window::new("PhysicsPlay Lab: Inertia (F = ma)").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(0.0, 6.0, 16.0), Vec3::new(0.0, 1.0, 0.0));
    let mut scene = SceneNode3d::empty();

    const FIXED_DT: f32 = 1.0 / 60.0;

    scene
        .add_light(Light::point(120.0).with_intensity(5.0))
        .set_position(Vec3::new(0.0, 12.0, 8.0));
    window.set_background_color(Color::new(0.02, 0.02, 0.04, 1.0));

    let mut world = World::new(WorldParams {
        gravity: Vec3::new(0.0, -9.81, 0.0),
        ..WorldParams::default()
    });

    // Ice rink: almost frictionless ground.
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

    const CUBE_HALF: f32 = 0.5;
    const SPAWN_A: Vec3 = Vec3::new(-3.0, CUBE_HALF, 0.0);
    const SPAWN_B: Vec3 = Vec3::new(3.0, CUBE_HALF, 0.0);

    let mut density_a = 1.0f32; // blue cube, light
    let mut density_b = 4.0f32; // red cube, heavy

    let spawn_cube = |world: &mut World, x: f32, density: f32| -> usize {
        let entity = Entity {
            translation: Vec3::new(x, CUBE_HALF, 0.0),
            shape: Shape::Cuboid {
                half_extents: Vec3::splat(CUBE_HALF),
            },
            kind: BodyKind::Dynamic,
            material: Material {
                friction: 0.01,
                restitution: 0.0,
                density,
            },
            ..Entity::default()
        };
        world.spawn(entity)
    };
    let cube_a = spawn_cube(&mut world, SPAWN_A.x, density_a);
    let cube_b = spawn_cube(&mut world, SPAWN_B.x, density_b);

    let mut objects: Vec<(usize, SceneNode3d)> = Vec::new();
    objects.push((
        cube_a,
        scene
            .add_cube(CUBE_HALF * 2.0, CUBE_HALF * 2.0, CUBE_HALF * 2.0)
            .set_position(SPAWN_A)
            .set_color(BLUE),
    ));
    objects.push((
        cube_b,
        scene
            .add_cube(CUBE_HALF * 2.0, CUBE_HALF * 2.0, CUBE_HALF * 2.0)
            .set_position(SPAWN_B)
            .set_color(RED),
    ));

    // --- Lab state ------------------------------------------------------------
    let mut force_n = 10.0f32;
    let mut push_ticks = 0u32;
    let mut chart_a = lab::instruments::StripChart::new("blue", egui::Color32::from_rgb(90, 170, 255));
    let mut chart_b = lab::instruments::StripChart::new("red", egui::Color32::from_rgb(240, 110, 110));
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
            if push_ticks > 0 {
                world.set_force(cube_a, Vec3::new(force_n, 0.0, 0.0));
                world.set_force(cube_b, Vec3::new(force_n, 0.0, 0.0));
                push_ticks -= 1;
                if push_ticks == 0 {
                    world.clear_force(cube_a);
                    world.clear_force(cube_b);
                }
            }
            world.step(FIXED_DT);
            let t = world.time();
            chart_a.push(t, world.speed(cube_a));
            chart_b.push(t, world.speed(cube_b));
            accumulator -= FIXED_DT;
        }

        for (id, node) in &mut objects {
            node.set_position(world.translation(*id));
            node.set_rotation(world.rotation(*id));
        }

        window.draw_ui(|ctx| {
            egui::Window::new("World laws").show(ctx, |ui| {
                ui.add(egui::Slider::new(&mut force_n, 1.0..=50.0).text("push force (N)"));
                ui.add(
                    egui::Slider::new(&mut density_a, 0.5..=10.0)
                        .text("blue cube density (kg/m³)"),
                );
                world.set_density(cube_a, density_a);
                ui.add(
                    egui::Slider::new(&mut density_b, 0.5..=10.0)
                        .text("red cube density (kg/m³)"),
                );
                world.set_density(cube_b, density_b);
                ui.separator();
                if ui.button("Push both (1 s)").clicked() {
                    push_ticks = 60;
                }
                if ui.button("Reset").clicked() {
                    reset_requested = true;
                }
                ui.small("Same force - different response?");
            });

            egui::Window::new("Cube readouts").show(ctx, |ui| {
                egui::Grid::new("readouts").show(ui, |ui| {
                    ui.strong("quantity");
                    ui.strong("blue (light)");
                    ui.strong("red (heavy)");
                    ui.end_row();

                    ui.label("mass (kg)");
                    ui.label(format!("{:.2}", world.mass(cube_a)));
                    ui.label(format!("{:.2}", world.mass(cube_b)));
                    ui.end_row();

                    ui.label("speed (m/s)");
                    ui.label(format!("{:.2}", world.speed(cube_a)));
                    ui.label(format!("{:.2}", world.speed(cube_b)));
                    ui.end_row();

                    ui.label("distance from start (m)");
                    let da = (world.translation(cube_a) - SPAWN_A).length();
                    let db = (world.translation(cube_b) - SPAWN_B).length();
                    ui.label(format!("{:.2}", da));
                    ui.label(format!("{:.2}", db));
                    ui.end_row();
                });
                if push_ticks > 0 {
                    ui.label(format!("pushing... {:.1} N", force_n));
                } else {
                    ui.label("Coasting. Watch the plateau: that speed came from F = ma.");
                }
            });

            egui::Window::new("Speed over time").show(ctx, |ui| {
                let t1 = world.time().max(10.0);
                let y_max = (chart_a.max_value().max(chart_b.max_value()) * 1.15).max(5.0);
                chart_a.draw(ui, t1, y_max);
                chart_b.draw(ui, t1, y_max);

                ui.horizontal(|ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(90, 170, 255),
                        format!("blue: {:.1} kg", world.mass(cube_a)),
                    );
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 120, 120),
                        format!("red: {:.1} kg", world.mass(cube_b)),
                    );
                    ui.label(format!("t = {:.1} s", world.time()));
                });
            });
        });

        if reset_requested {
            world.clear_force(cube_a);
            world.clear_force(cube_b);
            world.set_translation(cube_a, SPAWN_A);
            world.set_linvel(cube_a, Vec3::ZERO);
            world.set_translation(cube_b, SPAWN_B);
            world.set_linvel(cube_b, Vec3::ZERO);
            world.reset_clock();
            chart_a.clear();
            chart_b.clear();
            push_ticks = 0;
            reset_requested = false;
        }
    }
}
