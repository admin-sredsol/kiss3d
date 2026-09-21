// PhysicsPlay Studio - the full editor.
//
// One window over one world: add and delete objects, tune their material laws,
// edit the block program, run/pause the simulation, and move whole setups in
// and out as JSON text (native file save/load, and on the web copy/paste).
//
// Native: cargo +1.97.1 run --features egui --example studio
// Web:    examples/studio-web (trunk)

#[cfg(feature = "egui")]
mod lab;

#[cfg(feature = "egui")]
use physicsplay_core::{BlockProgram, BlockRuntime, BodyKind, Entity, Target, World, WorldParams};

#[cfg(feature = "egui")]
#[cfg(not(feature = "egui"))]
#[kiss3d::main]
async fn main() {
    panic!("The 'egui' feature must be enabled: cargo run --features egui --example studio");
}

#[cfg(feature = "egui")]
#[kiss3d::main]
async fn main() {
    use kiss3d::prelude::*;

    env_logger::init();
    let mut window = Window::new("PhysicsPlay Studio").await;
    let mut camera = OrbitCamera3d::new(Vec3::new(2.0, 9.0, 18.0), Vec3::new(0.0, 2.0, 0.0));
    let mut scene = SceneNode3d::empty();

    scene
        .add_light(Light::point(150.0).with_intensity(5.0))
        .set_position(Vec3::new(4.0, 16.0, 10.0));
    window.set_background_color(Color::new(0.02, 0.02, 0.04, 1.0));

    const FIXED_DT: f32 = 1.0 / 60.0;
    const GROUND_ID: usize = 0;

    // Ground: big and fixed, always id 0.
    let mut world = World::new(WorldParams::default());
    let _ground = world.spawn(Entity {
        translation: Vec3::new(0.0, -0.5, 0.0),
        shape: physicsplay_core::Shape::Cuboid {
            half_extents: Vec3::new(30.0, 0.5, 20.0),
        },
        kind: BodyKind::Fixed,
        material: physicsplay_core::Material {
            friction: 0.4,
            restitution: 0.0,
            ..physicsplay_core::Material::default()
        },
        ..Entity::default()
    });
    scene
        .add_cube(60.0, 1.0, 40.0)
        .set_position(Vec3::new(0.0, -0.5, 0.0))
        .set_color(GRAY);

    // --- Tracked entities (id -> label -> scene node) -------------------------
    struct Tracked {
        id: usize,
        label: String,
        node: SceneNode3d,
    }

    let palette: [Color; 6] = [BLUE, RED, LIME, YELLOW, CYAN, MAGENTA];
    let mut spawn_counter = 0u32;
    let mut tracked: Vec<Tracked> = Vec::new();
    let mut selected: Option<usize> = None;

    let spawn_entity = |tracked: &mut Vec<Tracked>,
                        world: &mut World,
                        scene: &mut SceneNode3d,
                        spawn_counter: &mut u32,
                        kind: &str| {
        *spawn_counter += 1;
        let n = *spawn_counter;
        let x = ((n as i32 * 7) % 9 - 4) as f32;
        let z = ((n as i32 * 3) % 7 - 3) as f32;
        let shape = if kind == "sphere" {
            physicsplay_core::Shape::Sphere { radius: 0.5 }
        } else {
            physicsplay_core::Shape::Cuboid {
                half_extents: Vec3::splat(0.5),
            }
        };
        let id = world.spawn(Entity {
            translation: Vec3::new(x, 6.0, z),
            shape,
            kind: BodyKind::Dynamic,
            ..Entity::default()
        });
        let label = format!("{kind} {n}");
        let color = palette[n as usize % palette.len()];
        let node = if kind == "sphere" {
            scene
                .add_sphere(0.5)
                .set_position(Vec3::new(x, 6.0, z))
                .set_color(color)
        } else {
            scene
                .add_cube(1.0, 1.0, 1.0)
                .set_position(Vec3::new(x, 6.0, z))
                .set_color(color)
        };
        tracked.push(Tracked { id, label, node });
        id
    };

    // Starter scene: a couple of objects so the world is alive immediately.
    spawn_entity(
        &mut tracked,
        &mut world,
        &mut scene,
        &mut spawn_counter,
        "sphere",
    );
    spawn_entity(
        &mut tracked,
        &mut world,
        &mut scene,
        &mut spawn_counter,
        "cube",
    );

    // --- Block program (editable) ---------------------------------------------
    let mut program = BlockProgram::default();
    // Starter rule so "Run" does something visible immediately: kick the
    // first spawned object when the program starts.
    program.rules.push(physicsplay_core::Rule {
        trigger: physicsplay_core::Trigger::OnStart,
        action: physicsplay_core::Action::ApplyImpulse {
            entity: Target::Id(1), // first spawned object
            impulse: Vec3::new(4.0, 0.0, 0.0),
            point: None,
        },
    });
    world.set_program(program.clone());
    let mut runtime = BlockRuntime::new(program.clone());
    let mut add_state = lab::program_editor::AddRuleState::default();

    // --- Studio state ----------------------------------------------------------
    let mut paused = false;
    let mut gravity_g = 9.81f32;
    let mut won: Option<String> = None;
    let mut run_requested = false;
    let mut world_json = String::new();
    let _speed_chart =
        lab::instruments::StripChart::new("selected speed", egui::Color32::from_rgb(120, 200, 255));
    let _chart_entity: Option<usize> = None;

    // --- Fixed-timestep simulation + render loop -------------------------------
    let mut accumulator = 0.0f32;
    let mut last = web_time::Instant::now();

    while window.render_3d(&mut scene, &mut camera).await {
        let now = web_time::Instant::now();
        accumulator += (now - last).as_secs_f32();
        last = now;
        accumulator = accumulator.min(0.25);

        while !paused && accumulator >= FIXED_DT {
            runtime.start(&mut world);
            world.step(FIXED_DT);
            let events = world.drain_events();
            runtime.feed_events(&mut world, &events);
            for message in runtime.take_said() {
                won = Some(message);
            }
            accumulator -= FIXED_DT;
        }

        for tracked in &mut tracked {
            tracked.node.set_position(world.translation(tracked.id));
            tracked.node.set_rotation(world.rotation(tracked.id));
        }

        // Entity labels for the program editor pickers.
        let labels: Vec<(usize, String)> = std::iter::once((GROUND_ID, "ground".to_string()))
            .chain(tracked.iter().map(|t| (t.id, t.label.clone())))
            .collect();

        window.draw_ui(|ctx| {
            egui::Window::new("World")
                .default_pos(egui::Pos2::new(10.0, 10.0))
                .show(ctx, |ui| {
                    ui.add(
                        egui::Slider::new(&mut gravity_g, 0.0..=30.0).text("world gravity (m/s²)"),
                    );
                    world.params_mut().gravity = Vec3::new(0.0, -gravity_g, 0.0);
                    ui.checkbox(&mut paused, "paused");
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("add sphere").clicked() {
                            spawn_entity(
                                &mut tracked,
                                &mut world,
                                &mut scene,
                                &mut spawn_counter,
                                "sphere",
                            );
                        }
                        if ui.button("add cube").clicked() {
                            spawn_entity(
                                &mut tracked,
                                &mut world,
                                &mut scene,
                                &mut spawn_counter,
                                "cube",
                            );
                        }
                    });

                    // Selected entity + material editors.
                    let options: Vec<(usize, String)> =
                        tracked.iter().map(|t| (t.id, t.label.clone())).collect();
                    egui::ComboBox::from_id_salt("selected-entity")
                        .selected_text(match selected {
                            Some(id) => options
                                .iter()
                                .find(|(eid, _)| *eid == id)
                                .map(|(_, l)| l.clone())
                                .unwrap_or_else(|| format!("entity {id}")),
                            None => "select an object".to_string(),
                        })
                        .show_ui(ui, |ui| {
                            for (eid, label) in &options {
                                ui.selectable_value(&mut selected, Some(*eid), label.clone());
                            }
                        });

                    if let Some(id) = selected {
                        if world.entity_count() > 0 && world.alive_ids().contains(&id) {
                            ui.label("material laws:");
                            let mut mat = world.entity_material(id);
                            if ui
                                .add(
                                    egui::Slider::new(&mut mat.friction, 0.0..=2.0)
                                        .text("friction"),
                                )
                                .changed()
                            {
                                world.set_friction(id, mat.friction);
                            }
                            if ui
                                .add(
                                    egui::Slider::new(&mut mat.restitution, 0.0..=1.0)
                                        .text("bounciness"),
                                )
                                .changed()
                            {
                                world.set_restitution(id, mat.restitution);
                            }
                            if ui
                                .add(
                                    egui::Slider::new(&mut mat.density, 0.1..=20.0)
                                        .text("density (kg/m³)"),
                                )
                                .changed()
                            {
                                world.set_density(id, mat.density);
                            }
                            ui.horizontal(|ui| {
                                if ui.button("lift to 8 m").clicked() {
                                    let p = world.translation(id);
                                    world.set_translation(id, Vec3::new(p.x, 8.0, p.z));
                                    world.set_linvel(id, Vec3::ZERO);
                                }
                                if ui.button("delete").clicked() {
                                    world.remove(id);
                                    if let Some(i) = tracked.iter().position(|t| t.id == id) {
                                        tracked[i].node.remove();
                                        tracked.remove(i);
                                    }
                                    selected = None;
                                }
                            });
                        } else {
                            selected = None;
                        }
                    }
                });

            egui::Window::new("Block program")
                .default_pos(egui::Pos2::new(390.0, 10.0))
                .show(ctx, |ui| {
                    ui.label("Rules fire on world events. Every number is a real quantity.");
                    if ui.button("▶ Run program").clicked() {
                        run_requested = true;
                    }
                    ui.add_space(4.0);

                    let mut delete_index: Option<usize> = None;
                    for index in 0..program.rules.len() {
                        if lab::program_editor::rule_card(
                            ui,
                            index,
                            &mut program.rules[index],
                            &labels,
                        ) {
                            delete_index = Some(index);
                        }
                        ui.add_space(2.0);
                    }
                    if let Some(i) = delete_index {
                        program.rules.remove(i);
                    }
                    ui.separator();
                    lab::program_editor::add_rule_section(
                        ui,
                        &mut program,
                        &labels,
                        &mut add_state,
                    );
                    ui.small("Run re-arms the program (when-start fires again).");
                });

            if let Some(message) = &won {
                egui::Window::new("Said").show(ctx, |ui| {
                    ui.colored_label(
                        egui::Color32::from_rgb(120, 230, 120),
                        egui::RichText::new(message).size(28.0).strong(),
                    );
                });
            }

            egui::Window::new("World data")
                .default_pos(egui::Pos2::new(390.0, 300.0))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button("read world -> text").clicked() {
                            world_json = serde_json::to_string_pretty(&world.snapshot()).unwrap();
                        }
                        if ui.button("apply text -> world").clicked() {
                            if let Ok(snapshot) =
                                serde_json::from_str::<physicsplay_core::WorldSnapshot>(&world_json)
                            {
                                let rebuilt = World::from_snapshot(&snapshot);
                                // Replace every tracked node with the rebuilt world.
                                for t in &mut tracked {
                                    t.node.remove();
                                }
                                tracked.clear();
                                selected = None;
                                won = None;
                                for id in rebuilt.alive_ids() {
                                    let shape = rebuilt.entity_shape(id);
                                    let pos = rebuilt.translation(id);
                                    let color = palette[id % palette.len()];
                                    let label = format!("object {id}");
                                    let node = match shape {
                                        physicsplay_core::Shape::Sphere { radius } => scene
                                            .add_sphere(radius)
                                            .set_position(pos)
                                            .set_color(color),
                                        physicsplay_core::Shape::Cuboid { half_extents } => scene
                                            .add_cube(
                                                half_extents.x * 2.0,
                                                half_extents.y * 2.0,
                                                half_extents.z * 2.0,
                                            )
                                            .set_position(pos)
                                            .set_color(color),
                                        physicsplay_core::Shape::Cylinder {
                                            half_height,
                                            radius,
                                        } => scene
                                            .add_cylinder(radius, half_height * 2.0)
                                            .set_position(pos)
                                            .set_color(color),
                                        physicsplay_core::Shape::Cone {
                                            half_height,
                                            radius,
                                        } => scene
                                            .add_cone(radius, half_height * 2.0)
                                            .set_position(pos)
                                            .set_color(color),
                                        physicsplay_core::Shape::Capsule {
                                            half_height,
                                            radius,
                                        } => scene
                                            .add_capsule(radius, half_height * 2.0)
                                            .set_position(pos)
                                            .set_color(color),
                                    };
                                    tracked.push(Tracked { id, label, node });
                                }
                                world = rebuilt;
                                program = world.program().cloned().unwrap_or_default();
                                runtime = BlockRuntime::new(program.clone());
                            }
                        }
                    });
                    egui::ScrollArea::vertical()
                        .max_height(140.0)
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut world_json)
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(360.0),
                            );
                        });
                    ui.small("Copy this JSON to hand a world out; paste one and apply.");
                });
        });

        if run_requested {
            world.set_program(program.clone());
            runtime = BlockRuntime::new(program.clone());
            won = None;
            run_requested = false;
        }
    }
}
