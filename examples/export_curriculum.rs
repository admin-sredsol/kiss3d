//! Generates the curriculum world files from the canonical experiment setups.
//!
//! Run: cargo run --example export_curriculum
//! Output: curriculum/worlds/*.json

mod lab;

use physicsplay_core::World;

fn main() {
    let out = std::path::Path::new("curriculum/worlds");
    std::fs::create_dir_all(out).expect("create curriculum dir");

    let worlds: Vec<(&str, World)> = vec![
        (
            "01-galileos-drop",
            lab::worlds::galileo().world,
        ),
        ("02-inertia", lab::worlds::inertia().world),
        ("03-friction-lanes", lab::worlds::friction_lanes().world),
        ("04-bounce-decay", lab::worlds::bounce().world),
        ("05-ramps", lab::worlds::ramp().world),
        ("06-collisions", lab::worlds::collide().world),
        (
            "07-first-block-program",
            lab::worlds::first_game().world,
        ),
    ];

    for (name, world) in &worlds {
        let path = out.join(format!("{name}.json"));
        world.save_to_file(&path).expect("save world");
        println!("wrote {}", path.display());
    }
    println!("curriculum: {} worlds exported", worlds.len());
}
