// PhysicsPlay - the programmable experiment.
//
// The lab runs a block program that the student can EDIT in the UI: change
// the trigger, the action and the quantities (all real: newtons, kg*m/s,
// meters per second), add or delete rules, then press "Run" to restart the
// world with the edited program. Save/Load round-trips world + program.
//
// Run with the `egui` feature:
//   cargo run --features egui --example blocks_demo

#[cfg(feature = "egui")]
mod lab;

#[cfg(feature = "egui")]
use physicsplay_core::{BlockProgram, BlockRuntime, Rule, Target, Trigger, World};

#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled: cargo run --features egui --example blocks_demo");
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    use kiss3d::prelude::*;

    env_logger::init();
    let mut window = Window::new("PhysicsPlay: Block program").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(0.0, 6.0, 14.0), Vec3::new(-1.0, 1.0, 0.0));
    let mut scene = SceneNode3d::empty();

    scene
        .add_light(Light::point(120.0).with_intensity(5.0))
        .set_position(Vec3::new(-2.0, 12.0, 8.0));
    window.set_background_color(Color::new(0.02, 0.02, 0.04, 1.0));

    const FIXED_DT: f32 = 1.0 / 60.0;
    const BALL_RADIUS: f32 = 0.5;

    // Canonical world setup for this experiment.
    let setup = lab::worlds::first_game();
    let mut world = setup.world;
    let player = setup.player;
    let target = setup.target;
    let ground_id = setup.ground;
    let spawn_player = setup.spawn_player;
    let spawn_target = setup.spawn_target;

    scene
        .add_cube(60.0, 1.0, 20.0)
        .set_position(Vec3::new(0.0, -0.5, 0.0))
        .set_color(GRAY);

    let mut objects: Vec<(usize, SceneNode3d)> = Vec::new();
    objects.push((
        player,
        scene
            .add_sphere(BALL_RADIUS)
            .set_position(spawn_player)
            .set_color(BLUE),
    ));
    objects.push((
        target,
        scene
            .add_sphere(BALL_RADIUS)
            .set_position(spawn_target)
            .set_color(RED),
    ));

    // Names the editor shows for entities.
    let labels: Vec<(usize, String)> = vec![
        (player, "blue ball".to_string()),
        (target, "red ball".to_string()),
        (ground_id, "ground".to_string()),
    ];

    // --- The starter program -------------------------------------------------
    let mut program = BlockProgram {
        rules: vec![
            Rule {
                trigger: Trigger::OnStart,
                action: physicsplay_core::Action::ApplyImpulse {
                    entity: Target::Id(player),
                    impulse: Vec3::new(4.0, 0.0, 0.0),
                    point: None,
                },
            },
            Rule {
                trigger: Trigger::Touches {
                    a: Target::Id(player),
                    b: Some(Target::Id(target)),
                },
                action: physicsplay_core::Action::Say {
                    message: "I WIN!".to_string(),
                },
            },
        ],
    };
    world.set_program(program.clone());
    let mut runtime = BlockRuntime::new(program.clone());

    let mut won: Option<String> = None;
    let mut run_requested = false;
    let mut add_state = lab::program_editor::AddRuleState::default();

    // --- Fixed-timestep simulation + render loop ------------------------------
    let mut accumulator = 0.0f32;
    let mut last = web_time::Instant::now();

    while window.render_3d(&mut scene, &mut camera).await {
        let now = web_time::Instant::now();
        accumulator += (now - last).as_secs_f32();
        last = now;
        accumulator = accumulator.min(0.25);

        while accumulator >= FIXED_DT {
            runtime.start(&mut world);
            world.step(FIXED_DT);
            let events = world.drain_events();
            runtime.feed_events(&mut world, &events);
            for message in runtime.take_said() {
                won = Some(message);
            }
            accumulator -= FIXED_DT;
        }

        for (id, node) in &mut objects {
            node.set_position(world.translation(*id));
            node.set_rotation(world.rotation(*id));
        }

        window.draw_ui(|ctx| {
            egui::Window::new("Block program").show(ctx, |ui| {
                ui.label("Edit the rules, then press Run. Every number is a real quantity.");
                ui.add_space(4.0);

                let mut delete_index: Option<usize> = None;
                for index in 0..program.rules.len() {
                    if lab::program_editor::rule_card(ui, index, &mut program.rules[index], &labels)
                    {
                        delete_index = Some(index);
                    }
                    ui.add_space(2.0);
                }
                if let Some(i) = delete_index {
                    program.rules.remove(i);
                }

                ui.separator();
                lab::program_editor::add_rule_section(ui, &mut program, &labels, &mut add_state);

                ui.separator();
                if ui.button("Run (restart world with this program)").clicked() {
                    run_requested = true;
                }
                ui.horizontal(|ui| {
                    if ui.button("Save world").clicked() {
                        world
                            .save_to_file(std::path::Path::new("worlds/blocks-demo.json"))
                            .ok();
                    }
                    if ui.button("Load world").clicked() {
                        if let Ok(loaded) =
                            World::load_from_file(std::path::Path::new("worlds/blocks-demo.json"))
                        {
                            program = loaded.program().cloned().unwrap_or_default();
                            world = loaded;
                            world.set_program(program.clone());
                            runtime = BlockRuntime::new(program.clone());
                            won = None;
                        }
                    }
                });
                ui.small("Edits apply on the next Run.");
            });

            if let Some(message) = &won {
                egui::Window::new("Said").show(ctx, |ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(120, 230, 120),
                        egui::RichText::new(message).size(28.0).strong(),
                    );
                });
            }

            egui::Window::new("Readouts").show(ctx, |ui| {
                ui.label(format!("player speed: {:.2} m/s", world.speed(player)));
                ui.label(format!(
                    "distance to target: {:.2} m",
                    (world.translation(player) - world.translation(target)).length()
                ));
            });
        });

        if run_requested {
            // Fresh start: entities back to their origins, program re-armed.
            world.set_translation(player, spawn_player);
            world.set_linvel(player, Vec3::ZERO);
            world.set_translation(target, spawn_target);
            world.set_linvel(target, Vec3::ZERO);
            world.reset_clock();
            world.set_program(program.clone());
            runtime = BlockRuntime::new(program.clone());
            won = None;
            run_requested = false;
        }
    }
}

// --- Rule editor widgets -------------------------------------------------------
