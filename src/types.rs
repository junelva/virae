#![allow(dead_code)]

use glam::{Quat, UVec2};
use std::{
    mem::size_of,
    sync::{Arc, Mutex},
};

use bytemuck::{ByteEq, ByteHash, Pod, Zeroable};
use glam::{Mat4, Vec2, Vec3, Vec4};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    Sampler, Texture, TextureView,
};
use wgpu::{
    Buffer, BufferAddress, BufferUsages, Device, PipelineLayout, Queue, RenderPipeline,
    ShaderModule, TextureFormat, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub location: Vec3,
    pub tex_coords: Vec2,
}

pub const UNIT_SQUARE_VERTICES: [Vertex; 4] = [
    Vertex {
        location: Vec3::new(0.0, 0.0, 0.0),
        tex_coords: Vec2::new(0.0, 0.0),
    },
    Vertex {
        location: Vec3::new(1.0, 0.0, 0.0),
        tex_coords: Vec2::new(1.0, 0.0),
    },
    Vertex {
        location: Vec3::new(1.0, -1.0, 0.0),
        tex_coords: Vec2::new(1.0, 1.0),
    },
    Vertex {
        location: Vec3::new(0.0, -1.0, 0.0),
        tex_coords: Vec2::new(0.0, 1.0),
    },
];
pub const UNIT_SQUARE_INDICES: [u16; 6] = [0, 2, 1, 2, 0, 3];

