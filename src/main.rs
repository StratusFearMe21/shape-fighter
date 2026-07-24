use std::{
    iter::{Copied, Cycle},
    ops::Range,
};

use avian2d::prelude::*;
use bevy::{prelude::*, scene::ScenePlugin};
use bevy_ratatui::{
    RatatuiContext, RatatuiPlugins,
    event::{KeyMessage, ResizeMessage},
};
use rand::RngExt;
use ratatui::{
    crossterm::event::KeyCode,
    layout::Size,
    style::Color,
    widgets::canvas::{self, Canvas, Painter},
};
use ratatui_kurbo::{
    BezPathShape, ShapeBuilder,
    kurbo::{self, Affine, Circle, Triangle, Vec2},
    peniko,
};

mod bloom_colors;
mod shape;

use crate::bloom_colors::RatatuiBloomColors;
use crate::shape::RatatuiShape;

#[derive(Component, Clone, Copy)]
#[repr(transparent)]
struct RatatuiColor(Color);

#[derive(Resource)]
#[repr(transparent)]
struct RatatuiColorIter(Copied<Cycle<std::slice::Iter<'static, RatatuiColor>>>);

fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(bevy::app::ScheduleRunnerPlugin::run_loop(
                std::time::Duration::from_secs_f32(1. / 60.),
            )),
            RatatuiPlugins {
                enable_kitty_protocol: true,
                enable_mouse_capture: true,
                enable_input_forwarding: true,
            },
            AssetPlugin::default(),
            ScenePlugin::default(),
            PhysicsPlugins::default(),
        ))
        .insert_resource(Gravity(bevy::math::Vec2::ZERO))
        .add_systems(PostStartup, setup)
        .add_systems(PreUpdate, input_system)
        .add_systems(FixedPreUpdate, particle_system)
        .add_systems(
            Update,
            (
                resize_system.run_if(on_message::<ResizeMessage>),
                draw_system,
            ),
        )
        .run();
}

#[derive(Component, Clone, Copy)]
struct Map;

#[derive(Component, Clone, Copy)]
struct Player;

fn map_shape(size: Size) -> (Collider, Transform, RatatuiShape) {
    let map_radius = size.width.min(size.height) as f32 / 2.0 - 16.0;
    (
        Collider::circle(map_radius),
        Transform::from_xyz(size.width as f32 / 2.0, size.height as f32 / 2.0, 0.0),
        RatatuiShape::Circle(BezPathShape::new(
            Circle::new((0.0, 0.0), map_radius as f64),
            peniko::Style::Stroke(kurbo::Stroke::new(2.0)),
        )),
    )
}

fn setup(mut commands: Commands, context: Res<RatatuiContext>) {
    let size = context.size().unwrap_or(Size::new(185, 57));
    let size = Size::new(size.width * 2, size.height * 4);
    let (collider, transform, shape) = map_shape(size);
    commands.spawn((
        // RigidBody::Static,
        collider,
        transform,
        RatatuiColor(Color::White),
        shape,
        Map,
    ));
    let triangle = Triangle::EQUILATERAL.inflate(10.0);
    commands.spawn((
        RigidBody::Dynamic,
        Collider::triangle(
            vec2(triangle.a.x as f32, triangle.a.y as f32),
            vec2(triangle.b.x as f32, triangle.b.y as f32),
            vec2(triangle.c.x as f32, triangle.c.y as f32),
        ),
        transform,
        RatatuiColor(Color::White),
        LinearDamping(0.5),
        AngularVelocity::ZERO,
        LinearVelocity::ZERO,
        RatatuiShape::Triangle(BezPathShape::new(
            triangle,
            peniko::Style::Stroke(kurbo::Stroke::new(1.0)),
        )),
        Player,
    ));
}

struct CircleIter(canvas::Circle, Range<i32>);

impl CircleIter {
    fn new(circle: canvas::Circle) -> Self {
        Self(circle, 0..360)
    }
}

impl Iterator for CircleIter {
    type Item = (f64, f64);

