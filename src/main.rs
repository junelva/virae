use wgpu::hal::Rect;
use winit::event::{Event, WindowEvent};

mod text;
mod window;

use crate::window::Context;

async fn run() {
    let (width, height) = (800, 600);
    let (event_loop, window, mut context) = Context::new("ivae", width, height).await;

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
            context.scale_factor * 2.0,
        );
    }

    // necessary to request initial frame on Wayland
    // otherwise no redraw requested, & no window shows
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
                        context.render();
                    }
                    WindowEvent::CloseRequested => target.exit(),
                    _ => {}
                }
            }
        })
        .unwrap();
}

fn main() {
    pollster::block_on(run());
}
