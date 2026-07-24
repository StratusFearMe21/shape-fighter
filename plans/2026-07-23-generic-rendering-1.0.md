# Generic Rendering for Balls and Rectangles

## Objective

Unify the rendering pipeline so that `draw_system` uses a single query and a single iteration loop to render all shape types (currently circles and rectangles), eliminating the need to add a new query parameter and loop body every time a new shape type is introduced. Additionally, resolve the color asymmetry where circles carry a `RatatuiColor` component but rectangles are hardcoded to white.

## Current Architecture Analysis

### Problem Statement

The rendering system at `src/main.rs:136-227` uses **two separate Bevy queries** — one for circles (3-tuple with color) and one for rectangles (2-tuple without color) — each with its own iteration loop that duplicates the affine-transform and `builder.draw()` logic. This pattern does not scale: every new shape type requires adding another query parameter, another loop, and another block of near-identical code.

### Root Cause

The component `RatatuiShape<S>` in `src/shape.rs:4-6` is generic over the kurbo shape type `S`. Because Bevy identifies components by their concrete type, `RatatuiShape<Circle>` and `RatatuiShape<Rect>` are **distinct ECS components** that cannot be queried together. This forces separate queries per shape type.

### Key Code Locations

| What | Where | Details |
|------|-------|---------|
| Generic shape component | `src/shape.rs:4-6` | `RatatuiShape<S>` wrapping `BezPathShape<S>` |
| Color component (circles only) | `src/main.rs:28-30` | `RatatuiColor` newtype around `ratatui::style::Color` |
| Circle spawns (static) | `src/main.rs:76-96` | Two static circles with color |
| Rectangle spawns (static) | `src/main.rs:98-128` | Three walls, no color component |
| Dynamic circle spawn | `src/main.rs:257-269` | Mouse-click spawned balls with color |
| **Draw system (circles loop)** | `src/main.rs:149-169` | Iterates circle query, uses `color.0` |
| **Draw system (rectangles loop)** | `src/main.rs:170-190` | Iterates rectangle query, hardcodes `Color::White` |
| `ShapeBuilder::draw` (library) | `ratatui_kurbo/src/lib.rs:181` | Already generic: `draw<S: KurboShape>(&mut self, shape: &BezPathShape<S>, params)` |

### Critical Insight

The underlying `ShapeBuilder::draw` method is **already generic** over any `S: KurboShape` (`ratatui_kurbo/src/lib.rs:181`). The lack of generality is purely at the **ECS component/query level** — Bevy treats `RatatuiShape<Circle>` and `RatatuiShape<Rect>` as different components. The fix is to replace the generic component with a single non-generic component that can hold any shape type.

### Prioritized Challenges

1. **Highest priority — Dual-query rendering loop**: The core problem. Must be replaced with a single unified query and loop. This is the primary deliverable.
2. **High priority — Color asymmetry**: Rectangles lack `RatatuiColor`, forcing different query tuple shapes and a hardcoded color. Must be unified so all renderable entities are treated identically.
3. **Medium priority — Spawn site updates**: Five spawn sites (2 static circles, 3 static rectangles, 1 dynamic circle) must be updated to use the new component type.
4. **Low priority — `Clone` derive preservation**: The current `RatatuiShape<S>` derives `Clone`. The replacement should preserve `Clone` if feasible, to avoid breaking any future code that clones components.

## Implementation Plan

### Recommended Approach: Enum-Based Component

Replace the generic `RatatuiShape<S>` with a single enum component `ShapeKind` that wraps all supported shape variants. This approach:
- Eliminates heap allocation and dynamic dispatch (zero-cost at runtime)
- Preserves `Clone` and `Copy`-like semantics where inner types allow
- Provides exhaustive pattern matching (compiler warns when new variants are added but not handled)
- Requires only a 2-line change (add variant + add match arm) to support new shape types

### Phase 1: Redesign the Shape Component (`src/shape.rs`)

