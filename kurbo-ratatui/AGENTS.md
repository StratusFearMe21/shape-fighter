# AGENTS.md

## Project Overview

This is a Rust library named `kurbo-ratatui` that bridges `kurbo` (2D graphics library) and `ratatui` (terminal UI library). The library allows rendering `kurbo::BezPath` objects on `ratatui::Canvas` widgets with non-zero fill rule support.

## Project Type

- **Language**: Rust
- **Build System**: Cargo
- **Type**: Library (with example binary)
- **Edition**: 2024 (note: verify this matches your Rust version; current stable is 2021)
- **Dependencies**:
  - `peniko` 0.6.0 - Graphics/color library
  - `ratatui` 0.29 - Terminal UI framework (provides Canvas, Painter, Shape trait)
  - `ratatui-widgets` 0.3.0 - TUI widgets (without default features)
  - `kurbo` 0.11 - 2D graphics primitives and path operations

## Project Structure

```
.
├── Cargo.toml          # Project manifest and dependencies
├── Cargo.lock          # Dependency lock file (version-controlled)
├── README.md           # Project documentation
├── AGENTS.md           # Agent-specific development documentation
├── .gitignore          # Ignores /target directory
└── src/
    ├── lib.rs          # Main library implementation (BezPathShape)
    └── main.rs         # Example binary demonstrating usage
```

## Essential Commands

### Building
```bash
cargo build              # Debug build
cargo build --release    # Optimized release build
```

### Running
```bash
cargo run                # Build and run (debug)
cargo run --release      # Build and run (release)
```

### Testing
```bash
cargo test               # Run all tests
cargo test --release     # Run tests with optimizations
```

### Code Quality
```bash
cargo check              # Fast compile check (no binary generation)
cargo clippy             # Lint with Clippy
cargo fmt                # Format code with rustfmt
cargo fmt --check        # Check formatting without modifying
```

### Documentation
```bash
cargo doc                # Generate documentation
cargo doc --open         # Generate and open in browser
```

### Other Useful Commands
```bash
cargo clean              # Clean build artifacts (target/)
cargo update             # Update dependencies
cargo tree               # Display dependency tree
cargo outdated           # Check for outdated dependencies (requires cargo-outdated)
```

## Code Patterns and Conventions

### Library Structure
The project provides a library `src/lib.rs` and an example binary `src/main.rs`:

- **BezPathShape** (src/lib.rs:14-44): Main library type implementing `ratatui::widgets::canvas::Shape`
  - Wraps a `kurbo::BezPath`
  - Renders with non-zero fill rule using kurbo's winding number algorithm
  - Paints with `Color::White`

### Implementation Details
The rendering algorithm (src/lib.rs:26-42):
1. Gets the bounding box of the path to limit iteration
2. For each point in the bounding box, checks if inside using `path.winding(point)`
3. Uses non-zero fill rule: points with winding != 0 are inside
4. Converts canvas coordinates to grid coordinates via `painter.get_point()`
5. Paints grid cells with `Color::White` via `painter.paint()`

### Dependencies
- `kurbo` provides `BezPath`, bounding box, and winding number algorithms
- `ratatui` provides `Canvas`, `Painter`, `Shape` trait, and `Color`
- `ratatui-widgets` provides additional TUI widgets (currently unused in library)
- `peniko` provides graphics support (transitive dependency)

### Trait Aliases
Due to naming conflicts, kurbo's `Shape` trait is aliased as `KurboShape` (src/lib.rs:1)

## Build Artifacts

- **Target directory**: `/target` (ignored by git)
- **Debug builds**: `target/debug/kurbo-ratatui`
- **Release builds**: `target/release/kurbo-ratatui`

## Development Notes

- The project uses `Cargo.lock` for reproducible builds - commit this file
- Only the `/target` directory is excluded from version control
- When adding dependencies, update `Cargo.toml` and run `cargo check` to generate `Cargo.lock`

## Testing

The library includes unit tests in `src/lib.rs` (lines 47-73):
- `test_bezpath_shape_creation`: Verifies shape can be created
- `test_simple_rect`: Tests winding number with a rectangle path
- `test_circle_shape`: Tests winding number with a circle converted to path

Test organization:
- Unit tests: Place in the same file with `#[cfg(test)]` module (as in lib.rs)
- Integration tests: Create `tests/` directory at project root if needed
- Run tests with `cargo test` before committing changes

All tests currently pass (3 passed).

## Gotchas

- **Rust Edition**: Cargo.toml specifies edition 2024, which may not be stable yet. If you encounter build errors related to edition, consider changing to 2021.
- **Trait Name Conflict**: Both `kurbo` and `ratatui` export a `Shape` trait. Use aliases to avoid conflicts: `use kurbo::Shape as KurboShape;`
- **Default Features**: `ratatui-widgets` disables default features - explicitly enable any features you need
- **Deprecated Methods**: ratatui 0.29 has deprecated `Frame::size()` - use `Frame::area()` instead
- **Painter API**: The `Painter` type doesn't have a `bounds()` method in some versions; iterate over path bounding box instead
- **Winding Number**: The non-zero fill rule is implemented via kurbo's `winding()` method - don't reimplement it manually
- **Grid vs Canvas Coordinates**: Canvas uses Cartesian coordinates (origin at bottom-left), grid uses screen coordinates (origin at top-left). `painter.get_point()` handles the conversion

## Extending the Library

To add new shape types or features:

1. Create new types implementing `ratatui::widgets::canvas::Shape`
2. Use kurbo's geometry primitives (Circle, Rect, Line, etc.) and convert to BezPath via `to_path(tolerance)`
3. Reuse the winding number algorithm for non-zero fill rule
4. Add tests in the `#[cfg(test)]` module
5. Run `cargo test` and `cargo clippy` before committing

Example: Adding support for other kurbo shapes
```rust
use kurbo::{Rect, KurboShape};

let rect = Rect::new(10.0, 10.0, 50.0, 50.0);
let path = rect.to_path(0.1);
let shape = BezPathShape::new(path);
```
