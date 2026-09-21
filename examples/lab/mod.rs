//! Shared lab code for PhysicsPlay experiments.
//!
//! Compiled as a module inside each example, not as its own example target.
//! - [`instruments`]: egui widgets (feature-gated).
//! - [`worlds`]: the canonical starting setups for the experiments
//!   (egui-free; the curriculum exporter uses these).

#![allow(dead_code)]

#[cfg(feature = "egui")]
pub mod instruments;

#[cfg(feature = "egui")]
pub mod program_editor;

pub mod worlds;
