use peniko::kurbo::{Affine, Point};
use ratatui::Frame;
use ratatui::crossterm::event;
use ratatui::layout::Constraint;
use ratatui::style::Color;
use ratatui::widgets::canvas::Canvas;
use ratatui_kurbo::{ShapeBuilder, ShapeParams, rust_r, rust_r_stroke};
use std::io;
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
    // Equivalent to
    // BezPathShape::new(
    //     RUST_R,
    //     Rect::from_origin_size((0.0, 0.0), (106.0, 106.0)),
    //     Fill::NonZero,
    //     color,
    //     Affine::IDENTITY,
    // )
    let bez_path_shapes = [rust_r(), rust_r_stroke()];
    let mut affines = [Affine::IDENTITY, Affine::IDENTITY];
    // let mut bez_path_shapes = [test_star(Color::White, peniko::Fill::EvenOdd)];
    // let first_triangle = Triangle::EQUILATERAL.inflate(8.0);
    // let second_triangle = Triangle::EQUILATERAL.inflate(32.0);
    // let third_triangle = Triangle::EQUILATERAL.inflate(64.0);
    // let mut bez_path_shapes = [
    //     BezPathShape::with_bounding_box(
    //         first_triangle,
    //         first_triangle.bounding_box().inset(8.0),
    //         peniko::Style::Stroke(Stroke::new(1.0).with_join(Join::Round)),
    //         Color::White,
    //         Affine::IDENTITY,
    //     ),
    //     BezPathShape::with_bounding_box(
    //         second_triangle,
    //         second_triangle.bounding_box().inset(16.0),
    //         peniko::Style::Stroke(Stroke::new(1.0).with_join(Join::Round)),
    //         Color::White,
    //         Affine::IDENTITY,
    //     ),
    //     BezPathShape::with_bounding_box(
    //         third_triangle,
    //         third_triangle.bounding_box().inset(32.0),
    //         peniko::Style::Stroke(Stroke::new(1.0).with_join(Join::Round)),
    //         Color::White,
    //         Affine::IDENTITY,
    //     ),
    // ];
    let path_center = Point::new(
        (bez_path_shapes[0].bbox.x0 + bez_path_shapes[0].bbox.x1) / 2.0,
        (bez_path_shapes[0].bbox.y0 + bez_path_shapes[0].bbox.y1) / 2.0,
    );

    let mut terminal = ratatui::try_init()?;
    terminal.clear()?;

    let mut time = Instant::now();
    loop {
        let delta_time = time.elapsed();
        time = Instant::now();
        affines.iter_mut().for_each(|affine| {
            *affine = affine.then_rotate_about(0.1 * delta_time.as_secs_f64(), path_center);
        });
        terminal.draw(|f: &mut Frame| {
            let area = f.area();
            let area = f.area().centered(
                Constraint::Length(area.width.min(area.height) * 2),
                Constraint::Length(area.width.min(area.height)),
            );
            let mut builder = ShapeBuilder::new();
            for (shape, affine) in bez_path_shapes.iter().zip(affines.iter()) {
                builder.draw(
                    shape,
                    ShapeParams {
                        color: Color::White,
                        affine: *affine,
                    },
                );
            }
            let shapes = builder.finish();
            let canvas = Canvas::default()
                // Each cell has a resolution of 2x4
                .x_bounds([area.left() as f64 * 2.0, area.right() as f64 * 2.0])
                .y_bounds([area.top() as f64 * 4.0, area.bottom() as f64 * 4.0])
                .paint(|ctx| ctx.draw(&shapes));
            f.render_widget(canvas, area);
        })?;

        if event::poll(Duration::from_millis(8))? {
            match event::read()? {
                event::Event::Key(event::KeyEvent {
                    code: event::KeyCode::Char('q'),
                    ..
                }) => break,
                _ => {}
            }
        }
    }

    terminal.show_cursor()?;

    ratatui::try_restore()
}
