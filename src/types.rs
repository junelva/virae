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
            // mat4x4 texture transform
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
    pub transform: ComponentTransform,
    pub tex_transform: ComponentTransform,
    pub color: Vec4,
}

#[derive(Copy, Clone, Pod, Zeroable, ByteEq, ByteHash)]
#[repr(C)]
pub struct InstanceData {
    pub transform: Mat4,
    pub tex_transform: Mat4,
    pub color: Vec4,
}

impl Default for InstanceData {
    fn default() -> Self {
        InstanceData {
            transform: Mat4::IDENTITY,
            tex_transform: Mat4::IDENTITY,
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
        transform: ComponentTransform,
        tex_transform: ComponentTransform,
        color: Vec4,
    ) {
        let queue = queue.lock().unwrap();
        let new_data = InstanceData {
            transform: transform.to_mat4(),
            tex_transform: tex_transform.to_mat4(),
            color,
        };
        queue.write_buffer(
            &self.buffer,
            (self.data.len() * size_of::<InstanceData>()) as u64,
            bytemuck::cast_slice(&[new_data]),
        );
        self.data.push(Instance {
            transform,
            tex_transform,
            color,
        });
    }

    pub fn recalc_screen_instances(&mut self, queue: Arc<Mutex<Queue>>, screen: UVec2) {
        let queue = queue.lock().unwrap();
        for (i, instance) in self.data.iter_mut().enumerate() {
            if instance.transform.pixel_rect.is_some() {
                let pr = instance
                    .transform
                    .pixel_rect
                    .expect("pixel rect unwrap error");
                instance.transform =
                    ComponentTransform::unit_square_transform_from_pixel_rect(PixelRect {
                        xy: pr.xy,
                        wh: pr.wh,
                        extent: screen,
                    });
                let new_data = InstanceData {
                    transform: instance.transform.to_mat4(),
                    tex_transform: instance.tex_transform.to_mat4(),
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
    pub offset: UVec2,
    pub cluster_size: UVec2,
    pub sub_size: UVec2,
    pub spacing: UVec2,
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
    pub dimensions: UVec2,
    pub texture: Texture,
    pub sampler: Sampler,
    pub view: TextureView,
}

impl TextureSheet {
    pub fn cluster_sub_transform(
        &self,
        cluster_index: usize,
        sub_index: usize,
    ) -> ComponentTransform {
        let c /*cluster*/ = &self.sheet_info.clusters[cluster_index];
        let rc /*row count*/ = {
            let mut rc = 0;
            for _ in (0..c.cluster_size.x).step_by((c.sub_size.x + c.spacing.x) as usize) {
                rc += 1;
            }
            rc
        };

        let row_index = sub_index as u32 / rc;
        let col_index = sub_index as u32 % rc;

        let x_offset = c.offset.x + col_index * (c.sub_size.x + c.spacing.x);
        let y_offset = c.offset.y + row_index * (c.sub_size.y + c.spacing.y);

        ComponentTransform::tex_transform_from_pixel_rect(PixelRect {
            xy: UVec2::new(x_offset, y_offset),
            wh: c.sub_size,
            extent: self.dimensions,
        })
    }
}

#[derive(Copy, Clone)]
pub struct PixelRect {
    pub xy: UVec2,
    pub wh: UVec2,
    pub extent: UVec2,
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
    pub fn to_mat4(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.location)
    }

    pub fn tex_transform_from_pixel_rect(pixel_rect: PixelRect) -> ComponentTransform {
        let xy = Vec2::new(pixel_rect.xy.x as f32, pixel_rect.xy.y as f32);
        let wh = Vec2::new(pixel_rect.wh.x as f32, pixel_rect.wh.y as f32);
        let extent = Vec2::new(pixel_rect.extent.x as f32, pixel_rect.extent.y as f32);

        let location = Vec3::new(xy.x / extent.x, xy.y / extent.y, 0.0);
        let rotation = Quat::IDENTITY;
        let scale = Vec3::new(wh.x / extent.x, wh.y / extent.y, 1.0);

        ComponentTransform {
            pixel_rect: Some(pixel_rect),
            location,
            rotation,
            scale,
        }
    }

    pub fn unit_square_transform_from_pixel_rect(pixel_rect: PixelRect) -> ComponentTransform {
        // given window pixels x, y (top left) of w, h (width, height) produce a transform
        // that positions the UNIT_SQUARE geometry as desired in render space...

        let xy = Vec2::new(pixel_rect.xy.x as f32, pixel_rect.xy.y as f32);
        let wh = Vec2::new(pixel_rect.wh.x as f32, pixel_rect.wh.y as f32);
        let extent = Vec2::new(pixel_rect.extent.x as f32, pixel_rect.extent.y as f32);

        let location = Vec3::new(
            (xy.x / extent.x) * 2.0 - 1.0,
            1.0 - (xy.y / extent.y) * 2.0,
            0.0,
        );
        let rotation = Quat::IDENTITY;
        let scale = Vec3::new((wh.x / extent.x) * 2.0, (wh.y / extent.y) * 2.0, 1.0);

        ComponentTransform {
            pixel_rect: Some(pixel_rect),
            location,
            rotation,
            scale,
        }
    }
}
