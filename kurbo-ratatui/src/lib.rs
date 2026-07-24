//! # ratatui_kurbo
//!
//! This crate allows you to render kurbo shapes using the [`ratatui_widgets::canvas::Canvas`].

use std::mem;
use std::ops::Deref;
use std::sync::LazyLock;

use kurbo::{Affine, BezPath, PathEl, Rect, Shape as KurboShape};
use peniko::Fill;
use peniko::kurbo::Line;
use peniko::kurbo::Stroke;
use peniko::kurbo::StrokeOpts;
use ratatui_core::style::Color;

use ratatui_widgets::canvas::Painter;
use ratatui_widgets::canvas::Shape;

pub use peniko;
pub use peniko::kurbo;
pub use ratatui_widgets;

const TOLERANCE: f64 = 0.25;
static RUST_R: LazyLock<BezPath> = LazyLock::new(|| {
    BezPath::from_svg(
        "M 44.5,38.5 H 57.5 C 65.5,38.5 65.5,46.5 57.5,46.5 H 44.5 Z\
M 13.5,75.5 H 53.5 V 64.5 H 44.5 V 56.5 H 54.5 C 65.5,56.5 59.5,75.5 68.5,75.5 H 93.5\
V 56.5 H 87.5 V 58.5 C 87.5,66.5 78.5,65.5 77.5,60.5 C 76.5,55.5 72.5,51.5 71.5,51.5 C 86.5
,43.5 77.5,27.5 65.5,27.5 H 18.5\
V 38.5 H 28.5 V 64.5 H 13.5 Z",
    )
    .expect("Failed to parse Rust R SVG")
});

static TEST_STAR: LazyLock<BezPath> = LazyLock::new(|| {
    BezPath::from_svg("M50 3, 20 97, 95 37, 5 37, 80 97, 50 3")
        .expect("Failed to parse star SVG path")
});

/// A neat example shape that you can use for testing and demo purposes
pub fn rust_r() -> BezPathShape<&'static BezPath> {
    BezPathShape::with_bounding_box(
        RUST_R.deref(),
        Rect::from_origin_size((0.0, 0.0), (106.0, 106.0)),
        peniko::Style::Fill(Fill::NonZero),
    )
}

/// The corresponding stroke of [`rust_r`]
pub fn rust_r_stroke() -> BezPathShape<&'static BezPath> {
    BezPathShape::with_bounding_box(
        RUST_R.deref(),
        Rect::from_origin_size((0.0, 0.0), (106.0, 106.0)),
        peniko::Style::Stroke(Stroke::new(1.0).with_join(kurbo::Join::Round)),
    )
}

/// A neat example shape that you can use for testing and demo purposes
pub fn test_star(fill: Fill) -> BezPathShape<&'static BezPath> {
    BezPathShape::with_bounding_box(
        TEST_STAR.deref(),
        Rect::from_origin_size((0.0, 0.0), (100.0, 100.0)),
        peniko::Style::Fill(fill),
    )
}

#[derive(Debug, Clone, Copy)]
struct Edge {
    shape_idx: usize,
    y_min: f64,
    y_max: f64,
    x: f64,
    slope_inv: f64, // (x1 - x0) / (y1 - y0)
    winding: i32,
}

/// A shape that renders a [`kurbo::Shape`] on a ratatui Canvas.
#[derive(Clone)]
pub struct BezPathShape<S> {
    /// The shape to be rendered
    pub shape: S,
    /// The fill rule to use when filling
    /// in the shape
    pub style: peniko::Style,
    /// The bounding box of the shape, can be
    /// derived with [`kurbo::Shape::bounding_box`],
    /// or can be custom in the case where there should
    /// be space around the shape.
    pub bbox: Rect,
}

impl<S> BezPathShape<S> {
    /// Creates a new [`BezPathShape`] from a [`kurbo::Shape`] (const version).
    pub const fn with_bounding_box(shape: S, bbox: Rect, style: peniko::Style) -> Self {
        Self { shape, bbox, style }
    }
}

impl<S: KurboShape> BezPathShape<S> {
    /// Creates a new [`BezPathShape`] from a [`kurbo::Shape`].
    pub fn new(shape: S, style: peniko::Style) -> Self {
        Self {
            bbox: shape.bounding_box(),
            shape,
            style,
        }
    }
}

