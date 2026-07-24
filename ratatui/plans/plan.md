## Implementation Plan: Refactor Context to be Generic over Grid

### Research Summary

This plan addresses refactoring the Context type in ratatui-widgets/src/canvas.rs:528-545 to use static dispatch (generics) instead of dynamic dispatch (Box<dyn Grid>) for the Grid trait. The current design stores the grid behind a Box<dyn Grid> trait object at canvas.rs:541, which introduces vtable indirection on every paint, save, and reset call. Making Context generic over the concrete Grid type eliminates this overhead and opens the door for further inlining and monomorphization optimizations.

───────────────────────────────────────────────────────────────────────────────────────────

### Key Findings

#### 1. Current Dynamic Dispatch Point

The core of the problem is at ratatui-widgets/src/canvas.rs:528-545:

pub struct Context<'a> {
    width: u16,
    height: u16,
    x_bounds: [f64; 2],
    y_bounds: [f64; 2],
    grid: Box<dyn Grid>,   // <-- dynamic dispatch
    dirty: bool,
    layers: Vec<Layer>,
    labels: Vec<Label<'a>>,
}

The Grid trait is defined at canvas.rs:95-111 with four methods: resolution(), paint(), save(), and reset(). There are three concrete implementors:
• PatternGrid<W, H> (canvas.rs:141)
• CharGrid (canvas.rs:240)
• HalfBlockGrid (canvas.rs:326)

#### 2. Where Box<dyn Grid> is Created

The factory method Context::marker_to_grid() at canvas.rs:591-606 matches on Marker and returns Box<dyn Grid>. This is the sole site where the grid type is decided.

#### 3. The marker() Method Complicates Generics

Context::marker() at canvas.rs:611-614 allows changing the marker type mid-render, which forces the current design to use dynamic dispatch since the grid type can change at runtime:

pub fn marker(&mut self, marker: Marker) {
    self.finish();
    self.grid = Self::marker_to_grid(self.width, self.height, marker);
}

#### 4. Painter Depends on Context

Painter at canvas.rs:416-419 holds a &'a mut Context<'b>. It will need to become generic as well:

pub struct Painter<'a, 'b> {
    context: &'a mut Context<'b>,
    resolution: (f64, f64),
}

#### 5. The Shape Trait Uses Painter

The Shape trait at canvas.rs:51-57 takes &mut Painter:

pub trait Shape {
    fn draw(&self, painter: &mut Painter);
}

This is a public trait that third-party code implements. If Painter becomes generic, Shape::draw must also become generic, which is a breaking API change for anyone implementing Shape externally.

#### 6. Canvas Widget Renders Context

The Canvas widget render implementation at canvas.rs:873-947 creates a Context::new() and calls painter(&mut ctx). The paint closure signature F: Fn(&mut Context) appears throughout the Canvas type bounds.

#### 7. External Usage

External code (e.g., examples/apps/volatility-surface/src/display/surface_3d.rs) uses Context directly inside paint closures, calling methods like ctx.draw(), ctx.layer(), etc. The Context type appears in closure signatures.

───────────────────────────────────────────────────────────────────────────────────────────

### Technical Details: Step-by-Step Implementation Plan

#### Phase 1: Make Grid Public and Add a Type Parameter to Context

File: ratatui-widgets/src/canvas.rs

Step 1.1: Make the Grid trait public (currently trait Grid at line 95). This is needed so external consumers who implement custom grids can use the generic Context<G>.

// Before (line 95):
trait Grid: fmt::Debug {

// After:
pub trait Grid: fmt::Debug {

Step 1.2: Add generic type parameter G: Grid to Context:

// Before (lines 528-545):
pub struct Context<'a> {
    ...
    grid: Box<dyn Grid>,
    ...
}

// After:
pub struct Context<'a, G: Grid> {
    ...
    grid: G,
    ...
}

Step 1.3: Update Context::new() to accept a grid directly instead of a Marker, or provide a constructor that takes G:

Option A (preferred): Change new() to take G directly and add a helper:

impl<'a, G: Grid> Context<'a, G> {
    pub fn new(width: u16, height: u16, x_bounds: [f64; 2], y_bounds: [f64; 2], grid: G
      ) -> Self {
        Self {
            width,
            height,
            x_bounds,
            y_bounds,
            grid,
            dirty: false,
            layers: Vec::new(),
            labels: Vec::new(),
        }
    }
}

