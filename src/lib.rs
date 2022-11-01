use rand::prelude::*;
use std::{f64::consts::PI, mem};
use wgpu::{include_wgsl, util::DeviceExt};
use winit::{event::WindowEvent, window::Window};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    clear_color: wgpu::Color,
    framecount: u64,
    pub size: winit::dpi::PhysicalSize<u32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    offset: [f32; 2],
    color: [f32; 3],
}

const X_RES: i64 = 640;
const Y_RES: i64 = 480;

const OBS_DIST: i64 = 1000;
const EYE_SEP: i64 = 300;

const FAR: i64 = 250;
const CLOSE: i64 = 200;

const X_SIDE: f32 = 2.0 / (X_RES as f32);
const Y_SIDE: f32 = 2.0 / (Y_RES as f32);

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0, 0.0],
    },
    Vertex {
        position: [X_SIDE, 0.0, 0.0],
    },
    Vertex {
        position: [X_SIDE, Y_SIDE, 0.0],
    },
    Vertex {
        position: [0.0, Y_SIDE, 0.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x3];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![1 => Float32x2, 2 => Float32x3];

    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(&include_wgsl!("shader.wgsl"));

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        let instances = (0..Y_RES)
            .flat_map(|y| {
                (0..X_RES).map(move |x| {
                    let (x, y) = (x as f32, y as f32);
                    let position = [-1.0 + x * X_SIDE, -1.0 + y * Y_SIDE];

                    Instance {
                        offset: position,
                        color: [0.0; 3],
                    }
                })
            })
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let clear_color = wgpu::Color {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };

        Self {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            instances,
            instance_buffer,
            clear_color,
            size,
            framecount: 0,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved {
                device_id: _,
                position: _,
                modifiers: _,
            } => true,
            _ => false,
        }
    }

    pub fn update(&mut self) {
        self.framecount += 1;

        for y in 0..Y_RES {
            let mut look: [i64; X_RES as usize] = [0; X_RES as usize];

            for x in 0..X_RES {
                look[x as usize] = x;
            }

            for x in 0..X_RES {
                const DT: f64 = 500.0;

                let radius = 100.0;

                let ox = radius * (self.framecount as f64 / DT).sin();
                let oy = radius * (self.framecount as f64 / DT).cos();

                let xmid = X_RES / 2 + ox as i64;
                let ymid = Y_RES / 2 + oy as i64;

                let r = x.abs_diff(xmid) * x.abs_diff(xmid) + y.abs_diff(ymid) * y.abs_diff(ymid);

                let z: i64 = if r <= 10000 {
                    CLOSE
                } else {
                    FAR - ((FAR - CLOSE) as f64
                        * (1.0 - (y.abs_diff(Y_RES / 2) as f64 * X_SIDE as f64 * PI).cos()))
                        as i64
                };

                let sep = EYE_SEP * z / (z + OBS_DIST);
                let left = x as i64 - sep / 2;
                let right = left + sep;

                if left >= 0 && right < X_RES {
                    look[right as usize] = left;
                }
            }

            let mut rng = StdRng::seed_from_u64(y as u64);

            for x in 0..X_RES {
                let offset = (X_RES * y) as usize;

                if look[x as usize] == x {
                    let x = x as usize;
                    self.instances[offset + x as usize].color = [rng.gen_range(0.0..=1.0); 3]
                } else {
                    let x = x as usize;
                    let lx = look[x] as usize;
                    self.instances[offset + x].color = self.instances[offset + lx].color;
                }
            }
        }

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what [[location(0)]] in the fragment shader targets
                    wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: true,
                        },
                    },
                ],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
