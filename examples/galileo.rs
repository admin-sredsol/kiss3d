// PhysicsPlay Lab - Experiment 1: Galileo's Drop.
//
// Two balls with different masses fall from the same height. The meters and
// the speed-over-time chart make the law visible: mass does not change how
// fast things fall. Students vary gravity and the two densities with sliders,
// re-drop, and watch the two curves stay on top of each other.
//
// Run with the `egui` feature:
//   cargo run --features egui --example galileo

#[cfg(feature = "egui")]
mod lab;

#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled: cargo run --features egui --example galileo");
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    use kiss3d::prelude::*;
    use physicsplay_core::{BodyKind, Entity, Material, Shape, World, WorldEvent, WorldParams};

    env_logger::init();

    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PhysicsPlay panic: {info}");
        web_sys::console::error_1(&msg.clone().into());
        // Surface the panic inside the page so it is visible without devtools.
        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Ok(pre) = document.create_element("pre") {
                    pre.set_text_content(Some(&msg));
                    if let Some(body) = document.body() {
                        let _ = body.append_child(&pre);
                    }
                }
            }
        }
    }));
    let mut window = Window::new("PhysicsPlay Lab: Galileo's Drop").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(8.0, 6.0, 14.0), Vec3::new(0.0, 4.0, 0.0));
    let mut scene = SceneNode3d::empty();

    const FIXED_DT: f32 = 1.0 / 60.0;

    scene
        .add_light(Light::point(120.0).with_intensity(5.0))
        .set_position(Vec3::new(4.0, 12.0, 8.0));
    window.set_background_color(Color::new(0.02, 0.02, 0.04, 1.0));

    // --- World ---------------------------------------------------------------
    let mut world = World::new(WorldParams {
        gravity: Vec3::new(0.0, -9.81, 0.0),
        ..WorldParams::default()
    });

    let ground_id = world.spawn(Entity {
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

    const BALL_RADIUS: f32 = 0.5;
    const DROP_HEIGHT: f32 = 10.0;
    const SPAWN_A: Vec3 = Vec3::new(-2.0, DROP_HEIGHT, 0.0);
    const SPAWN_B: Vec3 = Vec3::new(2.0, DROP_HEIGHT, 0.0);

    let mut density_a = 1.0f32; // blue ball, light
    let mut density_b = 8.0f32; // red ball, heavy

    let spawn_ball =
        |world: &mut World, x: f32, density: f32| -> usize {
            let entity = Entity {
                translation: Vec3::new(x, DROP_HEIGHT, 0.0),
                shape: Shape::Sphere {
                    radius: BALL_RADIUS,
                },
                kind: BodyKind::Dynamic,
                material: Material {
                    friction: 0.2,
                    restitution: 0.0,
                    density,
                },
                ..Entity::default()
            };
            world.spawn(entity)
        };
    let ball_a = spawn_ball(&mut world, SPAWN_A.x, density_a);
    let ball_b = spawn_ball(&mut world, SPAWN_B.x, density_b);

    let mut objects: Vec<(usize, SceneNode3d)> = Vec::new();
    objects.push((
        ball_a,
        scene
            .add_sphere(BALL_RADIUS)
            .set_position(SPAWN_A)
            .set_color(BLUE),
    ));
    objects.push((
        ball_b,
        scene
            .add_sphere(BALL_RADIUS)
            .set_position(SPAWN_B)
            .set_color(RED),
    ));

    // --- Lab state ------------------------------------------------------------
    let mut gravity_g = 9.81f32;
    let mut chart_a = lab::instruments::StripChart::new("blue", egui::Color32::from_rgb(90, 170, 255));
    let mut chart_b = lab::instruments::StripChart::new("red", egui::Color32::from_rgb(240, 110, 110));
    let mut fall_a: Option<f32> = None;
    let mut fall_b: Option<f32> = None;
    let mut reset_requested = false;
    #[cfg(not(target_arch = "wasm32"))]
    let mut save_requested = false;
    #[cfg(not(target_arch = "wasm32"))]
    let mut load_requested = false;
    #[cfg(not(target_arch = "wasm32"))]
    let mut io_status: Option<String> = None;
    let mut save_requested = false;
    let mut load_requested = false;
    let mut io_status: Option<String> = None;

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
            let sa = world.speed(ball_a);
            let sb = world.speed(ball_b);
            chart_a.push(t, sa);
            chart_b.push(t, sb);
            accumulator -= FIXED_DT;
        }

        for (id, node) in &mut objects {
            node.set_position(world.translation(*id));
            node.set_rotation(world.rotation(*id));
        }

        for event in world.drain_events() {
            if let WorldEvent::TouchStarted { a, b } = event {
                if a == ground_id || b == ground_id {
                    let other = if a == ground_id { b } else { a };
                    if other == ball_a && fall_a.is_none() {
                        fall_a = Some(world.time());
                    }
                    if other == ball_b && fall_b.is_none() {
                        fall_b = Some(world.time());
                    }
                }
            }
        }

        // --- Instruments (egui) ----------------------------------------------
        window.draw_ui(|ctx| {
            egui::Window::new("World laws").show(ctx, |ui| {
                ui.add(
                    egui::Slider::new(&mut gravity_g, 0.5..=25.0).text("gravity (m/s²)"),
                );
                world.params_mut().gravity = Vec3::new(0.0, -gravity_g, 0.0);
                ui.add(
                    egui::Slider::new(&mut density_a, 0.2..=20.0)
                        .text("blue ball density (kg/m³)"),
                );
                world.set_density(ball_a, density_a);
                ui.add(
                    egui::Slider::new(&mut density_b, 0.2..=20.0)
                        .text("red ball density (kg/m³)"),
                );
                world.set_density(ball_b, density_b);
                ui.separator();
                if ui.button("Drop again").clicked() {
                    reset_requested = true;
                }
                #[cfg(not(target_arch = "wasm32"))]
                ui.horizontal(|ui| {
                    if ui.button("Save world").clicked() {
                        save_requested = true;
                    }
                    if ui.button("Load world").clicked() {
                        load_requested = true;
                    }
                });
                if let Some(status) = &io_status {
                    ui.small(status);
                }
                ui.small("Different masses - same fall?");
            });

            egui::Window::new("Ball readouts").show(ctx, |ui| {
                egui::Grid::new("readouts").show(ui, |ui| {
                    ui.strong("quantity");
                    ui.strong("blue (light)");
                    ui.strong("red (heavy)");
                    ui.end_row();

                    ui.label("height (m)");
                    ui.label(format!("{:.2}", world.translation(ball_a).y));
                    ui.label(format!("{:.2}", world.translation(ball_b).y));
                    ui.end_row();

                    ui.label("speed (m/s)");
                    ui.label(format!("{:.2}", world.speed(ball_a)));
                    ui.label(format!("{:.2}", world.speed(ball_b)));
                    ui.end_row();

                    ui.label("mass (kg)");
                    ui.label(format!("{:.2}", world.mass(ball_a)));
                    ui.label(format!("{:.2}", world.mass(ball_b)));
                    ui.end_row();

                    ui.label("fall time (s)");
                    ui.label(fall_text(fall_a));
                    ui.label(fall_text(fall_b));
                    ui.end_row();
                });
                if let (Some(ta), Some(tb)) = (fall_a, fall_b) {
                    if (ta - tb).abs() < 0.05 {
                        ui.colored_label(
                            egui::Color32::from_rgb(120, 230, 120),
                            format!("Both landed in {:.2} s - different mass, same fall.", ta),
                        );
                    } else {
                        ui.colored_label(
                            egui::Color32::from_rgb(230, 180, 90),
                            format!("Fall times differ: {:.2} s vs {:.2} s.", ta, tb),
                        );
                    }
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
                        format!("blue: {:.1} kg", world.mass(ball_a)),
                    );
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 120, 120),
                        format!("red: {:.1} kg", world.mass(ball_b)),
                    );
                    ui.label(format!("t = {:.1} s", world.time()));
                });
            });
        });

        if reset_requested {
            world.set_translation(ball_a, SPAWN_A);
            world.set_linvel(ball_a, Vec3::ZERO);
            world.set_translation(ball_b, SPAWN_B);
            world.set_linvel(ball_b, Vec3::ZERO);
            world.reset_clock();
            chart_a.clear();
            chart_b.clear();
            fall_a = None;
            fall_b = None;
            reset_requested = false;
        }

        #[cfg(not(target_arch = "wasm32"))]
        if save_requested {
            io_status = Some(match world.save_to_file(std::path::Path::new("worlds/galileo.json"))
            {
                Ok(()) => "saved to worlds/galileo.json".to_string(),
                Err(e) => format!("save failed: {e}"),
            });
            save_requested = false;
        }

        #[cfg(not(target_arch = "wasm32"))]
        if load_requested {
            io_status = Some(match World::load_from_file(std::path::Path::new("worlds/galileo.json"))
            {
                Ok(loaded) => {
                    world = loaded;
                    "loaded from worlds/galileo.json".to_string()
                }
                Err(e) => format!("load failed: {e}"),
            });
            load_requested = false;
        }
    }
}

#[cfg(feature = "egui")]
fn fall_text(fall: Option<f32>) -> String {
    match fall {
        Some(t) => format!("{:.2}", t),
        None => "falling...".to_string(),
    }
}