Option B: Keep Context::new() taking Marker by providing a concrete grid type for each marker. This would be done via a separate factory or by making Context::new() generic. Since Marker determines the grid at runtime, you cannot have a single Context::new(width, height, x_bounds, y_bounds, marker) that returns a generic Context<G> without knowing G at compile time. This is the fundamental tension.

Key Insight: The marker() method on Context (line 611) allows changing the grid type at runtime. If Context becomes generic over G, this method cannot exist in its current form because G is a compile-time parameter. There are two approaches:

• Approach A: Remove Context::marker() and require the grid type to be fixed at
  construction time. Users who need multiple markers must create separate Context instances
  (i.e., use separate Canvas widgets or layers).
• Approach B: Keep Context::marker() on a separate non-generic Context type (or use an
  enum-based grid) for backward compatibility.

Recommendation: Approach A is cleaner. The marker() method is rarely used in practice (none of the examples or tests call it on Context directly). It can be removed or deprecated.

#### Phase 2: Make Painter Generic

Step 2.1: Update Painter at canvas.rs:416-419:

// Before:
pub struct Painter<'a, 'b> {
    context: &'a mut Context<'b>,
    resolution: (f64, f64),
}

// After:
pub struct Painter<'a, 'b, G: Grid> {
    context: &'a mut Context<'b, G>,
    resolution: (f64, f64),
}

Step 2.2: Update all Painter methods (get_point, paint, bounds) at lines 462-511. These methods don't depend on G except paint() which calls self.context.grid.paint(). The generic parameter just needs to be propagated.

Step 2.3: Update the From<&mut Context> impl at lines 513-521:

impl<'a, 'b, G: Grid> From<&'a mut Context<'b, G>> for Painter<'a, 'b, G> {
    fn from(context: &'a mut Context<'b, G>) -> Self {
        let resolution = context.grid.resolution();
        Self { context, resolution }
    }
}

#### Phase 3: Update the Shape Trait

Step 3.1: This is the most API-sensitive change. The Shape trait at line 51 must become generic:

// Before:
pub trait Shape {
    fn draw(&self, painter: &mut Painter);
}

// After:
pub trait Shape {
    fn draw<G: Grid>(&self, painter: &mut Painter<'_, '_, G>);
}

Impact: All existing Shape implementations need updating. Within this crate, these are:
• Circle (canvas/circle.rs:32-43)
• Line (canvas/line.rs:46-64)
• FilledLine (canvas/line.rs:205-234)
• Points (canvas/points.rs:21-28)
• Rectangle (canvas/rectangle.rs:40-76)
• Map (canvas/map.rs:49-57)

All of these only call painter.get_point() and painter.paint(), so they are already grid-agnostic. The change is purely mechanical: add <G: Grid> generic parameter to draw.

Breaking change: External implementations of Shape will need to update their signatures. This is a semver-major change.

Alternative (non-breaking): Use a trait object approach for Shape too, or use an enum dispatch pattern. But this defeats the purpose of the refactoring.

#### Phase 4: Update Context::draw() and Context::layer()

Step 4.1: Context::draw() at line 617:

// Before:
pub fn draw<S>(&mut self, shape: &S)
where
    S: Shape,
{
    self.dirty = true;
    let mut painter = Painter::from(self);
    shape.draw(&mut painter);
}

// After:
impl<'a, G: Grid> Context<'a, G> {
    pub fn draw<S>(&mut self, shape: &S)
    where
        S: Shape,
    {
        self.dirty = true;
        let mut painter = Painter::from(self);
        shape.draw::<G>(&mut painter);
    }
}

Step 4.2: Context::layer() at line 633 and finish() at line 657 need no changes beyond updating the impl block to impl<'a, G: Grid> Context<'a, G>.

#### Phase 5: Update Canvas Widget

Step 5.1: The Canvas widget at line 734 needs to dispatch on Marker to instantiate the correct Context<G> concrete type. The render method at lines 873-947 currently creates a Context::new() with a Marker. This must be changed to match on Marker and create the appropriate concrete grid type:

