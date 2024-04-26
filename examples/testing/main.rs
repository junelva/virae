use std::error::Error;

use glam::UVec2;
use virae::types::{
    ComponentTransform, PixelRect, TextureSheetClusterDefinition, TextureSheetDefinition,
};
use virae::window::Context;
use virae::{Event, HalRect, Vec2, Vec4, WindowEvent};

fn main() -> Result<(), Box<dyn Error>> {
    pollster::block_on(run())?;
    Ok(())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let (width, height) = (800, 600);
    let (event_loop, window, mut context) = Context::new("testing", width, height).await?;

    {
        let shader_path = "examples/testing/shader.wgsl";
        context.file_watcher.add_path(shader_path);
        let config = context.config.lock().unwrap();
        context.geos.new_unit_square(
            16,
            config.format,
            config.width,
            config.height,
            // TextureSheetDefinition::none(),
            TextureSheetDefinition {
                // this spritesheet is 32x32 sprites with outer border of 4px, 1px between, 8px btw l/r halves.
                path: "examples/testing/buch_match3.png".to_string(),
                clusters: vec![TextureSheetClusterDefinition {
                    label: "match3".to_string(),
                    size: UVec2::new(1, 1),
                    offset: UVec2::new(0, 0),
                    spacing: 0,
                }],
            },
            shader_path,
        )?;

        // test labels
        for i in 0..4 {
            let x = 8.0 + 132.0 * i as f32;
            let y = 8.0;
            let w = 128.0;
            let h = 128.0;
            context.geos.instance_groups[0].add_new(
                context.queue.clone(),
                ComponentTransform::pixel_rect_to_screen_transform(PixelRect {
                    xy: Vec2::new(x, y) * context.scale_factor as f32,
                    wh: Vec2::new(w, h),
                    screen: Vec2::new(config.width as f32, config.height as f32),
                }),
                Vec4::new(0.4, 0.8, 0.4, 1.0),
            );
            context.texts.new_text(
                HalRect {
                    x: x as f64 + 2.5,
                    y: y as f64 + 2.5,
                    w: w as f64,
                    h: h as f64,
                },
                format!("tx{}", i).as_str(),
                context.scale_factor,
                3.0,
            );
        }
    }

    // if event_loop's ControlFlow is not Poll, it's
    // necessary to request an initial frame on Wayland.
    // otherwise, no redraw is requested, and no window shows.
    window.request_redraw();

    event_loop.run(move |event, target| {
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
                    // it is unfortunate the errors from these functions
                    // don't ?-bubble out of this closure. todo, find a way...
                    context.update().expect("event loop context update error");
                    context.render().expect("event loop context render error");
                }
                WindowEvent::CloseRequested => {
                    target.exit();
                }
                _ => (),
            }
        }
    })?;
    Ok(())
}
