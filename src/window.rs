use wgpu::{
    CommandEncoderDescriptor, CompositeAlphaMode, Device, DeviceDescriptor, Features, IndexFormat,
    Instance, InstanceDescriptor, Limits, LoadOp, Operations, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor,
};
use winit::{
    dpi::{LogicalSize, PhysicalSize},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use std::{
    fs::metadata,
    sync::{Arc, Mutex},
    time::SystemTime,
};

use crate::geo::GeoManager;
use crate::text::TextCollection;

enum FileWatcherAction {
    ReloadShader,
}

struct FileWatcherEntry {
    path: String,
    last_modified: SystemTime,
    action: FileWatcherAction,
}

pub struct FileWatcher {
    entries: Vec<FileWatcherEntry>,
}

impl FileWatcher {
    fn new() -> Self {
        FileWatcher { entries: vec![] }
    }

    pub fn add_path(&mut self, path: &str) {
        let metadata = metadata(path).unwrap();
        self.entries.push(FileWatcherEntry {
            path: path.to_string(),
            last_modified: metadata.modified().unwrap(),
            action: FileWatcherAction::ReloadShader,
        })
    }
}

pub struct Context<'a> {
    pub device: Arc<Mutex<Device>>,
    pub queue: Arc<Mutex<Queue>>,
    pub surface: Arc<Mutex<Surface<'a>>>,
    pub config: Arc<Mutex<SurfaceConfiguration>>,
    pub swapchain_format: TextureFormat,
    pub scale_factor: f64,
    pub texts: TextCollection,
    pub geos: GeoManager,
    pub file_watcher: FileWatcher,
}

impl Context<'_> {
    pub async fn new(
        title: &str,
        width: u32,
        height: u32,
    ) -> (EventLoop<()>, Arc<winit::window::Window>, Self) {
        // event loop, window
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let window = Arc::new(
            WindowBuilder::new()
                .with_inner_size(LogicalSize::new(width as f64, height as f64))
                .with_decorations(true)
                .with_title(title)
                .build(&event_loop)
                .unwrap(),
        );
        let size = window.inner_size();
        let scale_factor = window.scale_factor();

        // instance, adapter, device, queue
        let instance = Instance::new(InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .unwrap();

        // surface, format, config
        let surface = instance
            .create_surface(window.clone())
            .expect("Create surface");
        let swapchain_format = TextureFormat::Bgra8UnormSrgb;
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::Fifo,
            alpha_mode: CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let device_arc = Arc::<Mutex<Device>>::new(Mutex::new(device));
        let queue_arc = Arc::<Mutex<Queue>>::new(Mutex::new(queue));
        let texts = TextCollection::new(device_arc.clone(), queue_arc.clone(), swapchain_format);

        (
            event_loop,
            window,
            Self {
                device: device_arc.clone(),
                queue: queue_arc,
                surface: Arc::<Mutex<Surface>>::new(Mutex::new(surface)),
                config: Arc::<Mutex<SurfaceConfiguration>>::new(Mutex::new(config)),
                swapchain_format,
                scale_factor,
                texts,
                geos: GeoManager::new(device_arc.clone(), swapchain_format),
                file_watcher: FileWatcher::new(),
            },
        )
    }

    pub fn check_watched_files(&mut self) {
        for fwe in self.file_watcher.entries.iter_mut() {
            let metadata = metadata(&*fwe.path).unwrap();
            if metadata.modified().unwrap() > fwe.last_modified {
                match fwe.action {
                    FileWatcherAction::ReloadShader => {
                        self.geos.reload_shader(self.device.clone(), &fwe.path)
                    }
                }
            }
        }
    }

    pub fn update(&mut self) {
        self.check_watched_files();
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let device = self.device.lock().unwrap();
        let mut config = self.config.lock().unwrap();
        config.width = size.width;
        config.height = size.height;
        let surface = self.surface.lock().unwrap();
        surface.configure(&device, &config);
        self.geos
            .update_view(self.queue.clone(), config.width, config.height);
    }

    pub fn render(&mut self) {
        let config = self.config.lock().unwrap();

        self.texts.prepare(
            self.device.clone(),
            self.queue.clone(),
            config.width,
            config.height,
        );

        let device = self.device.lock().unwrap();
        let queue = self.queue.lock().unwrap();
        let surface = self.surface.lock().unwrap();

        let frame = surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.1,
                            b: 0.15,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // include geos in pass
            pass.set_pipeline(
                &self.geos.instance_groups[0]
                    .render_pipeline_record
                    .render_pipeline,
            );
            pass.set_bind_group(0, &self.geos.instance_groups[0].bind_group, &[]);
            pass.set_index_buffer(
                self.geos.instance_groups[0].index_buffer.slice(..),
                IndexFormat::Uint16,
            );
            pass.set_vertex_buffer(0, self.geos.instance_groups[0].vertex_buffer.slice(..));
            pass.set_vertex_buffer(
                1,
                self.geos.instance_groups[0]
                    .instance_buffer_manager
                    .buffer
                    .slice(..),
            );
            pass.draw_indexed(0..6_u32, 0, 0..self.geos.num_instances(0));

            // include text labels in pass
            self.texts
                .text_renderer
                .render(&self.texts.atlas, &mut pass)
                .unwrap();
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
        self.texts.trim_atlas();
    }
}
