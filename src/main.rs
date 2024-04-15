use geo::{ComponentTransform, PixelRect};
use glam::{Vec2, Vec4};
use wgpu::hal::Rect;
use winit::event::{Event, WindowEvent};

mod geo;
mod text;
mod window;

use crate::window::Context;

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let (width, height) = (800, 600);
    let (event_loop, window, mut context) = Context::new("virae", width, height).await;

    {
        let shader_path = "shaders/shader.wgsl";
        let config = context.config.lock().unwrap();
        context
            .geos
            .new_unit_square(config.format, config.width, config.height, shader_path);
        context.file_watcher.add_path(shader_path);

        // test labels
        for i in 0..8 {
            let x = 45.0 * i as f32;
            let y = 45.0 * i as f32;
            let w = 45.0;
            let h = 45.0;
            context.geos.instance_groups[0].add_new(
                context.queue.clone(),
                ComponentTransform::pixel_rect_to_screen_transform(PixelRect {
                    xy: Vec2::new(x, y) * context.scale_factor as f32,
                    wh: Vec2::new(w, h),
                    screen: Vec2::new(config.width as f32, config.height as f32),
                }),
                Vec4::new(0.1, 0.1, 0.2, 1.0),
            );
            context.texts.new_text(
                Rect {
                    x: x as f64 + 2.5,
                    y: y as f64 + 2.5,
                    w: w as f64,
                    h: h as f64,
                },
                "WWW WWW WWW WWW WWW WWW WWW",
                context.scale_factor,
                1.0,
            );
        }
    }

    // if event_loop's ControlFlow is not Poll, it's
    // necessary to request an initial frame on Wayland.
    // otherwise, no redraw is requested, and no window shows.
    window.request_redraw();

    event_loop
        .run(move |event, target| {
            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                match event {
                    WindowEvent::Resized(size) => {
                        context.resize(size);
                        window.request_redraw();
                    }
                    WindowEvent::RedrawRequested => {
                        context.update();
                        context.render();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                }
            }
        })
        .unwrap();
}