    fn next(&mut self) -> Option<Self::Item> {
        let angle = self.1.next()?;
        let radians = f64::from(angle).to_radians();
        let circle_x = self.0.radius.mul_add(radians.cos(), self.0.x);
        let circle_y = self.0.radius.mul_add(radians.sin(), self.0.y);
        Some((circle_x, circle_y))
    }
}

fn draw_system(
    mut context: ResMut<RatatuiContext>,
    particles: Query<(&Particle, &Transform, &RatatuiBloomColors, &RatatuiColor)>,
    shapes: Query<(&RatatuiShape, &Transform, &RatatuiColor)>,
    map: Single<(&Collider, &Transform), With<Map>>,
) -> Result {
    context.draw(|frame| {
        let area = frame.area();
        let x_bounds = [0.0, area.width as f64 * 2.0];
        let y_bounds = [0.0, area.height as f64 * 4.0];
        let (map_collider, map_transform) = map.into_inner();

        // let x_bounds = [0.0, area.width as f64];
        // let y_bounds = [0.0, area.height as f64 * 2.0];

        let particle_canvas = Canvas::default()
            // .marker(ratatui::symbols::Marker::HalfBlock)
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .paint(|ctx| {
                ctx.draw(&ratatui::widgets::canvas::Points::new(&[], Color::White));
                let mut painter = Painter::from(ctx);
                for (_, transform, _, color) in &particles {
                    if map_collider.contains_point(
                        Position::new(bevy::prelude::Vec2::new(
                            map_transform.translation.x as f32,
                            map_transform.translation.y as f32,
                        )),
                        0.0,
                        transform.translation.xy(),
                    ) {
                        if let Some((x, y)) = painter.get_point(
                            transform.translation.x as f64,
                            transform.translation.y as f64,
                        ) {
                            painter.paint(x, y, color.0);
                            painter.paint(x, y.saturating_add(1), color.0);
                            painter.paint(x, y.saturating_sub(1), color.0);
                            // painter.paint(x.saturating_add(1), y, color.0);
                            // painter.paint(x.saturating_sub(1), y, color.0);
                        }
                    }
                }
            });
        frame.render_widget(particle_canvas, area);

        let mut builder = ShapeBuilder::new();
        for (shape, transform, color) in shapes {
            // let text = Text::raw("hello world\npress 'q' to quit");
            // let color: Color = (*color).into();
            let Mat3 { x_axis, y_axis, .. } = Mat3::from_quat(transform.rotation);
            shape.draw(
                &mut builder,
                color.0,
                Affine::new([
                    x_axis.x as f64,
                    x_axis.y as f64,
                    y_axis.x as f64,
                    y_axis.y as f64,
                    0.0,
                    0.0,
                ])
                .then_translate(Vec2::new(
                    transform.translation.x as f64,
                    transform.translation.y as f64,
                )),
            );
        }

        let shapes = builder.finish();
        let canvas = Canvas::default()
            // .marker(ratatui::symbols::Marker::HalfBlock)
            .x_bounds(x_bounds)
            .y_bounds(y_bounds)
            .paint(|ctx| ctx.draw(&shapes));
        frame.render_widget(canvas, area);

        // {
        //     let mut ctx = canvas::Context::new(
        //         area.width,
        //         area.height,
        //         [0.0, area.width as f64],
        //         [0.0, area.height as f64],
        //         Marker::Block,
        //     );
        //     let painter = Painter::from(&mut ctx);
        //     let buffer = frame.buffer_mut();
        //     for (_, transform, bloom_colors, _) in &particles {
        //         // let bloom_color = bloom_colors.0[0];
        //         let last_bloom_color = bloom_colors.0.len() - 1;

        //         // if map_collider.contains_point(
        //         //     Position::new(bevy::prelude::Vec2::new(
        //         //         map_transform.translation.x as f32,
        //         //         map_transform.translation.y as f32,
        //         //     )),
        //         //     0.0,
        //         //     transform.translation.xy(),
        //         // ) {
        //         // for circle_radius in (0..bloom_colors.0.len()).rev() {
        //         for (idx, color) in bloom_colors.0.iter().enumerate().rev() {
        //             let circle_radius = idx * 1;
        //             for circle_radius_extra in 0..1 {
        //                 for point in CircleIter::new(canvas::Circle {
        //                     x: transform.translation.x as f64 / 2.0,
        //                     y: transform.translation.y as f64 / 4.0,
        //                     radius: (circle_radius_extra + circle_radius) as f64,
        //                     color: Color::Reset,
        //                 }) {
        //                     if let Some((x, y)) = painter.get_point(point.0, point.1)
        //                         && let Some(cell) = buffer.cell_mut((x as u16, y as u16))
        //                     {
        //                         if cell.bg != Color::Reset {
        //                             if let Some(bloom_color_idx) =
        //                                 bloom_colors.0.iter().position(|p| *p == cell.bg)
        //                                 && bloom_color_idx != last_bloom_color
        //                             {
        //                                 cell.bg = bloom_colors.0
        //                                     [(bloom_color_idx + 1).min(last_bloom_color)];
        //                                 // cell.bg = bloom_color;
        //                                 // cell.bg = *color;
        //                             }
        //                         } else {
        //                             // cell.bg = bloom_color;
        //                             cell.bg = *color;
        //                         }
        //                     }
        //                 }
        //             }
        //         }
        //         // }
        //     }
        // }
    })?;

    Ok(())
}

