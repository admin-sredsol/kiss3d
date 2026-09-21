// PhysicsPlay Lab - Experiment 3: Friction.
//
// Three identical cubes are launched with the same impulse along three lanes
// with different friction. They slide, decelerate and stop at different
// distances: stopping distance ~ v^2 / (2 * mu * g). The chart shows the speed
// decay per lane.
//
// Run with the `egui` feature:
//   cargo run --features egui --example friction

#[cfg(feature = "egui")]
mod lab;

#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled: cargo run --features egui --example friction");
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    use kiss3d::prelude::*;
    use physicsplay_core::{BodyKind, Entity, Material, Shape, World, WorldParams};

    env_logger::init();
    let mut window = Window::new("PhysicsPlay Lab: Friction").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(-6.0, 16.0, 26.0), Vec3::new(4.0, 0.0, 0.0));
    let mut scene = SceneNode3d::empty();

    scene
        .add_light(Light::point(150.0).with_intensity(5.0))
        .set_position(Vec3::new(6.0, 18.0, 10.0));
    window.set_background_color(Color::new(0.02, 0.02, 0.04, 1.0));

    const FIXED_DT: f32 = 1.0 / 60.0;
    const CUBE_HALF: f32 = 0.5;
    const START_X: f32 = -15.0;

    // Lanes: z offset, friction, kiss3d color, egui color, label.
    const LANES: [(f32, f32, [f32; 3], [u8; 3], &str); 3] = [
        (-4.0, 0.1, [0.25, 0.55, 0.95], [90, 170, 255], "blue"),
        (0.0, 0.5, [0.95, 0.35, 0.35], [240, 110, 110], "red"),
        (4.0, 1.0, [0.35, 0.9, 0.4], [120, 230, 120], "green"),
    ];

    let mut world = World::new(WorldParams {
        gravity: Vec3::new(0.0, -9.81, 0.0),
        ..WorldParams::default()
    });

    // Three lane strips (fixed) with different friction.
    for (z, friction, _, _, _) in LANES {
        let _strip = world.spawn(Entity {
            translation: Vec3::new(0.0, -0.5, z),
            shape: Shape::Cuboid {
                half_extents: Vec3::new(30.0, 0.5, 2.0),
            },
            kind: BodyKind::Fixed,
            material: Material {
                friction,
                restitution: 0.0,
                ..Material::default()
            },
            ..Entity::default()
        });
        scene
            .add_cube(60.0, 1.0, 4.0)
            .set_position(Vec3::new(0.0, -0.5, z))
            .set_color(Color::new(0.30, 0.32, 0.36, 1.0));
    }

    // One cube per lane, matching the strip friction (average combine rule).
    let mut cubes = Vec::new();
    let mut origins = Vec::new();
    let mut objects: Vec<(usize, SceneNode3d)> = Vec::new();
    for (z, friction, kcolor, _, _) in LANES {
        let origin = Vec3::new(START_X, CUBE_HALF, z);
        let id = world.spawn(Entity {
            translation: origin,
            shape: Shape::Cuboid {
                half_extents: Vec3::splat(CUBE_HALF),
            },
            kind: BodyKind::Dynamic,
            material: Material {
                friction,
                restitution: 0.0,
                ..Material::default()
            },
            ..Entity::default()
        });
        cubes.push(id);
        origins.push(origin);
        let [r, g, b] = kcolor;
        objects.push((
            id,
            scene
                .add_cube(CUBE_HALF * 2.0, CUBE_HALF * 2.0, CUBE_HALF * 2.0)
                .set_position(origin)
                .set_color(Color::new(r, g, b, 1.0)),
        ));
    }

    // --- Lab state ------------------------------------------------------------
    let mut impulse_n = 8.0f32;
    let mut charts: Vec<lab::instruments::StripChart> = Vec::new();
    for (i, (_, _, _, ecolor, name)) in LANES.iter().enumerate() {
        let _ = i;
        let [r, g, b] = *ecolor;
        charts.push(lab::instruments::StripChart::new(
            format!("{} (mu = {:.1})", name, LANES[i].1),
            egui::Color32::from_rgb(r, g, b),
        ));
    }
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
            let t = world.time();
            for (lane, &id) in cubes.iter().enumerate() {
                charts[lane].push(t, world.speed(id));
            }
            accumulator -= FIXED_DT;
        }

        for (id, node) in &mut objects {
            node.set_position(world.translation(*id));
            node.set_rotation(world.rotation(*id));
        }

        window.draw_ui(|ctx| {
            egui::Window::new("World laws").show(ctx, |ui| {
                ui.add(
                    egui::Slider::new(&mut impulse_n, 2.0..=15.0)
                        .text("launch impulse (kg·m/s)"),
                );
                ui.separator();
                if ui.button("Launch all").clicked() {
                    for &id in &cubes {
                        world.apply_impulse(id, Vec3::new(impulse_n, 0.0, 0.0));
                    }
                }
                if ui.button("Reset").clicked() {
                    reset_requested = true;
                }
                ui.small("Same launch - which lane stops first, and why?");
            });

            egui::Window::new("Lane readouts").show(ctx, |ui| {
                egui::Grid::new("readouts").show(ui, |ui| {
                    ui.strong("lane");
                    ui.strong("friction (mu)");
                    ui.strong("speed (m/s)");
                    ui.strong("distance (m)");
                    ui.end_row();

                    for (lane, &id) in cubes.iter().enumerate() {
                        let (_, friction, _, ecolor, _) = LANES[lane];
                        let [r, g, b] = ecolor;
                        let color = egui::Color32::from_rgb(r, g, b);
                        ui.colored_label(color, LANES[lane].4);
                        ui.label(format!("{:.1}", friction));
                        ui.label(format!("{:.2}", world.speed(id)));
                        let d = (world.translation(id) - origins[lane]).length();
                        ui.label(format!("{:.2}", d));
                        ui.end_row();
                    }
                });
                ui.small("Stopping distance ~ v0^2 / (2 * mu * g)");
            });

            egui::Window::new("Speed over time").show(ctx, |ui| {
                let t1 = world.time().max(10.0);
                let mut y_max: f32 = 5.0;
                for chart in &charts {
                    y_max = y_max.max(chart.max_value());
                }
                let y_max = y_max * 1.15;
                for chart in &charts {
                    chart.draw(ui, t1, y_max);
                }
                ui.horizontal(|ui| {
                    for (lane, (_, _, _, ecolor, name)) in LANES.iter().enumerate() {
                        let [r, g, b] = *ecolor;
                        ui.colored_label(
                            egui::Color32::from_rgb(r, g, b),
                            format!("{}: mu {:.1}", name, LANES[lane].1),
                        );
                    }
                    ui.label(format!("t = {:.1} s", world.time()));
                });
            });
        });

        if reset_requested {
            for (lane, &id) in cubes.iter().enumerate() {
                world.set_translation(id, origins[lane]);
                world.set_linvel(id, Vec3::ZERO);
                world.clear_force(id);
            }
            world.reset_clock();
            for chart in &mut charts {
                chart.clear();
            }
            reset_requested = false;
        }
    }
}
