use glam::{Mat4, Vec3, Vec4};
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

        // create test squares
        context.geos.instance_groups[0].add_new(
            context.queue.clone(),
            Mat4::IDENTITY,
            Vec4::new(0.2, 0.2, 0.2, 1.0),
        );
        context.geos.instance_groups[0].add_new(
            context.queue.clone(),
            Mat4::IDENTITY + Mat4::from_translation(Vec3::new(0.25, 0.25, 0.0)),
            Vec4::new(0.2, 0.2, 0.8, 1.0),
        );
    }

    // test text strings
    for i in 0..10 {
        context.texts.new_text(
            Rect {
                x: 20.0 * i as f64,
                y: 30.0 * i as f64,
                w: width as f64,
                h: height as f64,
            },
            "This is a test string.",
            context.scale_factor,
            1.0,
        );
    }

    // if event_loop's ControlFlow is not Poll, it's
    // necessary to request initial frame on Wayland
    // otherwise no redraw requested; no window shows.
    // window.request_redraw();

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
