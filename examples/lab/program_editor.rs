//! egui editor for [`BlockProgram`]s: editable rule cards plus an add-rule
//! section. Every field is a physical quantity.

use physicsplay_core::{
    Action, BlockProgram, BodyKind, Comparison, InputKey, Rule, Sensor, Target, Trigger, Vec3,
};

pub const TRIGGER_NAMES: [&str; 10] = [
    "when started",
    "when [A] touches [B]",
    "when impact above",
    "when [sensor] of [E] [above/below]",
    "every [t] s",
    "when hinge [A]-[B] passes [deg]",
    "when I press [key]",
    "when distance [A]-[B] [above/below] [N] m",
    "after [t] s (once)",
    "when slide [A]-[B] passes [N] m",
];
pub const KEY_NAMES: [&str; 5] = ["space", "up", "down", "left", "right"];
pub const SENSOR_NAMES: [&str; 4] = [
    "speed (m/s)",
    "height (m)",
    "spin (rad/s)",
    "score (points)",
];
pub const COMPARISON_NAMES: [&str; 3] = ["above", "below", "reaches"];
pub const ACTION_NAMES: [&str; 20] = [
    "hit (impulse)",
    "push (force)",
    "stop push",
    "say",
    "add ball",
    "add box",
    "add cylinder",
    "add cone",
    "add pill",
    "score +",
    "hinge",
    "turn hinge to",
    "remove",
    "move to",
    "stop",
    "tilt",
    "add slider",
    "slide to",
    "random hit",
    "random move",
];

#[derive(Default)]
pub struct AddRuleState {
    pub trigger: usize,
    pub action: usize,
}

fn entity_label(id: usize, labels: &[(usize, String)]) -> String {
    labels
        .iter()
        .find(|(eid, _)| *eid == id)
        .map(|(_, l)| l.clone())
        .unwrap_or_else(|| format!("entity {id}"))
}

pub fn entity_combo(ui: &mut egui::Ui, salt: String, id: &mut usize, labels: &[(usize, String)]) {
    egui::ComboBox::from_id_salt(salt)
        .selected_text(entity_label(*id, labels))
        .show_ui(ui, |ui| {
            for (eid, label) in labels {
                ui.selectable_value(id, *eid, label.clone());
            }
        });
}

/// Entity picker that also offers "anything" (None).
pub fn any_entity_combo(
    ui: &mut egui::Ui,
    salt: String,
    id: &mut Option<usize>,
    labels: &[(usize, String)],
) {
    let text = match id {
        Some(eid) => entity_label(*eid, labels),
        None => "anything".to_string(),
    };
    egui::ComboBox::from_id_salt(salt)
        .selected_text(text)
        .show_ui(ui, |ui| {
            ui.selectable_value(id, None, "anything");
            for (eid, label) in labels {
                ui.selectable_value(id, Some(*eid), label.clone());
            }
        });
}

fn drag_vec3(ui: &mut egui::Ui, v: &mut Vec3) {
    ui.horizontal(|ui| {
        ui.add(egui::DragValue::new(&mut v.x).speed(0.1).prefix("x "));
        ui.add(egui::DragValue::new(&mut v.y).speed(0.1).prefix("y "));
        ui.add(egui::DragValue::new(&mut v.z).speed(0.1).prefix("z "));
    });
}

fn kind_combo(ui: &mut egui::Ui, salt: String, kind: &mut BodyKind) {
    let selected = match *kind {
        BodyKind::Dynamic => "dynamic",
        BodyKind::Fixed => "fixed",
    };
    egui::ComboBox::from_id_salt(salt)
        .selected_text(selected)
        .show_ui(ui, |ui| {
            ui.selectable_value(kind, BodyKind::Dynamic, "dynamic");
            ui.selectable_value(kind, BodyKind::Fixed, "fixed");
        });
}

