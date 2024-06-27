use std::error::Error;

use glam::{IVec2, UVec2, Vec2, Vec3};
use virae::types::{
    ComponentTransform, PixelRect, TextureSheetClusterDefinition, TextureSheetDefinition,
};
use virae::window::Context;
use virae::{Event, Vec4, WindowEvent};
use winit::event::{ElementState, KeyEvent};
use winit::event_loop::ControlFlow;

use winit::keyboard::{Key, NamedKey};

fn main() -> Result<(), Box<dyn Error>> {
    pollster::block_on(run())?;
    Ok(())
}

async fn run() -> Result<(), Box<dyn Error>> {
    let (width, height) = (800, 600);
    let (event_loop, window, mut context) =
        Context::new("testing", width, height, ControlFlow::Poll).await?;

    // initialize assets
    let (group_index, player_index) = {
        let shader_path = "examples/testing/shader.wgsl";
        context.file_watcher.add_path(shader_path);
        let config = context.config.lock().unwrap();
        let group_index = context.geos.new_unit_square(
            64,
            config.format,
            config.width,
            config.height,
            TextureSheetDefinition {
                path: "examples/shmuppin/shmuppin.png".to_string(),
                clusters: vec![TextureSheetClusterDefinition {
                    label: "tinyshmup".to_string(),
                    offset: UVec2::new(0, 0),
                    cluster_size: UVec2::new(128, 48),
                    sub_size: UVec2::new(16, 16),
                    spacing: UVec2::new(0, 0),
                }],
            },
            shader_path,
        )?;

        let player_index = context.geos.instance_groups[0].add_new(
            context.queue.clone(),
            ComponentTransform::unit_square_transform_from_pixel_rect(PixelRect {
                xy: IVec2::new(0, 0),
                wh: UVec2::new(64, 64),
                extent: UVec2::new(config.width, config.height),
            }),
            0,
            0,
            Vec4::new(1.0, 1.0, 1.0, 1.0),
        );
        (group_index, player_index)
    };

    struct DigitalInputState {
        up: bool,
        down: bool,
        left: bool,
        right: bool,
    }
    let mut digital_move = DigitalInputState {
        up: false,
        down: false,
        left: false,
        right: false,
    };
    let move_speed = 0.006;

    event_loop.run(move |event, target| {
        if let Event::WindowEvent {
            window_id: _,
            event,
        } = event
        {
            match event {
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            logical_key, state, ..
                        },
                    ..
                } => match logical_key.as_ref() {
                    Key::Named(NamedKey::ArrowUp) => {
                        if state == ElementState::Pressed {
                            digital_move.up = true;
                        } else if state == ElementState::Released {
                            digital_move.up = false;
                        }
                    }
                    Key::Named(NamedKey::ArrowDown) => {
                        if state == ElementState::Pressed {
                            digital_move.down = true;
                        } else if state == ElementState::Released {
                            digital_move.down = false;
                        }
                    }
                    Key::Named(NamedKey::ArrowLeft) => {
                        if state == ElementState::Pressed {
                            digital_move.left = true;
                        } else if state == ElementState::Released {
                            digital_move.left = false;
                        }
                    }
                    Key::Named(NamedKey::ArrowRight) => {
                        if state == ElementState::Pressed {
                            digital_move.right = true;
                        } else if state == ElementState::Released {
                            digital_move.right = false;
                        }
                    }
                    _ => (),
                },
                WindowEvent::Resized(size) => {
                    context.resize(size);
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => {
                    let mut move_impulse = Vec2::new(
                        if digital_move.left {
                            -1.0
                        } else if digital_move.right {
                            1.0
                        } else {
                            0.0
                        },
                        if digital_move.up {
                            -1.0
                        } else if digital_move.down {
                            1.0
                        } else {
                            0.0
                        },
                    );
                    move_impulse = move_impulse.normalize_or(Vec2::new(0.0, 0.0));
                    if move_impulse.length() > 0.0 {
                        context.geos.instance_groups[group_index]
                            .instance_buffer_manager
                            .data[player_index]
                            .translate(move_speed * Vec3::new(move_impulse.x, move_impulse.y, 0.0));
                    }
                    context.update().expect("event loop context update error");
                    context.render().expect("event loop context render error");
                }
                WindowEvent::CloseRequested => {
                    target.exit();
                }
                _ => {}
            }
            window.request_redraw();
        }
    })?;
    Ok(())
}
