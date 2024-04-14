use std::borrow::Cow;
use std::fs::read_to_string;
use std::mem::size_of;
use std::sync::{Arc, Mutex};

use bytemuck::{ByteEq, ByteHash, Pod, Zeroable};
use glam::{Mat4, Vec4};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    Buffer, BufferAddress, BufferBindingType, BufferSize, BufferUsages, Device, Face,
    FragmentState, MultisampleState, PipelineLayout, PrimitiveState, Queue, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

const UNIT_SQUARE_VERTICES: [Vec4; 4] = [
    Vec4::new(-0.5, 0.5, 0.0, 1.0),
    Vec4::new(0.5, 0.5, 0.0, 1.0),
    Vec4::new(0.5, -0.5, 0.0, 1.0),
    Vec4::new(-0.5, -0.5, 0.0, 1.0),
];
const UNIT_SQUARE_INDICES: [u16; 6] = [0, 2, 1, 2, 0, 3];

pub struct RenderPipelineRecord {
    pub render_pipeline: RenderPipeline,
    pub pipeline_layout: PipelineLayout,
    pub shader_module: ShaderModule,
    pub shader_path: String,
    pub format: TextureFormat,
}

pub struct GeoUniformMatrix {
    matrix: Mat4,
    buffer: Buffer,
}

#[derive(Copy, Clone, Pod, Zeroable, ByteEq, ByteHash)]
#[repr(C)]
struct InstanceData {
    transform: Mat4,
    color: Vec4,
}

pub struct InstanceBufferManager {
    data: Vec<InstanceData>,
    pub buffer: Buffer,
}

impl InstanceBufferManager {
    fn new(device: Arc<Mutex<Device>>) -> Self {
        let device = device.lock().unwrap();
        let init_buffer_data = &[0u8; size_of::<InstanceData>() * 8];
        InstanceBufferManager {
            data: vec![],
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("instance buffer"),
                contents: bytemuck::cast_slice(init_buffer_data),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }),
        }
    }

    fn add_instance(&mut self, queue: Arc<Mutex<Queue>>, transform: Mat4, color: Vec4) {
        let queue = queue.lock().unwrap();
        let new_data = InstanceData { transform, color };
        queue.write_buffer(
            &self.buffer,
            (self.data.len() * size_of::<InstanceData>()) as u64,
            bytemuck::cast_slice(&[new_data]),
        );
        // todo realloc this buffer when full
        self.data.push(new_data);
    }
}

// various things needed to render geometry.
pub struct GeoInstances {
    pub render_pipeline_record: RenderPipelineRecord,
    pub bind_group: BindGroup,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub view_matrix_uniform: GeoUniformMatrix,
    pub instance_buffer_manager: InstanceBufferManager,
}

impl GeoInstances {
    pub fn add_new(&mut self, queue: Arc<Mutex<Queue>>, transform: Mat4, color: Vec4) {
        self.instance_buffer_manager
            .add_instance(queue, transform, color);
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
        let aspect = width as f32 / height as f32;
        let view_matrix = Mat4::orthographic_lh(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
        for ig in self.instance_groups.iter_mut() {
            ig.view_matrix_uniform.matrix = view_matrix;
            queue.write_buffer(
                &ig.view_matrix_uniform.buffer,
                0,
                bytemuck::cast_slice(&[view_matrix]),
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
                            buffers: &[
                                // todo future work, this is duplicated from init below; macro?
                                VertexBufferLayout {
                                    array_stride: size_of::<Vec4>() as BufferAddress,
                                    step_mode: VertexStepMode::Vertex,
                                    attributes: &[VertexAttribute {
                                        format: VertexFormat::Float32x4,
                                        offset: 0,
                                        shader_location: 0,
                                    }],
                                },
                                VertexBufferLayout {
                                    array_stride: size_of::<InstanceData>() as BufferAddress,
                                    step_mode: VertexStepMode::Instance,
                                    attributes: &[
                                        // mat4x4 transform
                                        wgpu::VertexAttribute {
                                            offset: 0,
                                            shader_location: 5,
                                            format: VertexFormat::Float32x4,
                                        },
                                        wgpu::VertexAttribute {
                                            offset: size_of::<[f32; 4]>() as BufferAddress,
                                            shader_location: 6,
                                            format: VertexFormat::Float32x4,
                                        },
                                        wgpu::VertexAttribute {
                                            offset: size_of::<[f32; 8]>() as BufferAddress,
                                            shader_location: 7,
                                            format: VertexFormat::Float32x4,
                                        },
                                        wgpu::VertexAttribute {
                                            offset: size_of::<[f32; 12]>() as BufferAddress,
                                            shader_location: 8,
                                            format: VertexFormat::Float32x4,
                                        },
                                        // vec4 color
                                        wgpu::VertexAttribute {
                                            offset: size_of::<[f32; 16]>() as BufferAddress,
                                            shader_location: 9,
                                            format: VertexFormat::Float32x4,
                                        },
                                    ],
                                },
                            ],
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
        format: TextureFormat,
        width: u32,
        height: u32,
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

        // view matrix initialization
        let aspect = width as f32 / height as f32;
        let view_matrix = Mat4::orthographic_lh(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
        let view_matrix_uniform = GeoUniformMatrix {
            matrix: view_matrix,
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("unit square view_matrix"),
                contents: bytemuck::cast_slice(&[view_matrix]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }),
        };

        // bind group layout and bind group creation
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: BufferSize::new((size_of::<Mat4>()) as u64),
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: view_matrix_uniform.buffer.as_entire_binding(),
            }],
            label: None,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // pipeline itself, with necessary components kept for recreation later.
        let render_pipeline_record = RenderPipelineRecord {
            render_pipeline: device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("unit square pipeline"),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader_module,
                    entry_point: "vs_main",
                    buffers: &[
                        VertexBufferLayout {
                            array_stride: size_of::<Vec4>() as BufferAddress,
                            step_mode: VertexStepMode::Vertex,
                            attributes: &[VertexAttribute {
                                format: VertexFormat::Float32x4,
                                offset: 0,
                                shader_location: 0,
                            }],
                        },
                        VertexBufferLayout {
                            array_stride: size_of::<InstanceData>() as BufferAddress,
                            step_mode: VertexStepMode::Instance,
                            attributes: &[
                                // todo replace with macro (and above in instance buffer)
                                // mat4x4 transform
                                VertexAttribute {
                                    offset: 0,
                                    shader_location: 5,
                                    format: VertexFormat::Float32x4,
                                },
                                VertexAttribute {
                                    offset: size_of::<[f32; 4]>() as BufferAddress,
                                    shader_location: 6,
                                    format: VertexFormat::Float32x4,
                                },
                                VertexAttribute {
                                    offset: size_of::<[f32; 8]>() as BufferAddress,
                                    shader_location: 7,
                                    format: VertexFormat::Float32x4,
                                },
                                VertexAttribute {
                                    offset: size_of::<[f32; 12]>() as BufferAddress,
                                    shader_location: 8,
                                    format: VertexFormat::Float32x4,
                                },
                                // vec4 color
                                VertexAttribute {
                                    offset: size_of::<[f32; 16]>() as BufferAddress,
                                    shader_location: 9,
                                    format: VertexFormat::Float32x4,
                                },
                            ],
                        },
                    ],
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
            view_matrix_uniform,
            instance_buffer_manager: InstanceBufferManager::new(self.device.clone()),
        });
    }
}