/// Edit one rule in place. Returns true when the delete button was pressed.
pub fn rule_card(
    ui: &mut egui::Ui,
    index: usize,
    rule: &mut Rule,
    labels: &[(usize, String)],
) -> bool {
    let mut delete = false;

    let trigger_kind = match &rule.trigger {
        Trigger::OnStart => 0,
        Trigger::Touches { .. } => 1,
        Trigger::ImpactAbove { .. } => 2,
        Trigger::Condition { .. } => 3,
        Trigger::Every { .. } => 4,
        Trigger::JointAngle { .. } => 5,
        Trigger::PlayerInput { .. } => 6,
        Trigger::Distance { .. } => 7,
        Trigger::After { .. } => 8,
        Trigger::JointOffset { .. } => 9,
    };
    let action_kind = match &rule.action {
        Action::ApplyImpulse { .. } => 0,
        Action::SetForce { .. } => 1,
        Action::ClearForce { .. } => 2,
        Action::Say { .. } => 3,
        Action::AddBall { .. } => 4,
        Action::AddBox { .. } => 5,
        Action::AddScore { .. } => 6,
        Action::Hinge { .. } => 7,
        Action::MotorTo { .. } => 8,
        Action::Remove { .. } => 9,
        Action::MoveTo { .. } => 10,
        Action::Stop { .. } => 11,
        Action::TiltTo { .. } => 12,
        Action::Slider { .. } => 13,
        Action::SlideTo { .. } => 14,
        Action::RandomImpulse { .. } => 15,
        Action::RandomMove { .. } => 16,
        Action::AddCylinder { .. } => 17,
        Action::AddCone { .. } => 18,
        Action::AddCapsule { .. } => 19,
    };

    egui::Frame::new()
        .fill(egui::Color32::from_rgb(34, 36, 44))
        .inner_margin(6.0)
        .show(ui, |ui| {
            // --- trigger ---
            egui::Frame::new()
                .fill(egui::Color32::from_rgb(96, 76, 22))
                .inner_margin(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut kind = trigger_kind;
                        egui::ComboBox::from_id_salt(format!("r{index}-trigger"))
                            .selected_text(TRIGGER_NAMES[trigger_kind])
                            .show_ui(ui, |ui| {
                                for (i, name) in TRIGGER_NAMES.iter().enumerate() {
                                    ui.selectable_value(&mut kind, i, *name);
                                }
                            });
                        match (kind, &mut rule.trigger) {
                            (0, Trigger::OnStart) => {}
                            (0, _) => rule.trigger = Trigger::OnStart,
                            (
                                1,
                                Trigger::Touches {
                                    a: Target::Id(a),
                                    b,
                                },
                            ) => {
                                ui.label("A:");
                                entity_combo(ui, format!("r{index}-ta"), a, labels);
                                ui.label("B:");
                                let mut b_local = b.as_ref().and_then(|t| match t {
                                    Target::Id(i) => Some(*i),
                                    _ => None,
                                });
                                any_entity_combo(ui, format!("r{index}-tb"), &mut b_local, labels);
                                *b = b_local.map(Target::Id);
                            }
                            (1, _) => {
                                rule.trigger = Trigger::Touches {
                                    a: Target::Id(labels[0].0),
                                    b: Some(Target::Id(labels[1].0)),
                                };
                            }
                            (2, Trigger::ImpactAbove { entity, threshold }) => {
                                ui.label("entity:");
                                let mut entity_local = entity.as_ref().and_then(|t| match t {
                                    Target::Id(i) => Some(*i),
                                    _ => None,
                                });
                                any_entity_combo(
                                    ui,
                                    format!("r{index}-ie"),
                                    &mut entity_local,
                                    labels,
                                );
                                *entity = entity_local.map(Target::Id);
                                ui.label("above (N):");
                                ui.add(
                                    egui::DragValue::new(threshold)
                                        .speed(1.0)
                                        .range(0.0..=1.0e6),
                                );
                            }
                            (2, _) => {
                                rule.trigger = Trigger::ImpactAbove {
                                    entity: None,
                                    threshold: 50.0,
                                };
                            }
                            (
                                3,
                                Trigger::Condition {
                                    entity,
                                    sensor,
                                    comparison,
                                    threshold,
                                },
                            ) => {
                                ui.label("when");
                                let sensor_index = match sensor {
                                    Sensor::Speed => 0,
                                    Sensor::Height => 1,
                                    Sensor::Spin => 2,
                                    Sensor::Score => 3,
                                };
                                egui::ComboBox::from_id_salt(format!("r{index}-cs"))
                                    .selected_text(SENSOR_NAMES[sensor_index])
                                    .show_ui(ui, |ui| {
                                        for (i, name) in SENSOR_NAMES.iter().enumerate() {
                                            let selected = sensor_index == i;
                                            if ui.selectable_label(selected, *name).clicked() {
                                                *sensor = match i {
                                                    0 => Sensor::Speed,
                                                    1 => Sensor::Height,
                                                    2 => Sensor::Spin,
                                                    _ => Sensor::Score,
                                                };
                                            }
                                        }
                                    });
                                ui.label("of");
                                let mut id_local = entity
                                    .as_ref()
                                    .and_then(|t| match t {
                                        Target::Id(i) => Some(*i),
                                        _ => None,
                                    })
                                    .unwrap_or(labels[0].0);
                                entity_combo(ui, format!("r{index}-ce"), &mut id_local, labels);
                                *entity = Some(Target::Id(id_local));
                                let comparison_index = match comparison {
                                    Comparison::Above => 0,
                                    Comparison::Below => 1,
                                    Comparison::Reaches => 2,
                                };
                                egui::ComboBox::from_id_salt(format!("r{index}-cc"))
                                    .selected_text(COMPARISON_NAMES[comparison_index])
                                    .show_ui(ui, |ui| {
                                        for (i, name) in COMPARISON_NAMES.iter().enumerate() {
                                            let selected = comparison_index == i;
                                            if ui.selectable_label(selected, *name).clicked() {
                                                *comparison = match i {
                                                    0 => Comparison::Above,
                                                    1 => Comparison::Below,
                                                    _ => Comparison::Reaches,
                                                };
                                            }
                                        }
                                    });
                                ui.add(
                                    egui::DragValue::new(threshold)
                                        .speed(0.1)
                                        .range(-1000.0..=1000.0),
                                );
                            }
                            (3, _) => {
                                rule.trigger = Trigger::Condition {
                                    entity: Some(Target::Id(labels[0].0)),
                                    sensor: Sensor::Speed,
                                    comparison: Comparison::Above,
                                    threshold: 1.0,
                                };
                            }
                            (4, Trigger::Every { seconds }) => {
                                ui.label("every");
                                ui.add(
                                    egui::DragValue::new(seconds)
                                        .speed(0.1)
                                        .range(0.05..=600.0)
                                        .suffix(" s"),
                                );
                            }
                            (4, _) => {
                                rule.trigger = Trigger::Every { seconds: 1.0 };
                            }
                            (
                                5,
                                Trigger::JointAngle {
                                    a: Target::Id(a),
                                    b: Target::Id(b),
                                    comparison,
                                    threshold_deg,
                                },
                            ) => {
                                ui.label("when hinge");
                                entity_combo(ui, format!("r{index}-ja"), a, labels);
                                entity_combo(ui, format!("r{index}-jb"), b, labels);
                                ui.label("passes");
                                let comparison_index = match comparison {
                                    Comparison::Above => 0,
                                    Comparison::Below => 1,
                                    Comparison::Reaches => 2,
                                };
                                egui::ComboBox::from_id_salt(format!("r{index}-jc"))
                                    .selected_text(COMPARISON_NAMES[comparison_index])
                                    .show_ui(ui, |ui| {
                                        for (i, name) in COMPARISON_NAMES.iter().enumerate() {
                                            let selected = comparison_index == i;
                                            if ui.selectable_label(selected, *name).clicked() {
                                                *comparison = match i {
                                                    0 => Comparison::Above,
                                                    1 => Comparison::Below,
                                                    _ => Comparison::Reaches,
                                                };
                                            }
                                        }
                                    });
                                ui.add(
                                    egui::DragValue::new(threshold_deg)
                                        .speed(1.0)
                                        .range(-360.0..=360.0)
                                        .suffix("°"),
                                );
                            }
                            (5, _) => {
                                rule.trigger = Trigger::JointAngle {
                                    a: Target::Id(labels[0].0),
                                    b: Target::Id(labels[1].0),
                                    comparison: Comparison::Above,
                                    threshold_deg: 30.0,
                                };
                            }
                            (6, Trigger::PlayerInput { key }) => {
                                let key_index = match key {
                                    InputKey::Space => 0,
                                    InputKey::Up => 1,
                                    InputKey::Down => 2,
                                    InputKey::Left => 3,
                                    InputKey::Right => 4,
                                };
                                egui::ComboBox::from_id_salt(format!("r{index}-pk"))
                                    .selected_text(KEY_NAMES[key_index])
                                    .show_ui(ui, |ui| {
                                        for (i, name) in KEY_NAMES.iter().enumerate() {
                                            let selected = key_index == i;
                                            if ui.selectable_label(selected, *name).clicked() {
                                                *key = match i {
                                                    0 => InputKey::Space,
                                                    1 => InputKey::Up,
                                                    2 => InputKey::Down,
                                                    3 => InputKey::Left,
                                                    _ => InputKey::Right,
                                                };
                                            }
                                        }
                                    });
                            }
                            (6, _) => {
                                rule.trigger = Trigger::PlayerInput {
                                    key: InputKey::Space,
                                };
                            }
                            (
                                7,
                                Trigger::Distance {
                                    a: Target::Id(a),
                                    b: Target::Id(b),
                                    comparison,
                                    meters,
                                },
                            ) => {
                                ui.label("when distance between");
                                entity_combo(ui, format!("r{index}-da"), a, labels);
                                entity_combo(ui, format!("r{index}-db"), b, labels);
                                let comparison_index = match comparison {
                                    Comparison::Above => 0,
                                    Comparison::Below => 1,
                                    Comparison::Reaches => 2,
                                };
                                egui::ComboBox::from_id_salt(format!("r{index}-dc"))
                                    .selected_text(COMPARISON_NAMES[comparison_index])
                                    .show_ui(ui, |ui| {
                                        for (i, name) in COMPARISON_NAMES.iter().enumerate() {
                                            let selected = comparison_index == i;
                                            if ui.selectable_label(selected, *name).clicked() {
                                                *comparison = match i {
                                                    0 => Comparison::Above,
                                                    1 => Comparison::Below,
                                                    _ => Comparison::Reaches,
                                                };
                                            }
                                        }
                                    });
                                ui.add(
                                    egui::DragValue::new(meters)
                                        .speed(0.1)
                                        .range(0.0..=1000.0)
                                        .suffix(" m"),
                                );
                            }
                            (7, _) => {
                                rule.trigger = Trigger::Distance {
                                    a: Target::Id(labels[0].0),
                                    b: Target::Id(labels[1].0),
                                    comparison: Comparison::Below,
                                    meters: 2.0,
                                };
                            }
                            (8, Trigger::After { seconds }) => {
                                ui.label("after");
                                ui.add(
                                    egui::DragValue::new(seconds)
                                        .speed(0.1)
                                        .range(0.05..=600.0)
                                        .suffix(" s (once)"),
                                );
                            }
                            (8, _) => {
                                rule.trigger = Trigger::After { seconds: 3.0 };
                            }
                            (
                                9,
                                Trigger::JointOffset {
                                    a: Target::Id(a),
                                    b: Target::Id(b),
                                    comparison,
                                    meters,
                                },
                            ) => {
                                ui.label("when slide");
                                entity_combo(ui, format!("r{index}-sa"), a, labels);
                                entity_combo(ui, format!("r{index}-sb"), b, labels);
                                ui.label("passes");
                                let comparison_index = match comparison {
                                    Comparison::Above => 0,
                                    Comparison::Below => 1,
                                    Comparison::Reaches => 2,
                                };
                                egui::ComboBox::from_id_salt(format!("r{index}-sc"))
                                    .selected_text(COMPARISON_NAMES[comparison_index])
                                    .show_ui(ui, |ui| {
                                        for (i, name) in COMPARISON_NAMES.iter().enumerate() {
                                            let selected = comparison_index == i;
                                            if ui.selectable_label(selected, *name).clicked() {
                                                *comparison = match i {
                                                    0 => Comparison::Above,
                                                    1 => Comparison::Below,
                                                    _ => Comparison::Reaches,
                                                };
                                            }
                                        }
                                    });
                                ui.add(
                                    egui::DragValue::new(meters)
                                        .speed(0.1)
                                        .range(-1000.0..=1000.0)
                                        .suffix(" m"),
                                );
                            }
                            (9, _) => {
                                rule.trigger = Trigger::JointOffset {
                                    a: Target::Id(labels[0].0),
                                    b: Target::Id(labels[1].0),
                                    comparison: Comparison::Above,
                                    meters: 1.0,
                                };
                            }
                            _ => {}
                        }
                    });
                });

            // --- action ---
            egui::Frame::new()
                .fill(match action_kind {
                    3 => egui::Color32::from_rgb(30, 90, 40),
                    2 => egui::Color32::from_rgb(60, 60, 30),
                    _ => egui::Color32::from_rgb(20, 60, 90),
                })
                .inner_margin(4.0)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut kind = action_kind;
                        egui::ComboBox::from_id_salt(format!("r{index}-action"))
                            .selected_text(ACTION_NAMES[action_kind])
                            .show_ui(ui, |ui| {
                                for (i, name) in ACTION_NAMES.iter().enumerate() {
                                    ui.selectable_value(&mut kind, i, *name);
                                }
                            });
                        match (kind, &mut rule.action) {
                            (
                                0,
                                Action::ApplyImpulse {
                                    entity: Target::Id(entity),
                                    impulse,
                                    point,
                                },
                            ) => {
                                ui.label("on");
                                entity_combo(ui, format!("r{index}-ae"), entity, labels);
                                drag_vec3(ui, impulse);
                                if let Some(point) = point {
                                    ui.label("at point");
                                    drag_vec3(ui, point);
                                }
                            }
                            (0, _) => {
                                rule.action = Action::ApplyImpulse {
                                    entity: Target::Id(labels[0].0),
                                    impulse: Vec3::new(4.0, 0.0, 0.0),
                                    point: None,
                                };
                            }
                            (
                                1,
                                Action::SetForce {
                                    entity: Target::Id(entity),
                                    force,
                                    point,
                                },
                            ) => {
                                ui.label("on");
                                entity_combo(ui, format!("r{index}-fe"), entity, labels);
                                drag_vec3(ui, force);
                                if let Some(point) = point {
                                    ui.label("at point");
                                    drag_vec3(ui, point);
                                }
                            }
                            (1, _) => {
                                rule.action = Action::SetForce {
                                    entity: Target::Id(labels[0].0),
                                    force: Vec3::new(10.0, 0.0, 0.0),
                                    point: None,
                                };
                            }
                            (
                                2,
                                Action::ClearForce {
                                    entity: Target::Id(entity),
                                },
                            ) => {
                                ui.label("on");
                                entity_combo(ui, format!("r{index}-ce"), entity, labels);
                            }
                            (2, _) => {
                                rule.action = Action::ClearForce {
                                    entity: Target::Id(labels[0].0),
                                };
                            }
                            (3, Action::Say { message }) => {
                                ui.text_edit_singleline(message);
                            }
                            (3, _) => {
                                rule.action = Action::Say {
                                    message: "message".to_string(),
                                };
                            }
                            (
                                4,
                                Action::AddBall {
                                    label,
                                    radius,
                                    density,
                                    kind,
                                    position,

                                    friction: None,
                                    restitution: None,
                                },
                            ) => {
                                ui.text_edit_singleline(label);
                                ui.add(
                                    egui::DragValue::new(radius)
                                        .speed(0.05)
                                        .range(0.01..=10.0)
                                        .prefix("r "),
                                );
                                ui.add(
                                    egui::DragValue::new(density)
                                        .speed(0.1)
                                        .range(0.01..=100.0)
                                        .prefix("ρ "),
                                );
                                kind_combo(ui, format!("r{index}-abk"), kind);
                                drag_vec3(ui, position);
                            }
                            (4, _) => {
                                rule.action = Action::AddBall {
                                    label: "ball A".to_string(),
                                    radius: 0.5,
                                    density: 1.0,
                                    kind: BodyKind::Dynamic,
                                    position: Vec3::new(0.0, 2.0, 0.0),

                                    friction: None,
                                    restitution: None,
                                };
                            }
                            (
                                5,
                                Action::AddBox {
                                    label,
                                    half_extents,
                                    density,
                                    kind,
                                    position,

                                    friction: None,
                                    restitution: None,
                                },
                            ) => {
                                ui.text_edit_singleline(label);
                                ui.add(
                                    egui::DragValue::new(&mut half_extents.x)
                                        .speed(0.1)
                                        .range(0.01..=100.0)
                                        .prefix("x "),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut half_extents.y)
                                        .speed(0.1)
                                        .range(0.01..=100.0)
                                        .prefix("y "),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut half_extents.z)
                                        .speed(0.1)
                                        .range(0.01..=100.0)
                                        .prefix("z "),
                                );
                                ui.add(
                                    egui::DragValue::new(density)
                                        .speed(0.1)
                                        .range(0.01..=100.0)
                                        .prefix("ρ "),
                                );
                                kind_combo(ui, format!("r{index}-axk"), kind);
                                drag_vec3(ui, position);
                            }
                            (5, _) => {
                                rule.action = Action::AddBox {
                                    label: "box A".to_string(),
                                    half_extents: Vec3::new(2.0, 0.5, 2.0),
                                    density: 1.0,
                                    kind: BodyKind::Fixed,
                                    position: Vec3::new(0.0, -0.5, 0.0),

                                    friction: None,
                                    restitution: None,
                                };
                            }
                            (6, Action::AddScore { amount }) => {
                                ui.label("score");
                                ui.add(
                                    egui::DragValue::new(amount)
                                        .speed(0.1)
                                        .range(-1000.0..=1000.0),
                                );
                            }
                            (6, _) => {
                                rule.action = Action::AddScore { amount: 1.0 };
                            }
                            (
                                7,
                                Action::Hinge {
                                    a: Target::Id(a),
                                    b: Target::Id(b),
                                    point,
                                    axis,

                                    limits_deg: None,
                                },
                            ) => {
                                ui.label("hinge");
                                entity_combo(ui, format!("r{index}-ha"), a, labels);
                                ui.label("to");
                                entity_combo(ui, format!("r{index}-hb"), b, labels);
                                ui.label("at point");
                                drag_vec3(ui, point);
                                ui.label("around axis");
                                drag_vec3(ui, axis);
                            }
                            (7, _) => {
                                rule.action = Action::Hinge {
                                    a: Target::Id(labels[0].0),
                                    b: Target::Id(labels[1].0),
                                    point: Vec3::new(0.0, 6.0, 0.0),
                                    axis: Vec3::new(0.0, 0.0, 1.0),

                                    limits_deg: None,
                                };
                            }
                            (
                                8,
                                Action::MotorTo {
                                    a: Target::Id(a),
                                    b: Target::Id(b),
                                    target_deg,
                                    stiffness,
                                    damping,
                                },
                            ) => {
                                ui.label("turn hinge");
                                entity_combo(ui, format!("r{index}-ma"), a, labels);
                                entity_combo(ui, format!("r{index}-mb"), b, labels);
                                ui.label("to");
                                ui.add(
                                    egui::DragValue::new(target_deg)
                                        .speed(1.0)
                                        .range(-360.0..=360.0)
                                        .suffix("°"),
                                );
                                ui.add(
                                    egui::DragValue::new(stiffness)
                                        .speed(1.0)
                                        .range(1.0..=10000.0)
                                        .prefix("stiffness "),
                                );
                                ui.add(
                                    egui::DragValue::new(damping)
                                        .speed(1.0)
                                        .range(0.0..=1000.0)
                                        .prefix("damping "),
                                );
                            }
                            (8, _) => {
                                rule.action = Action::MotorTo {
                                    a: Target::Id(labels[0].0),
                                    b: Target::Id(labels[1].0),
                                    target_deg: 45.0,
                                    stiffness: 100.0,
                                    damping: 20.0,
                                };
                            }
                            (
                                9,
                                Action::Remove {
                                    entity: Target::Id(entity),
                                },
                            ) => {
                                ui.label("remove");
                                entity_combo(ui, format!("r{index}-re"), entity, labels);
                            }
                            (9, _) => {
                                rule.action = Action::Remove {
                                    entity: Target::Id(labels[0].0),
                                };
                            }
                            (
                                10,
                                Action::MoveTo {
                                    entity: Target::Id(entity),
                                    position,
                                },
                            ) => {
                                ui.label("move");
                                entity_combo(ui, format!("r{index}-me"), entity, labels);
                                ui.label("to");
                                drag_vec3(ui, position);
                            }
                            (10, _) => {
                                rule.action = Action::MoveTo {
                                    entity: Target::Id(labels[0].0),
                                    position: Vec3::new(0.0, 5.0, 0.0),
                                };
                            }
                            (
                                11,
                                Action::Stop {
                                    entity: Target::Id(entity),
                                },
                            ) => {
                                ui.label("stop");
                                entity_combo(ui, format!("r{index}-se"), entity, labels);
                            }
                            (11, _) => {
                                rule.action = Action::Stop {
                                    entity: Target::Id(labels[0].0),
                                };
                            }
                            (
                                12,
                                Action::TiltTo {
                                    entity: Target::Id(entity),
                                    deg,
                                    axis,
                                },
                            ) => {
                                ui.label("tilt");
                                entity_combo(ui, format!("r{index}-te"), entity, labels);
                                ui.add(
                                    egui::DragValue::new(deg)
                                        .speed(1.0)
                                        .range(-360.0..=360.0)
                                        .suffix("°"),
                                );
                                ui.label("around axis");
                                drag_vec3(ui, axis);
                            }
                            (12, _) => {
                                rule.action = Action::TiltTo {
                                    entity: Target::Id(labels[0].0),
                                    deg: 15.0,
                                    axis: Vec3::new(0.0, 0.0, 1.0),
                                };
                            }
                            (
                                13,
                                Action::Slider {
                                    a: Target::Id(a),
                                    b: Target::Id(b),
                                    axis,
                                },
                            ) => {
                                ui.label("slide");
                                entity_combo(ui, format!("r{index}-la"), a, labels);
                                ui.label("along");
                                entity_combo(ui, format!("r{index}-lb"), b, labels);
                                ui.label("around axis");
                                drag_vec3(ui, axis);
                            }
                            (13, _) => {
                                rule.action = Action::Slider {
                                    a: Target::Id(labels[0].0),
                                    b: Target::Id(labels[1].0),
                                    axis: Vec3::new(1.0, 0.0, 0.0),
                                };
                            }
                            (
                                14,
                                Action::SlideTo {
                                    a: Target::Id(a),
                                    b: Target::Id(b),
                                    meters,
                                    stiffness,
                                    damping,
                                },
                            ) => {
                                ui.label("slide");
                                entity_combo(ui, format!("r{index}-ta"), a, labels);
                                entity_combo(ui, format!("r{index}-tb"), b, labels);
                                ui.label("to");
                                ui.add(
                                    egui::DragValue::new(meters)
                                        .speed(0.1)
                                        .range(-1000.0..=1000.0)
                                        .suffix(" m"),
                                );
                                ui.add(
                                    egui::DragValue::new(stiffness)
                                        .speed(1.0)
                                        .range(1.0..=10000.0)
                                        .prefix("stiffness "),
                                );
                                ui.add(
                                    egui::DragValue::new(damping)
                                        .speed(1.0)
                                        .range(0.0..=1000.0)
                                        .prefix("damping "),
                                );
                            }
                            (14, _) => {
                                rule.action = Action::SlideTo {
                                    a: Target::Id(labels[0].0),
                                    b: Target::Id(labels[1].0),
                                    meters: 2.0,
                                    stiffness: 100.0,
                                    damping: 20.0,
                                };
                            }
                            (
                                15,
                                Action::RandomImpulse {
                                    entity: Target::Id(entity),
                                    max,
                                },
                            ) => {
                                ui.label("random hit");
                                entity_combo(ui, format!("r{index}-ri"), entity, labels);
                                ui.label("up to");
                                drag_vec3(ui, max);
                            }
                            (15, _) => {
                                rule.action = Action::RandomImpulse {
                                    entity: Target::Id(labels[0].0),
                                    max: Vec3::new(2.0, 2.0, 0.0),
                                };
                            }
                            (
                                16,
                                Action::RandomMove {
                                    entity: Target::Id(entity),
                                    min,
                                    max,
                                },
                            ) => {
                                ui.label("random move");
                                entity_combo(ui, format!("r{index}-rm"), entity, labels);
                                ui.label("between");
                                drag_vec3(ui, min);
                                ui.label("and");
                                drag_vec3(ui, max);
                            }
                            (16, _) => {
                                rule.action = Action::RandomMove {
                                    entity: Target::Id(labels[0].0),
                                    min: Vec3::new(-5.0, 5.0, 0.0),
                                    max: Vec3::new(5.0, 10.0, 0.0),
                                };
                            }
                            (
                                17,
                                Action::AddCylinder {
                                    label,
                                    radius,
                                    height,
                                    density,
                                    kind,
                                    position,
                                    friction,
                                    restitution,
                                },
                            ) => {
                                ui.text_edit_singleline(label);
                                ui.add(
                                    egui::DragValue::new(radius)
                                        .speed(0.05)
                                        .range(0.01..=10.0)
                                        .prefix("r "),
                                );
                                ui.add(
                                    egui::DragValue::new(height)
                                        .speed(0.05)
                                        .range(0.02..=20.0)
                                        .prefix("h "),
                                );
                                ui.add(
                                    egui::DragValue::new(density)
                                        .speed(0.1)
                                        .range(0.01..=100.0)
                                        .prefix("ρ "),
                                );
                                kind_combo(ui, format!("r{index}-ck"), kind);
                                drag_vec3(ui, position);
                                if let Some(f) = friction {
                                    ui.add(
                                        egui::DragValue::new(f)
                                            .speed(0.05)
                                            .range(0.0..=2.0)
                                            .prefix("friction "),
                                    );
                                }
                                if let Some(r) = restitution {
                                    ui.add(
                                        egui::DragValue::new(r)
                                            .speed(0.05)
                                            .range(0.0..=1.0)
                                            .prefix("bounce "),
                                    );
                                }
                            }
                            (17, _) => {
                                rule.action = Action::AddCylinder {
                                    label: "drum A".to_string(),
                                    radius: 0.5,
                                    height: 1.0,
                                    density: 1.0,
                                    kind: BodyKind::Dynamic,
                                    position: Vec3::new(0.0, 2.0, 0.0),
                                    friction: None,
                                    restitution: None,
                                };
                            }
                            (
                                18,
                                Action::AddCone {
                                    label,
                                    radius,
                                    height,
                                    density,
                                    kind,
                                    position,
                                    friction: _,
                                    restitution: _,
                                },
                            )
                            | (
                                19,
                                Action::AddCapsule {
                                    label,
                                    radius,
                                    height,
                                    density,
                                    kind,
                                    position,
                                    friction: _,
                                    restitution: _,
                                },
                            ) => {
                                ui.text_edit_singleline(label);
                                ui.add(
                                    egui::DragValue::new(radius)
                                        .speed(0.05)
                                        .range(0.01..=10.0)
                                        .prefix("r "),
                                );
                                ui.add(
                                    egui::DragValue::new(height)
                                        .speed(0.05)
                                        .range(0.02..=20.0)
                                        .prefix("h "),
                                );
                                ui.add(
                                    egui::DragValue::new(density)
                                        .speed(0.1)
                                        .range(0.01..=100.0)
                                        .prefix("ρ "),
                                );
                                kind_combo(ui, format!("r{index}-nk"), kind);
                                drag_vec3(ui, position);
                            }
                            (18, _) => {
                                rule.action = Action::AddCone {
                                    label: "cone A".to_string(),
                                    radius: 0.5,
                                    height: 1.5,
                                    density: 1.0,
                                    kind: BodyKind::Dynamic,
                                    position: Vec3::new(0.0, 2.0, 0.0),
                                    friction: None,
                                    restitution: None,
                                };
                            }
                            (19, _) => {
                                rule.action = Action::AddCapsule {
                                    label: "pill A".to_string(),
                                    radius: 0.5,
                                    height: 2.0,
                                    density: 1.0,
                                    kind: BodyKind::Dynamic,
                                    position: Vec3::new(0.0, 2.0, 0.0),
                                    friction: None,
                                    restitution: None,
                                };
                            }
                            _ => {}
                        }
                    });
                });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("delete").clicked() {
                    delete = true;
                }
            });
        });

    delete
}