#[derive(Component, Clone, Copy)]
struct Particle {}

fn resize_system(
    mut commands: Commands,
    context: Res<RatatuiContext>,
    particles: Query<(Entity, &Particle, &mut Transform), Without<Map>>,
    map: Single<(&mut Collider, &mut Transform, &mut RatatuiShape), With<Map>>,
) {
    let size = context.size().unwrap_or(Size::new(185, 57));
    let size = Size::new(size.width * 2, size.height * 4);

    let (mut collider, mut transform, mut shape) = map.into_inner();
    let (new_collider, new_transform, new_shape) = map_shape(size);

    *collider = new_collider;
    *transform = new_transform;
    *shape = new_shape;

    for (entity, _, _) in particles {
        commands.entity(entity).despawn();
    }
}

fn particle_system(
    mut commands: Commands,
    context: Res<RatatuiContext>,
    particles: Query<(&Particle, &mut Transform)>,
) {
    let mut rng = rand::rng();
    let size = context.size().unwrap_or(Size::new(185, 57));
    let size = Size::new(size.width * 2, size.height * 4);

    let mut particles_saturated = false;

    for (_, mut transform) in particles {
        transform.translation.y += 3.0;

        if rand::random_bool(0.5) {
            transform.translation.x += 1.0;
        } else {
            transform.translation.x -= 1.0;
        }

        if transform.translation.y > size.height as f32 {
            transform.translation.y = 0.0;
            particles_saturated = true;
        }
    }

    if !particles_saturated {
        for _ in 0..5 {
            // if rand::random_bool(0.05) {
            commands.spawn((
                Transform::from_xyz(rng.random_range(0.0..size.width as f32), 0.0, 0.0),
                // Transform::from_xyz(*column as f32, *row as f32 * 2.0, 0.0),
                RatatuiColor(Color::Cyan),
                RatatuiBloomColors(&[
                    Color::Indexed(53),
                    Color::Indexed(54),
                    Color::Indexed(91),
                    Color::Indexed(127),
                    Color::Indexed(163),
                    Color::Indexed(199),
                    Color::Indexed(205),
                    Color::Indexed(212),
                    Color::Indexed(218),
                    Color::Indexed(225),
                    Color::Indexed(231),
                ]),
                Particle {},
            ));
        }
        // }
    }
}

fn input_system(
    mut messages: MessageReader<KeyMessage>,
    mut exit: MessageWriter<AppExit>,
    player: Single<(&mut LinearVelocity, &mut AngularVelocity), With<Player>>,
) {
    let (mut linear_velocity, mut angular_velocity) = player.into_inner();
    for message in messages.read() {
        if message.is_release() {
            continue;
        }
        if let KeyCode::Char('q') = message.code {
            exit.write_default();
        }
        if let KeyCode::Char(' ') = message.code {
            linear_velocity.y += 25.0;
            angular_velocity.0 += 1.0;
        }
    }
}