impl<F> Widget for &Canvas<'_, F>
where
    F: Fn(&mut Context<'_, ???>),  // Problem: what is G?
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        // ...
        match self.marker {
            Marker::Braille => { /* create Context<PatternGrid<2,4>> */ }
            Marker::Block => { /* create Context<CharGrid> */ }
            // etc.
        }
    }
}

The fundamental challenge: The closure F: Fn(&mut Context) must now be generic over G, but Rust doesn't support F: Fn(&mut Context<G>) for all G: Grid. The closure must be callable with any concrete Context<G>.

Solution: Use a helper trait or use an enum-based dispatch at the Canvas level. Here are the options:

Option A: Erased Context (recommended)

Keep an internal erased wrapper for the render path. The Canvas render method matches on Marker, creates the concrete Context<G>, calls the closure with a &mut Context<G>, then extracts layers. This works if the closure type F can accept &mut Context<G> for any G. Since F is defined by the user and G is determined by the marker, this requires the closure to be written generically.

This would change the Canvas API to:

pub struct Canvas<'a, F>
where
    F: for<G: Grid> Fn(&mut Context<'_, G>),

But Rust doesn't support higher-ranked trait bounds over associated types like this. This approach is not directly expressible in Rust's type system.

Option B: Enum Dispatch for Grid

Create an enum that wraps all grid types and implements Grid without dynamic dispatch (enum dispatch pattern):

enum CanvasGrid {
    Braille(PatternGrid<2, 4>),
    Block(CharGrid),
    HalfBlock(HalfBlockGrid),
    Quadrant(PatternGrid<2, 2>),
    Sextant(PatternGrid<2, 3>),
    Octant(PatternGrid<2, 4>),
    Dot(CharGrid),
    Bar(CharGrid),
    Custom(CharGrid),
}

Then Context becomes Context<'a, G: Grid> but Canvas uses Context<'a, CanvasGrid> internally. This gives monomorphization for the Context + Painter + Shape pipeline while keeping the Canvas API unchanged.

This is the recommended approach. It avoids the higher-ranked trait bound problem and maintains backward compatibility.

Option C: Separate Canvas types per marker

Create CanvasBraille, CanvasBlock, etc. This is too verbose and breaks the existing API.

#### Phase 6: Implement Enum Dispatch (Recommended Approach)

Step 6.1: Define CanvasGrid enum in canvas.rs:

#[derive(Debug)]
pub enum CanvasGrid {
    Braille(PatternGrid<2, 4>),
    Block(CharGrid),
    HalfBlock(HalfBlockGrid),
    Quadrant(PatternGrid<2, 2>),
    Sextant(PatternGrid<2, 3>),
    Octant(PatternGrid<2, 4>),
    Dot(CharGrid),
    Bar(CharGrid),
    Custom(CharGrid),
}

Step 6.2: Implement Grid for CanvasGrid:

impl Grid for CanvasGrid {
    fn resolution(&self) -> (f64, f64) {
        match self {
            Self::Braille(g) => g.resolution(),
            Self::Block(g) => g.resolution(),
            // ... etc
        }
    }

    fn paint(&mut self, x: usize, y: usize, color: Color) {
        match self {
            Self::Braille(g) => g.paint(x, y, color),
            // ... etc
        }
    }

    fn save(&self) -> Layer { /* similar match */ }
    fn reset(&mut self) { /* similar match */ }
}

This is "manual monomorphization" via enum dispatch. The compiler can still inline and optimize through the match arms because all types are concrete and known.

Step 6.3: Define type aliases for the common case:

/// The standard context type used by Canvas, with enum-dispatched grid.
pub type CanvasContext<'a> = Context<'a, CanvasGrid>;

Step 6.4: Update Canvas to use Context<'a, CanvasGrid>:

impl<F> Widget for &Canvas<'_, F>
where
    F: Fn(&mut Context<'_, CanvasGrid>),
{
    fn render(self, area: Rect, buf: &mut Buffer) {
        // ... existing block/area setup ...
        let grid = marker_to_grid(width, height, self.marker);
        let mut ctx = Context::new(width, height, self.x_bounds, self.y_bounds, grid);
        painter(&mut ctx);
        ctx.finish();
        // ... existing layer rendering ...
    }
}

Where marker_to_grid returns CanvasGrid instead of Box<dyn Grid>.

Step 6.5: The Shape trait can remain non-generic if we use CanvasGrid as the grid type:

pub trait Shape {
    fn draw(&self, painter: &mut Painter<'_, '_, CanvasGrid>);
}

This avoids the breaking change to Shape. The trade-off is that custom Grid implementations outside the crate cannot be used with Shape unless they are wrapped in CanvasGrid. But this is acceptable since Grid is currently private anyway.

#### Phase 7: Allow Advanced Users to Use Fully Generic Context

For users who want maximum performance with a single known grid type, expose the generic Context<'a, G> and Painter<'a, 'b, G> publicly. They can implement their own Shape variant or use Painter directly.

───────────────────────────────────────────────────────────────────────────────────────────

### Insights and Context

Why dynamic dispatch was chosen: The original design used Box<dyn Grid> because Context::marker() allows changing the grid type at runtime within a single paint closure. This is a flexible API but introduces vtable overhead on every paint() call (which happens thousands of times per frame for shapes like Map with 5000 points).

Performance impact of the refactoring: The enum dispatch approach eliminates the heap allocation (Box::new) and vtable indirection. Each paint() call becomes a match on an enum variant followed by a direct function call. For hot paths like Map::draw() which calls painter.paint() 5000 times, this can be measurable. The compiler may also be able to inline the match arms.

The Context::marker() method: This method at canvas.rs:611-614 is the main obstacle. It allows changing the grid type mid-render. With enum dispatch, this method can be preserved by converting between CanvasGrid variants. The finish() call saves the current layer, and then the grid is replaced with a new CanvasGrid variant.

Breaking changes assessment:
• If Shape stays non-generic (using CanvasGrid): No breaking changes for external Shape
  implementors, since Grid was private and Painter's type changes are hidden behind the
  CanvasGrid alias.
• If Shape becomes generic over G: Grid: Breaking change for external implementors.

───────────────────────────────────────────────────────────────────────────────────────────

### Follow-up Suggestions

1. Benchmark before/after: Create a benchmark comparing Box<dyn Grid> vs CanvasGrid enum
   dispatch for rendering a Map with high resolution (5000 points). The Map shape is the
   best benchmark candidate since it's the most paint-heavy.

2. Consider removing Context::marker(): Survey whether any downstream code actually uses
   Context::marker() to change markers mid-render. If not, removing it simplifies the
   design significantly.

3. Expose Grid trait publicly: If making Grid public, document it clearly as an extension
   point for custom grid types. Consider adding a CustomGrid variant to CanvasGrid that
   boxes a user-provided grid for full extensibility.

4. Consider Shape generic version: In a future major version bump, make Shape::draw()
   generic over G: Grid for maximum flexibility. This would allow shape implementations
   that are specialized for particular grid types (e.g., a HalfBlockCircle that uses the
   two-color capability of HalfBlockGrid).

5. Related file changes summary:
      ◦ ratatui-widgets/src/canvas.rs:95-111 - Make Grid public
      ◦ ratatui-widgets/src/canvas.rs:141-233 - PatternGrid (no changes needed)
      ◦ ratatui-widgets/src/canvas.rs:240-310 - CharGrid (no changes needed)
      ◦ ratatui-widgets/src/canvas.rs:326-409 - HalfBlockGrid (no changes needed)
      ◦ ratatui-widgets/src/canvas.rs:416-521 - Painter becomes generic over G: Grid
      ◦ ratatui-widgets/src/canvas.rs:528-662 - Context becomes generic over G: Grid
      ◦ ratatui-widgets/src/canvas.rs:734-947 - Canvas uses CanvasGrid enum
      ◦ ratatui-widgets/src/canvas/circle.rs:32-43 - Shape::draw signature update
      ◦ ratatui-widgets/src/canvas/line.rs:46-64 - Shape::draw signature update
      ◦ ratatui-widgets/src/canvas/line.rs:205-234 - Shape::draw signature update
      ◦ ratatui-widgets/src/canvas/points.rs:21-28 - Shape::draw signature update
      ◦ ratatui-widgets/src/canvas/rectangle.rs:40-76 - Shape::draw signature update
      ◦ ratatui-widgets/src/canvas/map.rs:49-57 - Shape::draw signature update
      ◦ ratatui-widgets/src/canvas/line.rs:87-96 - draw_line helper signature update