fn shape_to_lines(
    path: impl IntoIterator<Item = PathEl>,
    transform: Affine,
    mut callback: impl FnMut(Line),
) {
    let mut start_last = None;
    kurbo::flatten(path.into_iter().map(|el| transform * el), TOLERANCE, |el| {
        // We first need to check whether this is the first
        // path element we see to fill in the start position.
        let (start, last) = start_last.get_or_insert_with(|| {
            let point = match el {
                PathEl::MoveTo(p) => p,
                PathEl::LineTo(p) => p,
                PathEl::ClosePath => panic!("Can't start a segment on a ClosePath"),
                _ => unreachable!(),
            };
            (point, point)
        });

        match el {
            PathEl::MoveTo(p) => {
                *start = p;
                *last = p;
            }
            PathEl::LineTo(p) => {
                callback(Line::new(mem::replace(last, p), p));
            }
            PathEl::ClosePath => {
                if *last != *start {
                    callback(Line::new(mem::replace(last, *start), *start));
                }
            }
            _ => unreachable!(),
        }
    });
}

struct InternalShapeParams {
    color: Color,
    fill: Fill,
}

impl InternalShapeParams {
    fn is_inside(&self, shape_winding: i32) -> bool {
        match self.fill {
            Fill::NonZero => shape_winding != 0,
            Fill::EvenOdd => (shape_winding.abs() % 2) != 0,
        }
    }
}

pub struct ShapeParams {
    pub color: Color,
    pub affine: Affine,
}

pub struct ShapeBuilder {
    edges: Vec<Edge>,
    shape_params: Vec<InternalShapeParams>,
    bbox: Rect,
}

impl ShapeBuilder {
    pub fn new() -> Self {
        Self {
            edges: Vec::new(),
            shape_params: Vec::new(),
            bbox: Rect::ZERO,
        }
    }

    pub fn draw<S: KurboShape>(&mut self, shape: &BezPathShape<S>, params: ShapeParams) {
        let target_rect = params.affine.transform_rect_bbox(shape.bbox);

        self.bbox = self.bbox.union(target_rect);
        let shape_idx = self.shape_params.len();

        let shape_iter = shape
            .shape
            .path_elements(TOLERANCE)
            .chain(std::iter::once(PathEl::ClosePath));

        let callback = |l: Line| {
            if (l.p0.y - l.p1.y).abs() > f64::EPSILON {
                // Winding is determined by direction of traversal
                let winding = if l.p0.y > l.p1.y { 1 } else { -1 };
                let (top, bottom) = if l.p0.y < l.p1.y {
                    (l.p0, l.p1)
                } else {
                    (l.p1, l.p0)
                };
                self.edges.push(Edge {
                    shape_idx,
                    y_min: top.y,
                    y_max: bottom.y,
                    x: top.x,
                    slope_inv: (bottom.x - top.x) / (bottom.y - top.y),
                    winding,
                });
            }
        };

        let fill;
        match &shape.style {
            peniko::Style::Fill(f) => {
                fill = *f;
                shape_to_lines(shape_iter, params.affine, callback);
            }
            peniko::Style::Stroke(s) => {
                fill = Fill::NonZero;
                shape_to_lines(
                    kurbo::stroke(shape_iter, s, &StrokeOpts::default(), TOLERANCE),
                    params.affine,
                    callback,
                );
            }
        }
        self.shape_params.push(InternalShapeParams {
            color: params.color,
            fill,
        });
    }

    pub fn finish(mut self) -> Shapes {
        self.edges
            .sort_by(|a, b| (a.y_min, a.x).partial_cmp(&(b.y_min, b.x)).unwrap());

        Shapes {
            edges: self.edges,
            shape_params: self.shape_params,
            bbox: self.bbox,
        }
    }
}

pub struct Shapes {
    edges: Vec<Edge>,
    shape_params: Vec<InternalShapeParams>,
    bbox: Rect,
}

struct ShapeWindingState {
    shape_windings: Vec<i32>,
    visible_shapes: Vec<usize>,
}

impl ShapeWindingState {
    fn new(shape_params_len: usize) -> Self {
        Self {
            shape_windings: vec![0; shape_params_len],
            visible_shapes: Vec::with_capacity(shape_params_len),
        }
    }

    fn reset(&mut self) {
        self.shape_windings.iter_mut().for_each(|w| *w = 0);
        self.visible_shapes.clear();
    }

    fn contribute_winding(
        &mut self,
        winding: i32,
        shape_idx: usize,
        shape_params: &[InternalShapeParams],
    ) {
        if let Some(shape_winding) = self.shape_windings.get_mut(shape_idx) {
            *shape_winding += winding;

            let is_inside = shape_params
                .get(shape_idx)
                .map(|s| s.is_inside(*shape_winding))
                .unwrap_or_default();

            #[cfg(test)]
            assert!(shape_params.get(shape_idx).is_some());

            match self.visible_shapes.binary_search(&shape_idx) {
                Ok(pos) => {
                    if !is_inside {
                        self.visible_shapes.remove(pos);
                    }
                }
                Err(pos) => {
                    if is_inside {
                        self.visible_shapes.insert(pos, shape_idx);
                    }
                }
            }
        }
    }
}

