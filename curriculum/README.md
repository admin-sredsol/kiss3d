# PhysicsPlay Lab - Teacher Notes

Seven experiments, ordered so each physical law builds on the last. Every
experiment has a world file in `worlds/` (plain JSON, loadable via
`World::load_from_file`) and an interactive lab (`cargo run --features egui
--example <name>`). Each lesson follows the same loop: **predict -> run ->
measure -> compare**. Students state their guess before running; the meters and
the speed chart decide.

The block vocabulary is the lab instrument: every block names a physical
quantity. Nothing in the vocabulary is generic programming.

## 1. Galileo's drop  (`01-galileos-drop.json` / `galileo`)

- **Law**: free fall - mass does not change how fast things fall.
- **Question**: two balls, 0.5 kg and 4.2 kg, dropped from 10 m. Which lands first?
- **Measure**: fall time (s) per ball in the readouts; speed curves overlap on the chart.
- **Expected**: identical fall times (~1.4 s). Vary the densities and gravity with the sliders and re-drop.

## 2. Inertia  (`02-inertia.json` / `inertia`)

- **Law**: Newton's second law, a = F/m.
- **Question**: same 10 N push for one second on a 1 kg and a 4 kg cube - which moves faster?
- **Measure**: speed after the push; the chart shows two ramps with different slopes flattening into plateaus.
- **Expected**: the light cube ends at 4x the heavy cube's speed (within friction slop).

## 3. Friction lanes  (`03-friction-lanes.json` / `friction`)

- **Law**: sliding friction, stopping distance ~ v0^2 / (2 mu g).
- **Question**: same launch impulse on lanes with mu = 0.1, 0.5, 1.0 - where does each stop?
- **Measure**: distance from start per lane; speed decay curves on the chart.
- **Expected**: ~6x longer stopping distance on the low-friction lane (mu 0.1 vs 1.0).

## 4. Bounce decay  (`04-bounce-decay.json` / `bounce`)

- **Law**: restitution - each bounce keeps a fraction e of the impact speed, so peak heights shrink by ~e^2.
- **Question**: drop the ball at e = 0.7 - what happens to the bounce heights?
- **Measure**: flight peaks (tracked from touch events) and their ratio; the height chart shows the decaying envelope.
- **Expected**: peaks decay geometrically by roughly e^2 per bounce (the real-time solver is close to, not exactly, the textbook factor). The missing kinetic energy becomes heat and sound.

## 5. Ramps  (`05-ramps.json` / `ramp`)

- **Law**: gravity along an incline, a = g sin(theta).
- **Question**: tilt the ramp and watch the measured acceleration chase the theory value.
- **Measure**: the accelerometer readout (smoothed from the speed signal) vs g*sin(theta), plus the speed-vs-time slope.
- **Expected**: measured ~ theory (a little friction is included; the readout names it).

## 6. Collisions  (`06-collisions.json` / `collide`)

- **Law**: momentum conservation; restitution decides how speed is split.
- **Question**: ball A (7.6 m/s) hits resting ball B - what happens at e = 1 vs e = 0?
- **Measure**: speeds before/after (captured from the touch event), the momentum total, and the impact force readout.
- **Expected**: momentum is conserved at any e; e = 1 hands all speed to B (Newton's cradle), e = 0 makes them move together at v0/2.

## 7. The first block program  (`07-first-block-program.json` / `blocks_demo`)

- **Law**: reading - the world state before the experiment is applied.
- **Program**: WHEN START -> hit blue ball 4 kg*m/s; WHEN blue TOUCHES red -> SAY "I WIN!".
- **Task**: students edit the impulse (a real quantity) so the blue ball reaches the red one, then re-run. The block stack in the UI shows exactly what is programmed; Save/Load round-trips the world **with** its program.

## Determinism

The core steps Rapier with a fixed 1/60 s timestep, `enhanced-determinism` enabled, forces
applied per-tick. Same inputs on the same platform give the same run - students can compare
results. Cross-platform determinism (browser vs native) is approximate; compare runs on the
same platform.
