# kurbo-ratatui

A Rust library that bridges [kurbo](https://crates.io/crates/kurbo) (2D graphics library) and [ratatui](https://crates.io/crates/ratatui) (terminal UI library), allowing you to render `kurbo::BezPath` objects on `ratatui::Canvas` widgets.

## Features

- Render `kurbo::BezPath` paths to `ratatui::Canvas`
- Non-zero fill rule for proper path filling
- Simple and ergonomic API

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
kurbo-ratatui = "0.1"
kurbo = "0.11"
ratatui = "0.29"
```

## Usage

### Basic Example

```rust
use kurbo::{Circle, Shape as KurboShape};
use kurbo_ratatui::BezPathShape;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::canvas::Canvas;
use ratatui::Frame;

fn draw_circle(f: &mut Frame) {
    // Create a circle and convert it to a BezPath
    let circle = Circle::new((50.0, 50.0), 30.0);
    let path = circle.to_path(0.1);

    // Create a BezPathShape from the path
    let bez_path_shape = BezPathShape::new(path);

    // Render on a Canvas
    let canvas = Canvas::default()
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0])
        .paint(|ctx| {
            ctx.draw(&bez_path_shape);
        });

    f.render_widget(canvas, f.area());
}
```

### Creating Custom Paths

```rust
use kurbo::{BezPath, Point, PathEl};
use kurbo_ratatui::BezPathShape;

let mut path = BezPath::new();
path.move_to((0.0, 0.0));
path.line_to((10.0, 10.0));
path.curve_to((20.0, 0.0), (30.0, 20.0), (40.0, 10.0));
path.close_path();

let shape = BezPathShape::new(path);
```

## How It Works

The library uses kurbo's built-in winding number algorithm to implement the non-zero fill rule:

1. For each point in the path's bounding box, the winding number is calculated
2. Points with a non-zero winding number are considered "inside" the path
3. These points are rendered on the canvas with `Color::White`

## Non-Zero Fill Rule

The non-zero fill rule is a standard algorithm for determining which areas should be filled in complex paths:

- Count the winding number at each point (how many times the path winds around it)
- Positive and negative crossings cancel each other out
- Fill areas where the winding number is non-zero

This allows for proper rendering of:
- Self-intersecting paths
- Holes in shapes
- Complex polygon combinations

## API Reference

### `BezPathShape`

```rust
pub struct BezPathShape {
    // private fields
}
```

A shape that renders a `kurbo::BezPath` on a ratatui Canvas.

#### Methods

- **`new(path: kurbo::BezPath) -> Self`** - Creates a new `BezPathShape` from a `kurbo::BezPath`

## Dependencies

- `kurbo` 0.11 - 2D graphics primitives and path operations
- `ratatui` 0.29 - Terminal UI framework
- `peniko` 0.6 - Graphics and color support (transitive)

## Examples

See `src/main.rs` for a complete working example that renders a filled circle on the terminal.

## License

[Specify your license here]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