impl Shape for Shapes {
    /// When the shape is drawn, it will be scaled to the
    /// size of `painter.bounds()` while maintaining aspect
    /// ratio. The shape will be placed in the top-left of
    /// the rectangle formed by `painter.bounds()`.
    fn draw(&self, painter: &mut Painter<'_, '_>) {
        let [y_bottom_bound, y_top_bound] = *painter.bounds().1;
        // Get the bounding box of the transformed path to iterate over
        let y_min = self.bbox.y0.max(y_bottom_bound).floor() as i64;
        let y_max = self.bbox.y1.min(y_top_bound).ceil() as i64;

        let mut active_edges: Vec<Edge> = Vec::new();
        let mut shape_windings = ShapeWindingState::new(self.shape_params.len());
        let mut edge_iter = self.edges.iter().copied().peekable();

        // Scanline algorithm - iterate over the path's y bounds
        for y in y_min..=y_max {
            let current_y = y as f64 + 0.5; // Sample at pixel center

            // A. Update X for existing edges (DDA step)
            for edge in &mut active_edges {
                edge.x += edge.slope_inv;
            }

            // B. Add new edges from GET that start at this scanline
            while let Some(edge) = edge_iter.peek()
                && edge.y_min <= current_y
            {
                // Calculate initial X for the 0.5 offset
                let mut e = edge_iter.next().unwrap();
                let dy = current_y - e.y_min;
                e.x += dy * e.slope_inv;
                active_edges.push(e);
            }

            // C. Remove edges that ended at or above this scanline
            active_edges.retain(|e| e.y_max > current_y);

            active_edges.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());

            shape_windings.reset();
            // 5. Fill between pairs
            for chunk in active_edges.windows(2) {
                let Edge {
                    shape_idx,
                    x: x_start,
                    winding,
                    ..
                } = chunk[0];
                let Edge { x: x_end, .. } = chunk[1];

                // Add this edge's winding contribution
                shape_windings.contribute_winding(winding, shape_idx, &self.shape_params);

                // Find the topmost shape we are inside of
                if let Some(current_shape) = shape_windings
                    .visible_shapes
                    .last()
                    .and_then(|s| self.shape_params.get(*s))
                {
                    // Fill integer cells from x_start to x_end
                    let x_min = (x_start - 0.5).ceil() as i64;
                    let x_max = (x_end - 0.5).floor() as i64;
                    let x_range = if ((x_max as f64 + 0.5) - x_end).abs() > f64::EPSILON {
                        x_min..=x_max
                    } else {
                        // We want to apply the winding number of
                        // the next edge to `current_winding` if it
                        // intersects with a point in our range
                        // (in this case the very last point)
                        x_min..=x_max - 1
                    };
                    // for x in x_range {
                    //     #[cfg(feature = "flip-y")]
                    //     let point = painter.get_point(x as f64, y_top_bound - y as f64);
                    //     #[cfg(not(feature = "flip-y"))]
                    //     let point = painter.get_point(x as f64, y as f64);
                    //     if let Some((grid_x, grid_y)) = point {
                    //         painter.paint(grid_x, grid_y, current_shape.color);
                    //     }
                    // }
                    #[cfg(feature = "flip-y")]
                    let line = painter.get_line(
                        *x_range.start() as f64,
                        *x_range.end() as f64,
                        y_top_bound - y as f64,
                    );
                    #[cfg(not(feature = "flip-y"))]
                    let line =
                        painter.get_line(*x_range.start() as f64, *x_range.end() as f64, y as f64);
                    if let Some((x0, x1, y)) = line {
                        painter.line(x0, x1, y, current_shape.color);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{Deref, Range};

    use peniko::{
        Fill,
        kurbo::{
            self, Affine, BezPath, PathSeg, Point, Rect, Shape as KurboShape, Stroke, StrokeOpts,
            Vec2,
        },
    };
    use ratatui::{Frame, Terminal, backend::TestBackend};
    use ratatui_core::style::Color;
    use ratatui_widgets::canvas::{Canvas, Painter, Shape};
    use rstest::rstest;

    use crate::{
        BezPathShape, RUST_R, ShapeBuilder, ShapeParams, TEST_STAR, TOLERANCE, shape_to_lines,
    };

    pub struct NaiveBezPathShape<S> {
        pub affine: Affine,
        pub shape: S,
        pub bbox: Rect,
        pub color: Color,
        pub style: peniko::Style,
    }

    impl<S> NaiveBezPathShape<S> {
        pub fn new(
            shape: S,
            bbox: Rect,
            style: peniko::Style,
            color: Color,
            affine: Affine,
        ) -> Self {
            Self {
                affine,
                shape,
                bbox,
                color,
                style,
            }
        }
    }

    impl<S: KurboShape> Shape for NaiveBezPathShape<S> {
        fn draw(&self, painter: &mut Painter<'_, '_>) {
            #[cfg(feature = "flip-y")]
            let y_top_bound = painter.bounds().1[1];

            let target_rect = self.affine.transform_rect_bbox(self.bbox);
            let mut new_path = Vec::new();
            let fill;
            let shape_iter = self.shape.path_elements(TOLERANCE);
            let callback = |l| new_path.push(PathSeg::Line(l));

            match &self.style {
                peniko::Style::Fill(f) => {
                    fill = *f;
                    shape_to_lines(shape_iter, self.affine, callback);
                }
                peniko::Style::Stroke(s) => {
                    fill = Fill::NonZero;
                    shape_to_lines(
                        kurbo::stroke(shape_iter, s, &StrokeOpts::default(), TOLERANCE),
                        self.affine,
                        callback,
                    );
                }
            }

            let path = BezPath::from_path_segments(new_path.into_iter());

            // Get the bounding box of the path to limit our search space
            let x_min = target_rect.x0.floor() as i64;
            let x_max = target_rect.x1.ceil() as i64;
            let y_min = target_rect.y0.floor() as i64;
            let y_max = target_rect.y1.ceil() as i64;
            // Iterate over the path's bounding box in canvas coordinates
            for y in y_min..=y_max {
                for x in x_min..=x_max {
                    let point = Point::new(x as f64 + 0.5, y as f64 + 0.5);

                    let is_inside = match fill {
                        Fill::NonZero => path.winding(point) != 0,
                        Fill::EvenOdd => (path.winding(point) % 2) != 0,
                    };

                    if is_inside {
                        #[cfg(feature = "flip-y")]
                        let point = painter.get_point(x as f64, y_top_bound - y as f64);
                        #[cfg(not(feature = "flip-y"))]
                        let point = painter.get_point(x as f64, y as f64);
                        if let Some((grid_x, grid_y)) = point {
                            painter.paint(grid_x, grid_y, self.color);
                        }
                    }
                }
            }
        }
    }

    fn ensure_matches_naive(
        resolution_range: Range<u16>,
        path: &BezPath,
        bbox: Rect,
        fill: peniko::Fill,
    ) {
        for width in resolution_range.clone() {
            for height in 0..64 {
                let mut naive_terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
                let mut scanline_terminal = Terminal::new(TestBackend::new(width, height)).unwrap();

                let transform = {
                    let width = width * 2;
                    let height = height * 4;

                    // 1. Calculate the scale needed to fit
                    let scale_x = width as f64 / bbox.width();
                    let scale_y = height as f64 / bbox.height();

                    Affine::translate(Vec2::new(-bbox.x0, -bbox.y0))
                        .then_scale_non_uniform(scale_x, scale_y)
                };

                naive_terminal
                    .draw(|f: &mut Frame| {
                        let area = f.area();
                        let canvas = Canvas::default()
                            // Each cell has a resolution of 2x4
                            .x_bounds([area.left() as f64 * 2.0, area.right() as f64 * 2.0])
                            .y_bounds([area.top() as f64 * 4.0, area.bottom() as f64 * 4.0])
                            .paint(|ctx| {
                                ctx.draw(&NaiveBezPathShape::new(
                                    path,
                                    bbox,
                                    peniko::Style::Fill(fill),
                                    Color::White,
                                    transform,
                                ));
                                ctx.draw(&NaiveBezPathShape::new(
                                    path,
                                    bbox,
                                    peniko::Style::Stroke(
                                        Stroke::new(1.0).with_join(peniko::kurbo::Join::Round),
                                    ),
                                    Color::White,
                                    transform,
                                ));
                            });
                        f.render_widget(canvas, area);
                    })
                    .unwrap();

                scanline_terminal
                    .draw(|f: &mut Frame| {
                        let area = f.area();
                        let mut builder = ShapeBuilder::new();
                        builder.draw(
                            &BezPathShape::with_bounding_box(path, bbox, peniko::Style::Fill(fill)),
                            ShapeParams {
                                color: Color::White,
                                affine: transform,
                            },
                        );
                        builder.draw(
                            &BezPathShape::with_bounding_box(
                                path,
                                bbox,
                                peniko::Style::Stroke(
                                    Stroke::new(1.0).with_join(peniko::kurbo::Join::Round),
                                ),
                            ),
                            ShapeParams {
                                color: Color::White,
                                affine: transform,
                            },
                        );
                        let shapes = builder.finish();
                        let canvas = Canvas::default()
                            // Each cell has a resolution of 2x4
                            .x_bounds([area.left() as f64 * 2.0, area.right() as f64 * 2.0])
                            .y_bounds([area.top() as f64 * 4.0, area.bottom() as f64 * 4.0])
                            .paint(|ctx| ctx.draw(&shapes));
                        f.render_widget(canvas, area);
                    })
                    .unwrap();

                scanline_terminal
                    .backend()
                    .assert_buffer(naive_terminal.backend().buffer());
            }
        }
    }

    #[rstest]
    #[case(0..2)]
    #[case(2..4)]
    #[case(4..6)]
    #[case(6..8)]
    #[case(8..10)]
    #[case(10..12)]
    #[case(12..14)]
    #[case(14..16)]
    #[case(16..18)]
    #[case(18..20)]
    #[case(20..22)]
    #[case(22..24)]
    #[case(24..26)]
    #[case(26..28)]
    #[case(28..30)]
    #[case(30..32)]
    #[case(32..34)]
    #[case(34..36)]
    #[case(36..38)]
    #[case(38..40)]
    #[case(40..42)]
    #[case(42..44)]
    #[case(44..46)]
    #[case(46..48)]
    #[case(48..50)]
    #[case(50..52)]
    #[case(52..54)]
    #[case(54..56)]
    #[case(56..58)]
    #[case(58..60)]
    #[case(60..62)]
    #[case(62..64)]
    fn ensure_rust_matches_naive(#[case] resolution_range: Range<u16>) {
        let path = RUST_R.deref();

        ensure_matches_naive(
            resolution_range,
            path,
            Rect::from_origin_size((0.0, 0.0), (106.0, 106.0)),
            Fill::NonZero,
        );
    }

    #[rstest]
    #[case(0..2)]
    #[case(2..4)]
    #[case(4..6)]
    #[case(6..8)]
    #[case(8..10)]
    #[case(10..12)]
    #[case(12..14)]
    #[case(14..16)]
    #[case(16..18)]
    #[case(18..20)]
    #[case(20..22)]
    #[case(22..24)]
    #[case(24..26)]
    #[case(26..28)]
    #[case(28..30)]
    #[case(30..32)]
    #[case(32..34)]
    #[case(34..36)]
    #[case(36..38)]
    #[case(38..40)]
    #[case(40..42)]
    #[case(42..44)]
    #[case(44..46)]
    #[case(46..48)]
    #[case(48..50)]
    #[case(50..52)]
    #[case(52..54)]
    #[case(54..56)]
    #[case(56..58)]
    #[case(58..60)]
    #[case(60..62)]
    #[case(62..64)]
    fn ensure_star_matches_naive(#[case] resolution_range: Range<u16>) {
        let path = TEST_STAR.deref();

        ensure_matches_naive(
            resolution_range,
            path,
            Rect::from_origin_size((0.0, 0.0), (100.0, 100.0)),
            Fill::NonZero,
        );
    }

    #[rstest]
    #[case(0..2)]
    #[case(2..4)]
    #[case(4..6)]
    #[case(6..8)]
    #[case(8..10)]
    #[case(10..12)]
    #[case(12..14)]
    #[case(14..16)]
    #[case(16..18)]
    #[case(18..20)]
    #[case(20..22)]
    #[case(22..24)]
    #[case(24..26)]
    #[case(26..28)]
    #[case(28..30)]
    #[case(30..32)]
    #[case(32..34)]
    #[case(34..36)]
    #[case(36..38)]
    #[case(38..40)]
    #[case(40..42)]
    #[case(42..44)]
    #[case(44..46)]
    #[case(46..48)]
    #[case(48..50)]
    #[case(50..52)]
    #[case(52..54)]
    #[case(54..56)]
    #[case(56..58)]
    #[case(58..60)]
    #[case(60..62)]
    #[case(62..64)]
    fn ensure_star_matches_naive_evenodd(#[case] resolution_range: Range<u16>) {
        let path = TEST_STAR.deref();

        ensure_matches_naive(
            resolution_range,
            path,
            Rect::from_origin_size((0.0, 0.0), (100.0, 100.0)),
            Fill::EvenOdd,
        );
    }
}
