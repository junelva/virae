use std::error::Error;

use glam::UVec2;
use virae::types::{
    ComponentTransform, PixelRect, TextureSheetClusterDefinition, TextureSheetDefinition,
};
use virae::window::Context;
use virae::{Event, HalRect, Vec4, WindowEvent};

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
            64,
            config.format,
            config.width,
            config.height,
            // TextureSheetDefinition::none(),
            TextureSheetDefinition {
                // this spritesheet is 32x32 sprites with outer border of 4px, 1px between, 8px btw l/r halves.
                path: "examples/testing/buch_match3.png".to_string(),
                clusters: vec![TextureSheetClusterDefinition {
                    label: "match3".to_string(),
                    offset: UVec2::new(4, 4),
                    cluster_size: UVec2::new(197, 296),
                    sub_size: UVec2::new(32, 32),
                    spacing: UVec2::new(1, 1),
                }],
            },
            shader_path,
        )?;

        // test labels
        let mut y_count: i32 = -1;
        for i in 0..54 {
            let w = 64;
            let h = 64;
            let x_index = i % 6;
            let x_offset = (w + 8) * x_index;
            if x_index == 0 {
                y_count += 1;
            }
            let y_offset = (h + 8_u32) * y_count as u32;
            let x = 8 + x_offset;
            let y = 8 + y_offset;
            context.geos.instance_groups[0].add_new(
                context.queue.clone(),
                ComponentTransform::unit_square_transform_from_pixel_rect(PixelRect {
                    xy: UVec2::new(x, y),
                    wh: UVec2::new(w, h),
                    extent: UVec2::new(config.width, config.height),
                }),
                0,
                i as usize,
                Vec4::new(1.0, 1.0, 1.0, 1.0),
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
                1.0,
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