pub const UNIT_SQUARE_BUFFER_LAYOUT: [VertexBufferLayout<'_>; 2] = [
    VertexBufferLayout {
        array_stride: (size_of::<Vec3>() + size_of::<Vec2>()) as BufferAddress,
        step_mode: VertexStepMode::Vertex,
        attributes: &[
            VertexAttribute {
                // vertex position
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            VertexAttribute {
                // vertex tex coord
                format: VertexFormat::Float32x2,
                offset: size_of::<Vec3>() as u64,
                shader_location: 1,
            },
        ],
    },
    VertexBufferLayout {
        array_stride: size_of::<InstanceData>() as BufferAddress,
        step_mode: VertexStepMode::Instance,
        attributes: &[
            // mat4x4 texture transform
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
            // mat4x4 transform
            VertexAttribute {
                offset: size_of::<[f32; 16]>() as BufferAddress,
                shader_location: 9,
                format: VertexFormat::Float32x4,
            },
            VertexAttribute {
                offset: size_of::<[f32; 20]>() as BufferAddress,
                shader_location: 10,
                format: VertexFormat::Float32x4,
            },
            VertexAttribute {
                offset: size_of::<[f32; 24]>() as BufferAddress,
                shader_location: 11,
                format: VertexFormat::Float32x4,
            },
            VertexAttribute {
                offset: size_of::<[f32; 28]>() as BufferAddress,
                shader_location: 12,
                format: VertexFormat::Float32x4,
            },
            // vec4 color
            VertexAttribute {
                offset: size_of::<[f32; 32]>() as BufferAddress,
                shader_location: 13,
                format: VertexFormat::Float32x4,
            },
        ],
    },
];

pub struct RenderPipelineRecord {
    pub render_pipeline: RenderPipeline,
    pub pipeline_layout: PipelineLayout,
    pub shader_module: ShaderModule,
    pub shader_path: String,
    pub format: TextureFormat,
}

pub struct GeoUniformVec2 {
    pub vec: Vec2,
    pub buffer: Buffer,
}

pub struct GeoUniformMatrix {
    pub matrix: Mat4,
    pub buffer: Buffer,
}

pub struct Instance {
    pub tex_transform: ComponentTransform,
    pub transform: ComponentTransform,
    pub color: Vec4,
}

#[derive(Copy, Clone, Pod, Zeroable, ByteEq, ByteHash)]
#[repr(C)]
pub struct InstanceData {
    pub tex_transform: Mat4,
    pub transform: Mat4,
    pub color: Vec4,
}

impl Default for InstanceData {
    fn default() -> Self {
        InstanceData {
            tex_transform: Mat4::IDENTITY,
            transform: Mat4::IDENTITY,
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

pub struct InstanceBufferManager {
    pub data: Vec<Instance>,
    pub buffer: Buffer,
}

impl InstanceBufferManager {
    pub fn new(max_instances: usize, device: Arc<Mutex<Device>>) -> Self {
        let device = device.lock().unwrap();
        let init_buffer_data = vec![InstanceData::default(); max_instances];
        InstanceBufferManager {
            data: vec![],
            buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("instance buffer"),
                contents: bytemuck::cast_slice(&init_buffer_data),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            }),
        }
    }

    pub fn add_instance(
        &mut self,
        queue: Arc<Mutex<Queue>>,
        _tex_transform: Option<ComponentTransform>,
        transform: ComponentTransform,
        color: Vec4,
    ) {
        let queue = queue.lock().unwrap();
        let new_data = InstanceData {
            tex_transform: Mat4::IDENTITY,
            transform: Mat4::from_scale_rotation_translation(
                transform.scale,
                transform.rotation,
                transform.location,
            ),
            color,
        };
        queue.write_buffer(
            &self.buffer,
            (self.data.len() * size_of::<InstanceData>()) as u64,
            bytemuck::cast_slice(&[new_data]),
        );
        self.data.push(Instance {
            tex_transform: ComponentTransform::default(),
            transform,
            color,
        });
    }

    pub fn recalc_screen_instances(&mut self, queue: Arc<Mutex<Queue>>, screen: Vec2) {
        let queue = queue.lock().unwrap();
        for (i, instance) in self.data.iter_mut().enumerate() {
            if instance.transform.pixel_rect.is_some() {
                let pr = instance
                    .transform
                    .pixel_rect
                    .expect("pixel rect unwrap error");
                instance.transform =
                    ComponentTransform::pixel_rect_to_screen_transform(PixelRect {
                        xy: pr.xy,
                        wh: pr.wh,
                        screen,
                    });
                let new_data = InstanceData {
                    tex_transform: Mat4::IDENTITY,
                    transform: Mat4::from_scale_rotation_translation(
                        instance.transform.scale,
                        instance.transform.rotation,
                        instance.transform.location,
                    ),
                    color: instance.color,
                };
                queue.write_buffer(
                    &self.buffer,
                    (i * size_of::<InstanceData>()) as BufferAddress,
                    bytemuck::cast_slice(&[new_data]),
                );
            }
        }
    }
}

pub struct TextureSheetClusterDefinition {
    pub label: String,
    pub size: UVec2,
    pub offset: UVec2,
    pub spacing: usize,
}

pub struct TextureSheetDefinition {
    pub path: String,
    pub clusters: Vec<TextureSheetClusterDefinition>,
}

impl TextureSheetDefinition {
    pub fn none() -> Self {
        Self {
            path: "".to_string(),
            clusters: vec![],
        }
    }
}

pub struct TextureSheet {
    pub sheet_info: TextureSheetDefinition,
    pub texture: Texture,
    pub sampler: Sampler,
    pub view: TextureView,
}

#[derive(Copy, Clone)]
pub struct PixelRect {
    pub xy: Vec2,
    pub wh: Vec2,
    pub screen: Vec2,
}

pub struct ComponentTransform {
    pub pixel_rect: Option<PixelRect>,
    pub location: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Default for ComponentTransform {
    fn default() -> Self {
        Self {
            pixel_rect: None,
            location: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}

impl ComponentTransform {
    pub fn pixel_rect_to_screen_transform(pixel_rect: PixelRect) -> ComponentTransform {
        // given window pixels x, y (top left) of w, h (width, height) produce a transform
        // that positions the UNIT_SQUARE geometry as desired in render space...

        let xy = pixel_rect.xy;
        let wh = pixel_rect.wh;
        let screen = pixel_rect.screen;

        let location = Vec3::new(
            (xy.x / screen.x) * 2.0 - 1.0,
            1.0 - (xy.y / screen.y) * 2.0,
            0.0,
        );
        let rotation = Quat::IDENTITY;
        let scale = Vec3::new((wh.x / screen.x) * 2.0, (wh.y / screen.y) * 2.0, 1.0);

        ComponentTransform {
            pixel_rect: Some(pixel_rect),
            location,
            rotation,
            scale,
        }
    }
}
