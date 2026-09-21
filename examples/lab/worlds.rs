//! Canonical starting worlds for the seven PhysicsPlay experiments.
//!
//! These constructors are the single source of truth for the curriculum: the
//! exporter turns them into hand-out JSON files. They mirror the setups the
//! interactive lab examples run; keep both in sync when tuning.

use physicsplay_core::{BodyKind, Entity, Material, Shape, World, WorldParams, Vec3};

fn entity(shape: Shape, kind: BodyKind, translation: Vec3, material: Material) -> Entity {
    Entity {
        translation,
        shape,
        kind,
        material,
        ..Entity::default()
    }
}

// --- 1. Galileo's drop ------------------------------------------------------

pub struct GalileoWorld {
    pub world: World,
    pub ground: usize,
    pub ball_a: usize,
    pub ball_b: usize,
    pub ball_radius: f32,
    pub spawn_a: Vec3,
    pub spawn_b: Vec3,
}

pub fn galileo() -> GalileoWorld {
    const DROP_HEIGHT: f32 = 10.0;
    let mut world = World::new(WorldParams::default());
    let ground = world.spawn(Entity {
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
    let spawn = |x: f32, density: f32| Entity {
        translation: Vec3::new(x, DROP_HEIGHT, 0.0),
        shape: Shape::Sphere { radius: 0.5 },
        kind: BodyKind::Dynamic,
        material: Material {
            friction: 0.2,
            restitution: 0.0,
            density,
        },
        ..Entity::default()
    };
    let ball_a = world.spawn(spawn(-2.0, 1.0));
    let ball_b = world.spawn(spawn(2.0, 8.0));
    GalileoWorld {
        world,
        ground,
        ball_a,
        ball_b,
        ball_radius: 0.5,
        spawn_a: Vec3::new(-2.0, DROP_HEIGHT, 0.0),
        spawn_b: Vec3::new(2.0, DROP_HEIGHT, 0.0),
    }
}

// --- 2. Inertia (F = ma) ------------------------------------------------------

pub struct InertiaWorld {
    pub world: World,
    pub cube_a: usize,
    pub cube_b: usize,
    pub spawn_a: Vec3,
    pub spawn_b: Vec3,
    pub cube_half: f32,
}

pub fn inertia() -> InertiaWorld {
    const CUBE_HALF: f32 = 0.5;
    let mut world = World::new(WorldParams::default());
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
    let spawn = |x: f32, density: f32| Entity {
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
    let cube_a = world.spawn(spawn(-3.0, 1.0));
    let cube_b = world.spawn(spawn(3.0, 4.0));
    InertiaWorld {
        world,
        cube_a,
        cube_b,
        spawn_a: Vec3::new(-3.0, CUBE_HALF, 0.0),
        spawn_b: Vec3::new(3.0, CUBE_HALF, 0.0),
        cube_half: CUBE_HALF,
    }
}

// --- 3. Friction lanes --------------------------------------------------------

pub struct FrictionWorld {
    pub world: World,
    /// One cube per lane, in lane order (mu = 0.1, 0.5, 1.0).
    pub cubes: [usize; 3],
    pub origins: [Vec3; 3],
    pub frictions: [f32; 3],
    pub cube_half: f32,
}

pub fn friction_lanes() -> FrictionWorld {
    const CUBE_HALF: f32 = 0.5;
    const START_X: f32 = -15.0;
    let lanes: [(f32, f32); 3] = [(-4.0, 0.1), (0.0, 0.5), (4.0, 1.0)];

    let mut world = World::new(WorldParams::default());
    for (z, friction) in lanes {
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
    }
    let mut cubes = [0usize; 3];
    let mut origins = [Vec3::ZERO; 3];
    let mut frictions = [0.0f32; 3];
    for (i, (z, friction)) in lanes.iter().enumerate() {
        let origin = Vec3::new(START_X, CUBE_HALF, *z);
        cubes[i] = world.spawn(Entity {
            translation: origin,
            shape: Shape::Cuboid {
                half_extents: Vec3::splat(CUBE_HALF),
            },
            kind: BodyKind::Dynamic,
            material: Material {
                friction: *friction,
                restitution: 0.0,
                ..Material::default()
            },
            ..Entity::default()
        });
        origins[i] = origin;
        frictions[i] = *friction;
    }
    FrictionWorld {
        world,
        cubes,
        origins,
        frictions,
        cube_half: CUBE_HALF,
    }
}

// --- 4. Bounce decay -----------------------------------------------------------

pub struct BounceWorld {
    pub world: World,
    pub ground: usize,
    pub ball: usize,
    pub ball_radius: f32,
    pub spawn: Vec3,
}

pub fn bounce() -> BounceWorld {
    const BALL_RADIUS: f32 = 0.4;
    const DROP_HEIGHT: f32 = 8.0;
    let mut world = World::new(WorldParams::default());
    let ground = world.spawn(Entity {
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
    let ball = world.spawn(Entity {
        translation: Vec3::new(0.0, DROP_HEIGHT, 0.0),
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
    BounceWorld {
        world,
        ground,
        ball,
        ball_radius: BALL_RADIUS,
        spawn: Vec3::new(0.0, DROP_HEIGHT, 0.0),
    }
}

// --- 5. Ramps -------------------------------------------------------------------

pub struct RampWorld {
    pub world: World,
    pub ramp: usize,
    pub block: usize,
    pub ramp_center: Vec3,
    pub ramp_half_len: f32,
    pub ramp_half_thick: f32,
    pub block_half: f32,
}

pub fn ramp() -> RampWorld {
    const RAMP_CENTER: Vec3 = Vec3::new(0.0, 4.0, 0.0);
    const RAMP_HALF_LEN: f32 = 6.0;
    const RAMP_HALF_THICK: f32 = 0.3;
    const BLOCK_HALF: f32 = 0.25;
    let mut world = World::new(WorldParams::default());
    let _floor = world.spawn(Entity {
        translation: Vec3::new(0.0, -0.5, 0.0),
        shape: Shape::Cuboid {
            half_extents: Vec3::new(30.0, 0.5, 20.0),
        },
        kind: BodyKind::Fixed,
        material: Material {
            friction: 0.4,
            restitution: 0.0,
            ..Material::default()
        },
        ..Entity::default()
    });
    let ramp = world.spawn(Entity {
        translation: RAMP_CENTER,
        shape: Shape::Cuboid {
            half_extents: Vec3::new(RAMP_HALF_LEN, RAMP_HALF_THICK, 2.0),
        },
        kind: BodyKind::Fixed,
        material: Material {
            friction: 0.02,
            restitution: 0.0,
            ..Material::default()
        },
        ..Entity::default()
    });
    let block = world.spawn(Entity {
        translation: RAMP_CENTER,
        shape: Shape::Cuboid {
            half_extents: Vec3::splat(BLOCK_HALF),
        },
        kind: BodyKind::Dynamic,
        material: Material {
            friction: 0.02,
            restitution: 0.0,
            ..Material::default()
        },
        ..Entity::default()
    });
    RampWorld {
        world,
        ramp,
        block,
        ramp_center: RAMP_CENTER,
        ramp_half_len: RAMP_HALF_LEN,
        ramp_half_thick: RAMP_HALF_THICK,
        block_half: BLOCK_HALF,
    }
}

// --- 6. Collisions (momentum) ----------------------------------------------------

pub struct CollideWorld {
    pub world: World,
    pub ball_a: usize,
    pub ball_b: usize,
    pub spawn_a: Vec3,
    pub spawn_b: Vec3,
    pub ball_radius: f32,
}

pub fn collide() -> CollideWorld {
    const BALL_RADIUS: f32 = 0.5;
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
    let spawn = |x: f32| Entity {
        translation: Vec3::new(x, BALL_RADIUS, 0.0),
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
    };
    let ball_a = world.spawn(spawn(-9.0));
    let ball_b = world.spawn(spawn(0.0));
    CollideWorld {
        world,
        ball_a,
        ball_b,
        spawn_a: Vec3::new(-9.0, BALL_RADIUS, 0.0),
        spawn_b: Vec3::new(0.0, BALL_RADIUS, 0.0),
        ball_radius: BALL_RADIUS,
    }
}

// --- 7. The first block program ---------------------------------------------------

pub struct FirstGameWorld {
    pub world: World,
    pub ground: usize,
    pub player: usize,
    pub target: usize,
    pub spawn_player: Vec3,
    pub spawn_target: Vec3,
    pub ball_radius: f32,
}

pub fn first_game() -> FirstGameWorld {
    const BALL_RADIUS: f32 = 0.5;
    let mut world = World::new(WorldParams::default());
    let _ground = world.spawn(Entity {
        translation: Vec3::new(0.0, -0.5, 0.0),
        shape: Shape::Cuboid {
            half_extents: Vec3::new(30.0, 0.5, 10.0),
        },
        kind: BodyKind::Fixed,
        material: Material {
            friction: 0.3,
            restitution: 0.0,
            ..Material::default()
        },
        ..Entity::default()
    });
    let ground = 0; // first spawn in this constructor
    let spawn = |x: f32| Entity {
        translation: Vec3::new(x, BALL_RADIUS, 0.0),
        shape: Shape::Sphere {
            radius: BALL_RADIUS,
        },
        kind: BodyKind::Dynamic,
        material: Material {
            friction: 0.2,
            restitution: 0.2,
            ..Material::default()
        },
        ..Entity::default()
    };
    let player = world.spawn(spawn(-6.0));
    let target = world.spawn(spawn(3.0));
    FirstGameWorld {
        world,
        ground,
        player,
        target,
        spawn_player: Vec3::new(-6.0, BALL_RADIUS, 0.0),
        spawn_target: Vec3::new(3.0, BALL_RADIUS, 0.0),
        ball_radius: BALL_RADIUS,
    }
}
