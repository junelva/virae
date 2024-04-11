use std::borrow::Cow;
use std::mem::size_of;
use std::sync::{Arc, Mutex};

use glam::{Mat4, Vec4};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, Buffer, BufferUsages, Device, FragmentState,
    MultisampleState, PrimitiveState, RenderPipeline, RenderPipelineDescriptor,
    ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexState,
};

const UNIT_SQUARE_VERTICES: [Vec4; 4] = [
    Vec4::new(-0.5, 0.5, 0.0, 1.0),
    Vec4::new(0.5, 0.5, 0.0, 1.0),
    Vec4::new(0.5, -0.5, 0.0, 1.0),
    Vec4::new(-0.5, -0.5, 0.0, 1.0),
];
const UNIT_SQUARE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

// various things needed to render geometry.
pub struct GeoInstances {
    pub render_pipeline: RenderPipeline,
    pub bind_group: BindGroup,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub transforms: Vec<Mat4>,
}

pub struct GeoManager {
    pub device_arc: Arc<Mutex<Device>>,
    pub format: TextureFormat,
    pub instance_groups: Vec<GeoInstances>,
}

impl GeoManager {
    pub fn new(device_arc: Arc<Mutex<Device>>, format: TextureFormat) -> Self {
        Self {
            device_arc,
            format,
            instance_groups: vec![],
        }
    }

    pub fn new_unit_square(&mut self, device: Arc<Mutex<wgpu::Device>>, format: TextureFormat) {
        let device = device.lock().unwrap();

        // shader
        let shader_module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("test shader"),
            source: ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
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
        let vertex_buffer_layouts = [wgpu::VertexBufferLayout {
            array_stride: size_of::<Vec4>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            }],
        }];

        // bind groups and pipeline setup
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[],
            label: None,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // pipeline itself
        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("unit square pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &vertex_buffer_layouts,
            },
            fragment: Some(FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[Some(format.into())],
            }),
            primitive: PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview: None,
        });

        self.instance_groups.push(GeoInstances {
            render_pipeline,
            bind_group,
            vertex_buffer,
            index_buffer,
            transforms: vec![Mat4::IDENTITY],
        });
    }
}
