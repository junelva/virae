#![allow(dead_code)]

use crate::types::{
    ComponentTransform, GeoUniformMatrix, GeoUniformVec2, InstanceBufferManager,
    RenderPipelineRecord, TextureSheet, TextureSheetDefinition, UNIT_SQUARE_BUFFER_LAYOUT,
    UNIT_SQUARE_INDICES, UNIT_SQUARE_VERTICES,
};
use std::borrow::Cow;
use std::fs::read_to_string;
use std::mem::size_of;
use std::sync::{Arc, Mutex};
use wgpu::util::{BufferInitDescriptor, DeviceExt};

use glam::{Mat4, Vec2, Vec4};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    Buffer, BufferBindingType, BufferSize, BufferUsages, Device, Face, FragmentState,
    MultisampleState, PrimitiveState, Queue, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, TextureFormat, VertexState,
};

// various things needed to render geometry.
pub struct GeoInstances {
    pub render_pipeline_record: RenderPipelineRecord,
    pub bind_group: BindGroup,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub texture_sheet: Option<TextureSheet>,
    pub view_matrix_uniform: GeoUniformMatrix,
    pub screen_size_uniform: GeoUniformVec2,
    pub instance_buffer_manager: InstanceBufferManager,
}

impl GeoInstances {
    pub fn add_new(
        &mut self,
        queue: Arc<Mutex<Queue>>,
        transform: ComponentTransform,
        color: Vec4,
    ) {
        self.instance_buffer_manager
            .add_instance(queue, None, transform, color);
    }

    pub fn recalc_screen_instances(&mut self, queue: Arc<Mutex<Queue>>, screen: Vec2) {
        self.instance_buffer_manager
            .recalc_screen_instances(queue, screen);
    }
}

pub struct GeoManager {
    pub device: Arc<Mutex<Device>>,
    pub format: TextureFormat,
    pub instance_groups: Vec<GeoInstances>,
}

impl GeoManager {
    pub fn new(device: Arc<Mutex<Device>>, format: TextureFormat) -> Self {
        Self {
            device,
            format,
            instance_groups: vec![],
        }
    }

    pub fn num_instances(&self, group_index: usize) -> u32 {
        self.instance_groups[group_index]
            .instance_buffer_manager
            .data
            .len() as u32
    }

    pub fn update_view(&mut self, queue: Arc<Mutex<Queue>>, width: u32, height: u32) {
        let queue = queue.lock().unwrap();
        let view_matrix = Mat4::orthographic_lh(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0);
        let screen_size = Vec2::new(width as f32, height as f32);
        for ig in self.instance_groups.iter_mut() {
            ig.view_matrix_uniform.matrix = view_matrix;
            queue.write_buffer(
                &ig.view_matrix_uniform.buffer,
                0,
                bytemuck::cast_slice(&[view_matrix]),
            );
            ig.screen_size_uniform.vec = screen_size;
            queue.write_buffer(
                &ig.screen_size_uniform.buffer,
                0,
                bytemuck::cast_slice(&[screen_size]),
            );
        }
    }

    pub fn reload_shader(&mut self, device: Arc<Mutex<wgpu::Device>>, shader_path: &str) {
        let device = device.lock().unwrap();

        // for every instance group...
        for ig in self.instance_groups.iter_mut() {
            // does the instance group use this shader path?
            if ig.render_pipeline_record.shader_path == *shader_path.to_string() {
                // if so, rebuild the shader.
                ig.render_pipeline_record.shader_module =
                    device.create_shader_module(ShaderModuleDescriptor {
                        label: Some(&*format!("shader {}", shader_path)),
                        source: ShaderSource::Wgsl(Cow::Borrowed(
                            &*read_to_string(shader_path).unwrap(),
                        )),
                    });

                // and rebuild the render pipeline.
                ig.render_pipeline_record.render_pipeline =
                    device.create_render_pipeline(&RenderPipelineDescriptor {
                        label: Some(&*format!("pipeline {}", shader_path)),
                        layout: Some(&ig.render_pipeline_record.pipeline_layout),
                        vertex: VertexState {
                            module: &ig.render_pipeline_record.shader_module,
                            entry_point: "vs_main",
                            buffers: &UNIT_SQUARE_BUFFER_LAYOUT,
                        },
                        fragment: Some(FragmentState {
                            module: &ig.render_pipeline_record.shader_module,
                            entry_point: "fs_main",
                            targets: &[Some(ig.render_pipeline_record.format.into())],
                        }),
                        primitive: PrimitiveState {
                            cull_mode: None,
                            ..Default::default()
                        },
                        depth_stencil: None,
                        multisample: MultisampleState::default(),
                        multiview: None,
                    });
            }
        }
    }

    pub fn new_unit_square(
        &mut self,
        max_instances: usize,
        format: TextureFormat,
        width: u32,
        height: u32,
        _sheet_data: Option<TextureSheetDefinition>,
        shader_path: &str,
    ) {
        let device = self.device.lock().unwrap();

        // shader
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("test shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(&*read_to_string(shader_path).unwrap())),
        });

        // vertex and index buffers, layout
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("unit square vertices"),
            contents: bytemuck::cast_slice(&UNIT_SQUARE_VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("unit square indices"),
            contents: bytemuck::cast_slice(&UNIT_SQUARE_INDICES),
            usage: BufferUsages::INDEX,
        });

        // bind groups and pipeline setup

        // view matrix uniform setup
        let view_matrix = Mat4::orthographic_lh(-1.0, 1.0, -1.0, 1.0, -1.0, 1.0);
        let view_matrix_uniform = GeoUniformMatrix {
            matrix: view_matrix,
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("unit square view_matrix"),
                contents: bytemuck::cast_slice(&[view_matrix]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }),
        };

        // screen size uniform setup
        let screen_size = Vec2 {
            x: width as f32,
            y: height as f32,
        };
        let screen_size_uniform = GeoUniformVec2 {
            vec: screen_size,
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("unit square screen_res"),
                contents: bytemuck::cast_slice(&[screen_size]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }),
        };

        // bind group layout and bind group creation
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new((size_of::<Mat4>()) as u64),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new((size_of::<Vec2>()) as u64),
                    },
                    count: None,
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: view_matrix_uniform.buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: screen_size_uniform.buffer.as_entire_binding(),
                },
            ],
            label: None,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // pipeline itself, with necessary components for reconstruction retained.
        let render_pipeline_record = RenderPipelineRecord {
            render_pipeline: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("unit square pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &UNIT_SQUARE_BUFFER_LAYOUT,
                },
                fragment: Some(FragmentState {
                    module: &shader_module,
                    entry_point: "fs_main",
                    targets: &[Some(format.into())],
                }),
                primitive: PrimitiveState {
                    cull_mode: Some(Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
            }),
            pipeline_layout,
            shader_module,
            shader_path: shader_path.to_string(),
            format,
        };

        // drop device here because it's used to make the instance buffer below.
        drop(device);

        self.instance_groups.push(GeoInstances {
            render_pipeline_record,
            bind_group,
            vertex_buffer,
            index_buffer,
            texture_sheet: None,
            view_matrix_uniform,
            screen_size_uniform,
            instance_buffer_manager: InstanceBufferManager::new(max_instances, self.device.clone()),
        });
    }
}
