# Fix `Quat` → `Affine` Rotation Conversion in `draw_system`

## Objective

Fix the incorrect usage of bevy's `Quat` and kurbo's `Affine` at `src/main.rs:189-204`, where a 3D quaternion rotation is converted into a 2D affine transform. The current code discards the rotation axis (triggering an `unused variable` warning) and silently drops the sign of the rotation, so entities with negative rotation render incorrectly. The goal is a correct, idiomatic conversion that reads the 2D-relevant rotation from the `Quat` and composes a single equivalent `Affine`.

## Current Architecture Analysis

### The Two Types

**bevy `Quat`** (re-exported from `glam`, defined at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/glam-0.30.10/src/f32/scalar/quat.rs:40`):

A unit quaternion `(x, y, z, w)` representing a 3D rotation. The relevant accessor methods (all `#[must_use]`, return `f32`):

| Method | Signature | Source | Behavior |
|--------|-----------|--------|----------|
| `to_axis_angle` | `fn to_axis_angle(self) -> (Vec3, f32)` | `glam-0.30.10/src/f32/scalar/quat.rs:471` | Returns `(normalized axis, angle)` where `angle = 2 * atan2(|v|, w)`. Angle is always in `[0, π]`; the **axis encodes the sign**, so a negative rotation and its positive counterpart can produce the same angle with opposite axes. |
| `to_scaled_axis` | `fn to_scaled_axis(self) -> Vec3` | `glam-0.30.10/src/f32/scalar/quat.rs:487` | Returns `axis * angle` — the rotation axis scaled by the angle. This is the **signed** rotation vector; its `.z` component is the signed Z-rotation in radians. |
| `to_euler` | `fn to_euler(self, order: EulerRot) -> (f32, f32, f32)` | `glam-0.30.10/src/f32/scalar/quat.rs:495` | Returns `(i, j, k)` angles for the given sequence. For `EulerRot::ZYX`, the first element (`i`) is the **Z-axis rotation** in radians (signed). |

`EulerRot` is available via `bevy::prelude::*` (re-exported through `bevy_math::prelude` at `bevy_math-0.18.1/src/lib.rs:82`, which is glob-imported by `bevy_internal::prelude`). `EulerRot::ZYX` returns `(z, y, x)` — see `glam-0.30.10/src/euler.rs:28`.

**kurbo `Affine`** (defined at `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/kurbo-0.13.1/src/affine.rs:17`):

A 2D affine transform wrapping `[f64; 6]`. Relevant methods:

| Method | Signature | Source | Semantics |
|--------|-----------|--------|-----------|
| `translate` | `fn translate<V: Into<Vec2>>(p: V) -> Affine` | `affine.rs:106` | Pure translation. |
| `rotate` | `fn rotate(th: f64) -> Affine` | `affine.rs:88` | Rotation about the **origin**. Positive angle rotates +X toward +Y. |
| `rotate_about` | `fn rotate_about(th: f64, center: impl Into<Point>) -> Affine` | `affine.rs:97` | Rotation about `center`. |
| `then_rotate_about` | `fn then_rotate_about(self, th, center) -> Self` | `affine.rs:268` | `Affine::rotate_about(th, center) * self` — "self followed by rotation". |
| `pre_rotate` | `fn pre_rotate(self, th: f64) -> Self` | `affine.rs:180` | `self * Affine::rotate(th)` — "rotation followed by self". |

Key identity (from `affine.rs:97-102`): `rotate_about(th, c)` = `translate(-c).then_rotate(th).then_translate(c)`.

### Problem Statement

The current code at `src/main.rs:189-204`:

```rust
let (axis, angle) = transform.rotation.to_axis_angle();   // line 189
shape.draw(
    &mut builder,
    color.0,
    Affine::translate(Vec2::new(
        transform.translation.x as f64,
        transform.translation.y as f64,
    ))
    .then_rotate_about(
        angle as f64,                                       // line 198
        (
            transform.translation.x as f64,
            transform.translation.y as f64,
          ),                                                // line 199
    ),
);
```

This is a 2D game (the `Player` is a `Triangle`, spawned with `Transform::from_xyz` and `AngularVelocity`; all physics/colliders are 2D via `avian2d`). The only rotation that can affect a 2D affine transform is rotation about the Z axis. The code has three defects:

1. **`axis` is discarded (the warning).** `to_axis_angle()` returns a 3D axis; the code never inspects it. The compiler warns `unused variable: axis` at `src/main.rs:189`.

2. **Sign loss (correctness bug).** `to_axis_angle()`'s `angle` is always in `[0, π]`. For a Z-axis rotation, the *direction* of rotation is encoded entirely in the sign of `axis.z`, which is thrown away. A triangle rotating clockwise vs counter-clockwise by the same magnitude yields the same `angle`, so the rendered rotation is wrong for any rotation whose true signed angle is negative. The triangle's `AngularVelocity` (see `src/main.rs:109`) produces both positive and negative Z-rotations, so this is a live bug, not theoretical.

3. **Redundant affine composition.** `translate(t).then_rotate_about(angle, t)` expands (via the `rotate_about` identity) to `translate(t) * [translate(-t).then_rotate(angle).then_translate(t)]`. The leading `translate(t)` and the `translate(-t)` inside `rotate_about` cancel, leaving exactly `rotate(angle).then_translate(t)` = `Affine::translate(t).pre_rotate(angle)`. The center argument is mathematically a no-op because the translation and the rotation center are identical. The current form is correct in value (modulo defect #2) but obscures the intent and does extra matrix work.

### Root Cause

The author used `to_axis_angle()` (a general 3D API) without recognizing that (a) only the Z component matters for a 2D render, and (b) `to_axis_angle` deliberately folds the rotation sign into the axis, making the bare `angle` unsuitable when the axis is ignored.

## Implementation Plan

### Recommended Approach: `to_euler(EulerRot::ZYX).0`

Extract the signed Z-rotation directly with `to_euler`, then build a single affine via `Affine::translate(...).pre_rotate(z_angle)`. This is the minimal, idiomatic fix: it reads exactly the 2D-relevant scalar, preserves the sign, and produces the simplest equivalent affine.

Why `to_euler` over `to_scaled_axis().z`:
- `to_euler(EulerRot::ZYX).0` expresses intent ("the Z rotation") at the call site.
- `to_scaled_axis().z` is equally correct and one field access shorter, but reads as "the z of a scaled axis vector", which is less self-documenting. Both are acceptable; `to_euler` is recommended.

Why **not** keep `to_axis_angle`:
- It cannot yield a signed angle without also inspecting the axis, so any "fix" that keeps it must add `if axis.z < 0 { -angle } else { angle }` logic — more code, more error-prone, and still leaves the misleading `[0, π]` angle in play.

### Phase 1: Replace the Rotation Extraction and Affine Construction (`src/main.rs`)

- [ ] **1.1.** At `src/main.rs:189`, replace
  ```rust
  let (axis, angle) = transform.rotation.to_axis_angle();
  ```
  with
  ```rust
  let angle = transform.rotation.to_euler(EulerRot::ZYX).0 as f64;
  ```
  - **Rationale**: `EulerRot::ZYX` returns `(z, y, x)`; `.0` is the signed Z-rotation in radians (`glam-0.30.10/src/euler.rs:28`, `glam-0.30.10/src/f32/scalar/quat.rs:495`). This resolves the `unused variable: axis` warning and the sign-loss bug in one line. `EulerRot` is already in scope via the existing `use bevy::{prelude::*, ...}` at `src/main.rs:7` (re-exported through `bevy_math::prelude`, `bevy_math-0.18.1/src/lib.rs:82`). No new import is required.

- [ ] **1.2.** At `src/main.rs:193-203`, replace the affine construction
  ```rust
  Affine::translate(Vec2::new(
      transform.translation.x as f64,
      transform.translation.y as f64,
  ))
  .then_rotate_about(
      angle,
      (
          transform.translation.x as f64,
          transform.translation.y as f64,
      ),
  )
  ```
  with
  ```rust
  Affine::translate(Vec2::new(
      transform.translation.x as f64,
      transform.translation.y as f64,
  ))
  .pre_rotate(angle)
  ```
  - **Rationale**: Since the translation vector and the rotation center are identical, `translate(t).then_rotate_about(angle, t)` is algebraically equal to `translate(t).pre_rotate(angle)` (proof: `rotate_about(angle, t)` = `translate(-t).then_rotate(angle).then_translate(t)` per `affine.rs:97-102`; the outer `translate(t)` cancels the inner `translate(-t)`). `pre_rotate` is "rotation about the origin applied before the translation", which for a shape whose local geometry is centered at the origin (as the `Triangle`/`Circle`/`Rect` shapes are — see spawn sites `src/main.rs:79-82`, `111-114`) is exactly the intended world transform. This removes the redundant center tuple and the duplicate `translation.x/y` reads, and makes the "rotate then translate" intent explicit. Note `Vec2` here is `kurbo::Vec2` (imported at `src/main.rs:21`), and `Affine::translate` accepts `impl Into<Vec2>` (`affine.rs:106`).

### Phase 2 (Optional): Consolidate the Translation Read

- [ ] **2.1.** (Optional, readability) Hoist the translation into a local to avoid reading `transform.translation.x`/`.y` twice:
  ```rust
  let translation = Vec2::new(
      transform.translation.x as f64,
      transform.translation.y as f64,
  );
  shape.draw(
      &mut builder,
      color.0,
      Affine::translate(translation).pre_rotate(angle),
  );
  ```
  - **Rationale**: Pure cleanup; reduces duplication and clarifies that the same vector is used for both position and (formerly) rotation center. Skip if you prefer the most localized diff.

## Verification Criteria

- [ ] `cargo check` (or `cargo build`) compiles with **zero errors**.
- [ ] The `unused variable: axis` warning at `src/main.rs:189` is **gone** (no new warnings introduced).
- [ ] `cargo clippy` reports no new warnings in `draw_system`.
- [ ] Running the app: the `Player` triangle renders at the correct position **and** rotates in the correct direction matching its `AngularVelocity` (verify by observing both clockwise and counter-clockwise spin — previously the sign was lost so one direction rendered wrong).
- [ ] Static shapes (the `Map` circle at `src/main.rs:79` and the triangle at `111`) render identically to before when their rotation is identity (angle 0 ⇒ `pre_rotate(0.0)` is identity, matching the old `then_rotate_about(0.0, ...)`).
- [ ] No new `use` statements are required (confirm `EulerRot` resolves via `bevy::prelude::*`).

## Potential Risks and Mitigations

1. **`EulerRot` not in scope.**
   The plan asserts `EulerRot` is reachable from `bevy::prelude::*`. Verified: `bevy_math-0.18.1/src/lib.rs:82` exports `EulerRot` from `bevy_math::prelude`, and `bevy_internal-0.18.1/src/prelude.rs:3` glob-imports `math::prelude::*`. `src/main.rs:7` already does `use bevy::{prelude::*, ...}`. **Mitigation if it somehow fails**: add `use bevy::math::EulerRot;` — but this should not be necessary.

2. **Gimbal lock / Euler ambiguity for non-pure-Z rotations.**
   `to_euler(EulerRot::ZYX)` is well-defined for any quaternion, but if a transform ever carries X/Y rotation, the Z component alone may not reproduce the full orientation in 2D. **Mitigation**: This is a 2D game; all rotations are about Z (physics and spawns are 2D). If 3D rotations are ever introduced, the correct fix would be to project the quaternion onto the Z axis explicitly (e.g., `to_scaled_axis().z`), which is sign-correct and avoids Euler conventions entirely. The recommended `to_euler` approach is equivalent for pure-Z rotations.

3. **Coordinate-system handedness / Y-down.**
   `Affine::rotate` rotates +X toward +Y (`affine.rs:84-91`); the ratatui canvas is Y-down. The old code used the same `Affine::rotate_about` convention, so the sign convention is unchanged by this fix — only the previously-lost sign is now preserved. No handedness regression.

4. **Behavioral change for negative rotations.**
   This is the intended fix, not a risk: entities that previously rendered with a wrong (folded) angle will now render correctly. If any caller was *relying* on the buggy folding (unlikely), they would see a change. None of the spawn sites (`src/main.rs:74-117`) set a non-zero initial rotation, so initial renders are unaffected; only animated rotation correctness improves.

## Alternative Approaches

### Alternative A: `to_scaled_axis().z`

```rust
let angle = transform.rotation.to_scaled_axis().z as f64;
```
then the same `Affine::translate(...).pre_rotate(angle)`.

- **Pros**: One method call, no `EulerRot` argument, sign-correct by construction (`to_scaled_axis` = `axis * angle`, `glam-0.30.10/src/f32/scalar/quat.rs:487-490`). Robust to non-pure-Z rotations (extracts exactly the Z component of the rotation vector).
- **Cons**: `.z` of a "scaled axis" is slightly less self-documenting than "the Z euler angle".
- **When to choose**: If you want the most concise sign-correct extraction, or if there is any chance of non-pure-Z rotations sneaking in. Functionally equivalent to the recommended approach for this codebase.

### Alternative B: Keep `to_axis_angle`, recover the sign

```rust
let (axis, angle) = transform.rotation.to_axis_angle();
let z_angle = if axis.z < 0.0 { -angle } else { angle } as f64;
```

- **Pros**: Minimal diff to the existing line; keeps the familiar `to_axis_angle` call.
- **Cons**: Reintroduces a 3D concept (axis) only to immediately discard all but its sign; more verbose; relies on the `[0, π]` angle convention; still leaves `axis` partially "used" in a non-obvious way. Not recommended.

### Alternative C: `Quat::from_rotation_z` inverse via `to_array` / manual extraction

Manually compute the angle from the quaternion components (e.g., `2.0 * w.atan2(...)`). Not recommended — reimplements what `to_euler`/`to_scaled_axis` already provide correctly.