/// Append a default rule of the picked trigger/action kinds.
pub fn add_rule_section(
    ui: &mut egui::Ui,
    program: &mut BlockProgram,
    labels: &[(usize, String)],
    state: &mut AddRuleState,
) {
    ui.horizontal(|ui| {
        egui::ComboBox::from_id_salt("add-trigger")
            .selected_text(TRIGGER_NAMES[state.trigger])
            .show_ui(ui, |ui| {
                for (i, name) in TRIGGER_NAMES.iter().enumerate() {
                    ui.selectable_value(&mut state.trigger, i, *name);
                }
            });
        egui::ComboBox::from_id_salt("add-action")
            .selected_text(ACTION_NAMES[state.action])
            .show_ui(ui, |ui| {
                for (i, name) in ACTION_NAMES.iter().enumerate() {
                    ui.selectable_value(&mut state.action, i, *name);
                }
            });
        if ui.button("add rule").clicked() {
            let trigger = match state.trigger {
                0 => Trigger::OnStart,
                1 => Trigger::Touches {
                    a: Target::Id(labels[0].0),
                    b: Some(Target::Id(labels[1].0)),
                },
                2 => Trigger::ImpactAbove {
                    entity: None,
                    threshold: 50.0,
                },
                3 => Trigger::Condition {
                    entity: Some(Target::Id(labels[0].0)),
                    sensor: Sensor::Speed,
                    comparison: Comparison::Above,
                    threshold: 1.0,
                },
                4 => Trigger::Every { seconds: 1.0 },
                5 => Trigger::JointAngle {
                    a: Target::Id(labels[0].0),
                    b: Target::Id(labels[1].0),
                    comparison: Comparison::Above,
                    threshold_deg: 30.0,
                },
                6 => Trigger::PlayerInput {
                    key: InputKey::Space,
                },
                7 => Trigger::Distance {
                    a: Target::Id(labels[0].0),
                    b: Target::Id(labels[1].0),
                    comparison: Comparison::Below,
                    meters: 2.0,
                },
                8 => Trigger::After { seconds: 3.0 },
                9 => Trigger::JointOffset {
                    a: Target::Id(labels[0].0),
                    b: Target::Id(labels[1].0),
                    comparison: Comparison::Above,
                    meters: 1.0,
                },
                _ => Trigger::JointOffset {
                    a: Target::Id(labels[0].0),
                    b: Target::Id(labels[1].0),
                    comparison: Comparison::Above,
                    meters: 1.0,
                },
            };
            let action = match state.action {
                0 => Action::ApplyImpulse {
                    entity: Target::Id(labels[0].0),
                    impulse: Vec3::new(4.0, 0.0, 0.0),
                    point: None,
                },
                1 => Action::SetForce {
                    entity: Target::Id(labels[0].0),
                    force: Vec3::new(10.0, 0.0, 0.0),
                    point: None,
                },
                2 => Action::ClearForce {
                    entity: Target::Id(labels[0].0),
                },
                3 => Action::Say {
                    message: "message".to_string(),
                },
                4 => Action::AddBall {
                    label: "ball A".to_string(),
                    radius: 0.5,
                    density: 1.0,
                    kind: BodyKind::Dynamic,
                    position: Vec3::new(0.0, 2.0, 0.0),

                    friction: None,
                    restitution: None,
                },
                5 => Action::AddBox {
                    label: "box A".to_string(),
                    half_extents: Vec3::new(2.0, 0.5, 2.0),
                    density: 1.0,
                    kind: BodyKind::Fixed,
                    position: Vec3::new(0.0, -0.5, 0.0),

                    friction: None,
                    restitution: None,
                },
                6 => Action::AddScore { amount: 1.0 },
                7 => Action::Hinge {
                    a: Target::Id(labels[0].0),
                    b: Target::Id(labels[1].0),
                    point: Vec3::new(0.0, 6.0, 0.0),
                    axis: Vec3::new(0.0, 0.0, 1.0),

                    limits_deg: None,
                },
                8 => Action::MotorTo {
                    a: Target::Id(labels[0].0),
                    b: Target::Id(labels[1].0),
                    target_deg: 45.0,
                    stiffness: 100.0,
                    damping: 20.0,
                },
                9 => Action::Remove {
                    entity: Target::Id(labels[0].0),
                },
                10 => Action::MoveTo {
                    entity: Target::Id(labels[0].0),
                    position: Vec3::new(0.0, 5.0, 0.0),
                },
                11 => Action::Stop {
                    entity: Target::Id(labels[0].0),
                },
                12 => Action::TiltTo {
                    entity: Target::Id(labels[0].0),
                    deg: 15.0,
                    axis: Vec3::new(0.0, 0.0, 1.0),
                },
                13 => Action::Slider {
                    a: Target::Id(labels[0].0),
                    b: Target::Id(labels[1].0),
                    axis: Vec3::new(1.0, 0.0, 0.0),
                },
                14 => Action::SlideTo {
                    a: Target::Id(labels[0].0),
                    b: Target::Id(labels[1].0),
                    meters: 2.0,
                    stiffness: 100.0,
                    damping: 20.0,
                },
                15 => Action::RandomImpulse {
                    entity: Target::Id(labels[0].0),
                    max: Vec3::new(2.0, 2.0, 0.0),
                },
                16 => Action::RandomMove {
                    entity: Target::Id(labels[0].0),
                    min: Vec3::new(-5.0, 5.0, 0.0),
                    max: Vec3::new(5.0, 10.0, 0.0),
                },
                _ => Action::AddCylinder {
                    label: "drum A".to_string(),
                    radius: 0.5,
                    height: 1.0,
                    density: 1.0,
                    kind: BodyKind::Dynamic,
                    position: Vec3::new(0.0, 2.0, 0.0),
                    friction: None,
                    restitution: None,
                },
            };
            // Cone (18) and pill (19) in the add-rule section:
            let action = match state.action {
                18 => Action::AddCone {
                    label: "cone A".to_string(),
                    radius: 0.5,
                    height: 1.5,
                    density: 1.0,
                    kind: BodyKind::Dynamic,
                    position: Vec3::new(0.0, 2.0, 0.0),
                    friction: None,
                    restitution: None,
                },
                19 => Action::AddCapsule {
                    label: "pill A".to_string(),
                    radius: 0.5,
                    height: 2.0,
                    density: 1.0,
                    kind: BodyKind::Dynamic,
                    position: Vec3::new(0.0, 2.0, 0.0),
                    friction: None,
                    restitution: None,
                },
                _ => action,
            };
            program.rules.push(Rule { trigger, action });
        }
    });
}
