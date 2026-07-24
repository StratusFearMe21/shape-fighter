use criterion::{Criterion, criterion_group, criterion_main};
use peniko::kurbo::{Affine, Circle, Rect, Vec2};
use ratatui_kurbo::{BezPathShape, ShapeBuilder, ShapeParams};
use ratatui_widgets::canvas::{Context, Painter, Shape};

pub fn criterion_benchmark(c: &mut Criterion) {
    let circles = [
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 4f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Red,
            Affine::translate(Vec2::new(30f64, 100f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 4f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Green,
            Affine::translate(Vec2::new(60f64, 80f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Blue,
            Affine::translate(Vec2::new(56.438568115234375f64, 13.491851806640625f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Green,
            Affine::translate(Vec2::new(69.0445785522461f64, 28.975610733032227f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Blue,
            Affine::translate(Vec2::new(88.50345611572266f64, 33.45977020263672f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Cyan,
            Affine::translate(Vec2::new(78.2309799194336f64, 50.55574035644531f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::White,
            Affine::translate(Vec2::new(88.5118637084961f64, 67.63716125488281f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Red,
            Affine::translate(Vec2::new(73.93488311767578f64, 81.18177032470703f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Cyan,
            Affine::translate(Vec2::new(88.50747680664063f64, 94.7728500366211f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Green,
            Affine::translate(Vec2::new(88.04633331298828f64, 114.73848724365234f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Red,
            Affine::translate(Vec2::new(88.50000762939453f64, 134.70864868164063f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Green,
            Affine::translate(Vec2::new(88.50137329101563f64, 154.68820190429688f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Red,
            Affine::translate(Vec2::new(85.36587524414063f64, 174.4217529296875f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Magenta,
            Affine::translate(Vec2::new(66.2637710571289f64, 180.30661010742188f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Gray,
            Affine::translate(Vec2::new(50.29082107543945f64, 192.32081604003906f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Green,
            Affine::translate(Vec2::new(47.57716369628906f64, 212.13145446777344f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Red,
            Affine::translate(Vec2::new(31.148771286010742f64, 200.72665405273438f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Gray,
            Affine::translate(Vec2::new(13.499495506286621f64, 191.3231658935547f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Cyan,
            Affine::translate(Vec2::new(16.236461639404297f64, 171.5194549560547f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Blue,
            Affine::translate(Vec2::new(13.498665809631348f64, 151.73158264160156f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Magenta,
            Affine::translate(Vec2::new(32.653076171875f64, 157.4846649169922f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Green,
            Affine::translate(Vec2::new(49.43903732299805f64, 168.35752868652344f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Yellow,
            Affine::translate(Vec2::new(50.98638153076172f64, 148.4221649169922f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Gray,
            Affine::translate(Vec2::new(50.502994537353516f64, 128.4364776611328f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Yellow,
            Affine::translate(Vec2::new(68.89531707763672f64, 120.5880126953125f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Red,
            Affine::translate(Vec2::new(50.995758056640625f64, 108.45869445800781f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Blue,
            Affine::translate(Vec2::new(32.841346740722656f64, 116.82579803466797f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::White,
            Affine::translate(Vec2::new(13.49681282043457f64, 111.78634643554688f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Blue,
            Affine::translate(Vec2::new(18.254947662353516f64, 92.39866638183594f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Yellow,
            Affine::translate(Vec2::new(13.491758346557617f64, 73.08140563964844f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Green,
            Affine::translate(Vec2::new(33.22671890258789f64, 76.21759796142578f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Gray,
            Affine::translate(Vec2::new(43.23773193359375f64, 58.91799545288086f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Yellow,
            Affine::translate(Vec2::new(30.81169319152832f64, 43.25325012207031f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Red,
            Affine::translate(Vec2::new(13.495077133178711f64, 53.1652717590332f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::White,
            Affine::translate(Vec2::new(13.499042510986328f64, 33.23836898803711f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Gray,
            Affine::translate(Vec2::new(16.110090255737305f64, 13.488845825195313f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Magenta,
            Affine::translate(Vec2::new(36.095054626464844f64, 13.497900009155273f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Magenta,
            Affine::translate(Vec2::new(48.9368782043457f64, 88.57662200927734f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Cyan,
            Affine::translate(Vec2::new(32.67744445800781f64, 137.4893035888672f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Magenta,
            Affine::translate(Vec2::new(13.523924827575684f64, 131.7548370361328f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Blue,
            Affine::translate(Vec2::new(33.87458038330078f64, 180.91650390625f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::White,
            Affine::translate(Vec2::new(69.38750457763672f64, 200.0570068359375f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Cyan,
            Affine::translate(Vec2::new(88.50032043457031f64, 194.16819763183594f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::White,
            Affine::translate(Vec2::new(69.38417053222656f64, 160.5581512451172f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::White,
            Affine::translate(Vec2::new(69.3786849975586f64, 140.56753540039063f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Gray,
            Affine::translate(Vec2::new(69.37635803222656f64, 100.6148681640625f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Magenta,
            Affine::translate(Vec2::new(58.72828674316406f64, 46.294342041015625f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Yellow,
            Affine::translate(Vec2::new(88.5038070678711f64, 13.495083808898926f64)),
        ),
        (
            BezPathShape::new(
                Circle::new((0f64, 0f64), 10f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::Cyan,
            Affine::translate(Vec2::new(46.296993255615234f64, 30.67096710205078f64)),
        ),
    ];
    let rectangles = [
        (
            BezPathShape::new(
                Rect::new(-100f64, -1.5f64, 100f64, 1.5f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::White,
            Affine::translate(Vec2::new(100f64, 2f64)),
        ),
        (
            BezPathShape::new(
                Rect::new(-1.5f64, -100f64, 1.5f64, 100f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::White,
            Affine::translate(Vec2::new(2f64, 100f64)),
        ),
        (
            BezPathShape::new(
                Rect::new(-1.5f64, -100f64, 1.5f64, 100f64),
                peniko::Style::Fill(peniko::Fill::NonZero),
            ),
            ratatui::style::Color::White,
            Affine::translate(Vec2::new(100f64, 100f64)),
        ),
    ];

    let area = ratatui::layout::Rect::new(0, 0, 91, 57);
    let mut ctx = Context::new(
        area.width,
        area.height,
        [area.left() as f64 * 2.0, area.right() as f64 * 2.0],
        [area.top() as f64 * 4.0, area.bottom() as f64 * 4.0],
        ratatui::symbols::Marker::Braille,
    );

    c.bench_function("Builder", |b| {
        // bez_path_shape
        //     .path
        //     .apply_affine(Affine::rotate_about(0.1, path_center));
        b.iter(|| {
            let mut builder = ShapeBuilder::new();
            for (circle, color, transform) in &circles {
                builder.draw(
                    circle,
                    ShapeParams {
                        color: *color,
                        affine: *transform,
                    },
                );
            }
            for (rectangle, color, transform) in &rectangles {
                builder.draw(
                    rectangle,
                    ShapeParams {
                        color: *color,
                        affine: *transform,
                    },
                );
            }
            builder.finish()
        })
    });

    c.bench_function("Rasterizer", |b| {
        let mut builder = ShapeBuilder::new();
        for (circle, color, transform) in &circles {
            builder.draw(
                circle,
                ShapeParams {
                    color: *color,
                    affine: *transform,
                },
            );
        }
        for (rectangle, color, transform) in &rectangles {
            builder.draw(
                rectangle,
                ShapeParams {
                    color: *color,
                    affine: *transform,
                },
            );
        }
        let shapes = builder.finish();
        let mut painter = Painter::from(&mut ctx);
        b.iter(|| shapes.draw(&mut painter));
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