- [ ] **1.1.** Replace the generic `RatatuiShape<S>` struct with a non-generic enum `ShapeKind` that has one variant per supported kurbo shape type (e.g., `Circle(BezPathShape<Circle>)` and `Rect(BezPathShape<Rect>)`). Derive `Component` and `Clone` on the enum. Import `Circle` and `Rect` from `ratatui_kurbo::kurbo`.
  - **Rationale**: A single concrete component type allows one Bevy query to fetch all renderable entities regardless of their geometric shape.

- [ ] **1.2.** Add a `draw_to` method on `ShapeKind` that takes `&mut ShapeBuilder` and `ShapeParams`, and internally matches on the variant to call `builder.draw(&inner, params)`. Import `ShapeBuilder` and `ShapeParams` from `ratatui_kurbo`.
  - **Rationale**: Encapsulates the variant-to-`builder.draw()` dispatch in one location. The draw system calls `shape.draw_to(...)` without knowing which variant it is, making the system fully agnostic to shape types. Adding a new shape only requires adding a variant and a match arm here — the draw system never changes.

### Phase 2: Unify Color Handling (`src/main.rs`)

- [ ] **2.1.** Add `RatatuiColor(ratatui::style::Color::White)` to all three rectangle spawn bundles in `setup` (currently at `src/main.rs:98-128`).
  - **Rationale**: Makes all renderable entities carry a color component, eliminating the asymmetry that currently forces separate query tuple shapes. Rectangles render white as before, but now through the same data path as circles.

### Phase 3: Refactor the Draw System (`src/main.rs`)

- [ ] **3.1.** Replace the two separate query parameters (`circles` and `rectangles` at `src/main.rs:138-139`) with a single query: `Query<(&ShapeKind, &Transform, &RatatuiColor)>`.
  - **Rationale**: One query fetches all renderable entities uniformly. The `RatatuiColor` is now mandatory (not optional) because all entities have it after Phase 2.

- [ ] **3.2.** Replace the two iteration loops (`src/main.rs:149-169` for circles and `src/main.rs:170-190` for rectangles) with a single loop that calls `shape.draw_to(&mut builder, ShapeParams { color: color.0, affine: ... })` for every entity.
  - **Rationale**: Eliminates duplicated affine-transform and draw logic. The loop body is identical for all shape types.

- [ ] **3.3.** Update the import of `RatatuiShape` to `ShapeKind` from `crate::shape` at `src/main.rs:24`.
  - **Rationale**: The old component name no longer exists.

### Phase 4: Update Spawn Sites (`src/main.rs`)

- [ ] **4.1.** Update the two static circle spawns in `setup` (`src/main.rs:76-96`): replace `RatatuiShape(BezPathShape::new(Circle::new(...), ...))` with `ShapeKind::Circle(BezPathShape::new(Circle::new(...), ...))`.
  - **Rationale**: Spawn sites must use the new enum component.

- [ ] **4.2.** Update the three static rectangle spawns in `setup` (`src/main.rs:98-128`): replace `RatatuiShape(BezPathShape::new(Rect::from_center_size(...), ...))` with `ShapeKind::Rect(BezPathShape::new(Rect::from_center_size(...), ...))`, and add `RatatuiColor(ratatui::style::Color::White)` to each bundle.
  - **Rationale**: Same as 4.1, plus adding the color component per Phase 2.

- [ ] **4.3.** Update the dynamic circle spawn in `input_system` (`src/main.rs:257-269`): replace `RatatuiShape(BezPathShape::new(Circle::new(...), ...))` with `ShapeKind::Circle(BezPathShape::new(Circle::new(...), ...))`.
  - **Rationale**: The mouse-click spawn path must also use the new component.

### Phase 5: Clean Up Unused Imports (`src/main.rs`)

