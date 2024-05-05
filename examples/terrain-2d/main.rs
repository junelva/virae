use glam::{IVec2, UVec2};
use rand::Rng;
use std::error::Error;
use std::sync::{Arc, Mutex};
use virae::geo::GeoInstances;
use virae::types::{
    ComponentTransform, PixelRect, TextureSheetClusterDefinition, TextureSheetDefinition,
};
use virae::window::Context;
use virae::{Event, Vec4, WindowEvent};
use wgpu::Queue;
use winit::event_loop::ControlFlow;

fn main() -> Result<(), Box<dyn Error>> {
    pollster::block_on(run())?;
    Ok(())
}

#[derive(Copy, Clone)]
struct TerrainBlock {
    cluster: usize,
    sub_variants: UVec2,
}

const DIRT_INTERIOR: TerrainBlock = TerrainBlock {
    cluster: 1,
    sub_variants: UVec2::new(0, 4),
};

struct Terrain<'a> {
    screen_size: UVec2,
    queue: Arc<Mutex<Queue>>,
    geo: &'a mut GeoInstances,
    xy: UVec2,
    wh: UVec2,
    block_size: UVec2,
    blocks: Vec<TerrainBlock>,
}

impl Terrain<'_> {
    fn fill_with_block(&mut self, block: &TerrainBlock, zoom: u32) {
        for x in 0..self.wh.x {
            for y in 0..self.wh.y {
                self.geo.add_new(
                    self.queue.clone(),
                    ComponentTransform::unit_square_transform_from_pixel_rect(PixelRect {
                        xy: IVec2::new(
                            (self.xy.x + self.block_size.x * x) as i32,
                            (self.xy.y + self.block_size.y * y) as i32,
                        ) * zoom as i32,
                        wh: UVec2::new(self.block_size.x, self.block_size.y) * zoom,
                        extent: UVec2::new(self.screen_size.x, self.screen_size.y),
                    }),
                    block.cluster,
                    rand::thread_rng()
                        .gen_range(block.sub_variants.x as usize..block.sub_variants.y as usize),
                    Vec4::new(1.0, 1.0, 1.0, 1.0),
                );
                self.blocks.push(*block);
            }
        }
    }
}

// fn terrain_test(queue: Arc<Mutex<Queue>>, geo: &mut GeoInstances, screen_size: (u32, u32)) {
//     geo.add_new(
//         queue,
//         ComponentTransform::unit_square_transform_from_pixel_rect(PixelRect {
//             xy: UVec2::new(100, 100),
//             wh: UVec2::new(16, 16),
//             extent: UVec2::new(screen_size.0, screen_size.1),
//         }),
//         1,
//         1,
//         Vec4::new(1.0, 1.0, 1.0, 1.0),
//     );
// }

async fn run() -> Result<(), Box<dyn Error>> {
    let (width, height) = (800, 600);
    let (event_loop, window, mut context) =
        Context::new("testing", width, height, ControlFlow::Wait).await?;

    {
        let shader_path = "examples/testing/shader.wgsl";
        context.file_watcher.add_path(shader_path);
        let config = context.config.lock().unwrap();
        context.geos.new_unit_square(
            64,
            config.format,
            config.width,
            config.height,
            TextureSheetDefinition {
                path: "examples/terrain-2d/terrain-2d.png".to_string(),
                clusters: vec![
                    TextureSheetClusterDefinition {
                        label: "dirt-exterior".to_string(),
                        offset: UVec2::new(0, 0),
                        cluster_size: UVec2::new(32, 32),
                        sub_size: UVec2::new(8, 8),
                        spacing: UVec2::new(0, 0),
                    },
                    TextureSheetClusterDefinition {
                        label: "dirt-interior".to_string(),
                        offset: UVec2::new(0, 32),
                        cluster_size: UVec2::new(16, 16),
                        sub_size: UVec2::new(8, 8),
                        spacing: UVec2::new(0, 0),
                    },
                    TextureSheetClusterDefinition {
                        label: "stone-exterior".to_string(),
                        offset: UVec2::new(33, 0),
                        cluster_size: UVec2::new(32, 32),
                        sub_size: UVec2::new(8, 8),
                        spacing: UVec2::new(0, 0),
                    },
                    TextureSheetClusterDefinition {
                        label: "stone-interior".to_string(),
                        offset: UVec2::new(16, 32),
                        cluster_size: UVec2::new(16, 16),
                        sub_size: UVec2::new(8, 8),
                        spacing: UVec2::new(0, 0),
                    },
                    TextureSheetClusterDefinition {
                        label: "roots-exterior".to_string(),
                        offset: UVec2::new(64, 0),
                        cluster_size: UVec2::new(48, 48),
                        sub_size: UVec2::new(8, 8),
                        spacing: UVec2::new(0, 0),
                    },
                    TextureSheetClusterDefinition {
                        label: "roots-interior".to_string(),
                        offset: UVec2::new(32, 32),
                        cluster_size: UVec2::new(16, 16),
                        sub_size: UVec2::new(8, 8),
                        spacing: UVec2::new(0, 0),
                    },
                ],
            },
            shader_path,
        )?;

        let mut terrain = Terrain {
            screen_size: UVec2::new(width, height),
            queue: context.queue.clone(),
            geo: &mut context.geos.instance_groups[0],
            xy: UVec2::new(32, 32),
            wh: UVec2::new(8, 8),
            block_size: UVec2::new(8, 8),
            blocks: vec![],
        };

        terrain.fill_with_block(&DIRT_INTERIOR, 4)

        // terrain_test(
        //     context.queue.clone(),
        //     &mut context.geos.instance_groups[0],
        //     (width, height),
        // );
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