- [ ] **5.1.** Remove `Circle` and `Rect` from the `ratatui_kurbo::kurbo` import at `src/main.rs:20` if they are no longer directly referenced in `main.rs` after the refactor (they will still be needed if spawn sites construct them inline — verify before removing).
  - **Rationale**: Prevents compiler warnings about unused imports. Note: `Circle` and `Rect` are still needed at spawn sites that construct `Circle::new(...)` and `Rect::from_center_size(...)`, so they likely remain in scope.

## Verification Criteria

- [ ] `cargo build` compiles with zero errors and zero warnings
- [ ] `cargo clippy` passes without new warnings
- [ ] Running the application shows the same visual output as before: two colored static circles, three white walls, and dynamically spawned colored balls on mouse click
- [ ] The HUD timing display (build/rasterize/TTY times) still functions correctly
- [ ] `draw_system` contains exactly one `Query` parameter for shapes (not two)
- [ ] `draw_system` contains exactly one iteration loop over shapes (not two)
- [ ] All renderable entities (circles and rectangles) carry a `RatatuiColor` component
- [ ] Adding a hypothetical new shape type requires changes only in `shape.rs` (new enum variant + match arm) and at spawn sites — `draw_system` requires no modification

## Potential Risks and Mitigations

1. **`BezPathShape<S>` `Clone` bound for enum derive**
   The enum derives `Clone`, which requires `BezPathShape<Circle>` and `BezPathShape<Rect>` to implement `Clone`. Since `Circle` and `Rect` are `Copy` types in kurbo and `BezPathShape` derives `Clone` (confirmed at `ratatui_kurbo/src/lib.rs:78`), this is satisfied. No mitigation needed.

2. **Loss of type-level query discrimination**
   Currently, `RatatuiShape<Circle>` and `RatatuiShape<Rect>` allow type-safe queries for specific shape kinds. After the change, querying for only circles requires filtering by the enum variant at runtime. **Mitigation**: If per-shape-type queries are needed in the future, add Bevy marker components (e.g., `struct CircleMarker;`) at spawn time, or use the enum's `match` in the query loop. For the current codebase, no system other than `draw_system` queries shapes by type, so this is not an issue.

3. **`Send + Sync` / `Component` requirements**
   The enum contains `BezPathShape<Circle>` and `BezPathShape<Rect>`, both of which are `Send + Sync + 'static` (simple f64-based types). The `Component` derive will work without issues.

4. **Stale comment in `despawn_fallen_objects`**
   Unrelated to this task, but `src/main.rs:284-286` has a comment mentioning `despawn_recursive` while the code calls `despawn()`. This is pre-existing and out of scope.

## Alternative Approaches

### Alternative A: Trait Object with `Box<dyn DrawableShape>`

Define a trait `DrawableShape` with a `draw_to_builder` method, implement it generically for all `BezPathShape<S: KurboShape>`, and store `Box<dyn DrawableShape>` in a wrapper component.

- **Pros**: Truly open-closed — new shape types only implement the trait, no central enum to modify. Maximally extensible.
- **Cons**: Heap allocation per entity (one `Box` per spawn). Dynamic dispatch on every draw call. Loses `Clone` derive (would need a `clone_box` method on the trait). More complex for minimal benefit given the small, known set of shape types.
- **When to choose**: If the project expects many arbitrary shape types (e.g., user-loaded SVG paths, procedural polygons, Bezier curves) added frequently and dynamically.

### Alternative B: Bevy Trait Query (Experimental)

Bevy 0.18 may support querying for `&dyn Trait` directly if the trait is registered as a component trait. This would allow keeping `RatatuiShape<Circle>` and `RatatuiShape<Rect>` as separate components while querying them together via a shared trait.

- **Pros**: No wrapper component needed. Each shape type remains its own component. No allocation or dispatch overhead.
- **Cons**: Depends on Bevy's trait query feature maturity and API stability. More complex setup (trait registration). May not be fully stable in Bevy 0.18.
- **When to choose**: If Bevy's trait query API is confirmed stable in the target version and per-type component isolation is important.
