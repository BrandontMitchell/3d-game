use crate::assets::{Assets, ModelRef};
use crate::camera::Camera;
use crate::model::*;
use crate::texture;
use crate::Game;
use cgmath::SquareMatrix;
use pixels::Pixels;
use std::collections::BTreeMap;
use wgpu::util::DeviceExt;

use winit::{dpi::PhysicalSize, window::Window};
pub(crate) struct Render {
    surface: wgpu::Surface,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    pub(crate) texture_layout: wgpu::BindGroupLayout,
    pub(crate) camera: Camera,
    uniforms: Uniforms,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    pub(crate) ambient: f32,
    light_ambient_buffer: wgpu::Buffer,
    lights: Vec<crate::lights::Light>,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    depth_texture: texture::Texture,
    instance_groups: InstanceGroups,
}

impl Render {
    pub(crate) async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: adapter.get_swap_chain_preferred_format(&surface),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler {
                            comparison: false,
                            filtering: true,
                        },
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let camera = Camera {
            eye: (0.0, 5.0, -10.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
            aspect: sc_desc.width as f32 / sc_desc.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 200.0,
        };

        let mut uniforms = Uniforms::new();
        uniforms.update_view_proj(&camera);

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });
        use crate::geom::*;
        let lights = vec![crate::lights::Light::point(
            Pos3::new(0.0, 10.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
        )];
        let light_uniform_size =
            (10 * std::mem::size_of::<crate::lights::Light>()) as wgpu::BufferAddress;
        let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Lights buffer"),
            size: light_uniform_size,
            usage: wgpu::BufferUsage::UNIFORM
                | wgpu::BufferUsage::COPY_SRC
                | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });
        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                10 * std::mem::size_of::<crate::lights::Light>()
                                    as wgpu::BufferAddress,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(
                                std::mem::size_of::<f32>() as wgpu::BufferAddress
                            ),
                        },
                        count: None,
                    },
                ],
                label: Some("light_bind_group_layout"),
            });

        let ambient = 1.0;

        let light_ambient_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("light_ambient"),
            contents: bytemuck::cast_slice(&[ambient]),
            usage: wgpu::BufferUsage::UNIFORM
                | wgpu::BufferUsage::COPY_SRC
                | wgpu::BufferUsage::COPY_DST,
        });

        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: light_ambient_buffer.as_entire_binding(),
                },
            ],
            label: Some("light_bind_group"),
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &sc_desc, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &uniform_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let vs_module = device.create_shader_module(&wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("shader.frag.spv"));

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format: sc_desc.format,
                    alpha_blend: wgpu::BlendState::REPLACE,
                    color_blend: wgpu::BlendState::REPLACE,
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                // Setting this to true requires Features::DEPTH_CLAMPING
                clamp_depth: false,
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        });

        Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            size,
            render_pipeline,
            camera,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
            ambient,
            light_ambient_buffer,
            lights,
            light_buffer,
            light_bind_group,
            texture_layout: texture_bind_group_layout,
            depth_texture,
            instance_groups: InstanceGroups::new(),
        }
    }

    pub(crate) fn set_ambient(&mut self, amb: f32) {
        self.ambient = amb;
        self.queue
            .write_buffer(&self.light_ambient_buffer, 0, bytemuck::cast_slice(&[amb]));
    }

    pub(crate) fn set_lights(&mut self, ls: Vec<crate::lights::Light>) {
        assert!(ls.len() < 10);
        self.lights = ls;
        self.queue
            .write_buffer(&self.light_buffer, 0, bytemuck::cast_slice(&self.lights));
    }

    pub(crate) fn update_buffers<R, G: Game<StaticData = R>>(
        &mut self,
        game: &mut G,
        _rules: &R,
        assets: &mut Assets,
        pixels: &mut (Pixels, PhysicalSize<u32>),
    ) -> bool {
        self.uniforms.update_view_proj(&self.camera);
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
        self.instance_groups.clear();
        let two_d = game.render(&mut self.instance_groups, pixels);
        self.instance_groups
            .update_buffers(&self.queue, &self.device, assets);

        two_d
    }

    pub(crate) fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.sc_desc.width = new_size.width;
        self.sc_desc.height = new_size.height;
        self.camera.aspect = self.sc_desc.width as f32 / self.sc_desc.height as f32;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.sc_desc, "depth_texture");
    }

    pub(crate) fn render<R, G: Game<StaticData = R>>(
        &mut self,
        game: &mut G,
        rules: &R,
        assets: &mut Assets,
        pixels: &mut (Pixels, PhysicalSize<u32>),
    ) -> Result<(), wgpu::SwapChainError> {
        let two_d = self.update_buffers(game, rules, assets, pixels);

        match two_d {
            false => {
                let frame = self.swap_chain.get_current_frame()?.output;

                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &frame.view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.1,
                                    g: 0.2,
                                    b: 0.3,
                                    a: 1.0,
                                }),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: Some(
                            wgpu::RenderPassDepthStencilAttachmentDescriptor {
                                attachment: &self.depth_texture.view,
                                depth_ops: Some(wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(1.0),
                                    store: true,
                                }),
                                stencil_ops: None,
                            },
                        ),
                    });

                    render_pass.set_pipeline(&self.render_pipeline);
                    for (mr, (irs, buf, _cap)) in self.instance_groups.groups.iter() {
                        render_pass.set_vertex_buffer(1, buf.as_ref().unwrap().slice(..));
                        render_pass.draw_model_instanced(
                            assets.get_model(*mr).unwrap(),
                            0..irs.len() as u32,
                            &self.uniform_bind_group,
                            &self.light_bind_group,
                        );
                    }
                }

                self.queue.submit(std::iter::once(encoder.finish()));
            }
            true => {}
        }
        Ok(())
    }
}

pub struct InstanceGroups {
    groups: BTreeMap<ModelRef, (Vec<InstanceRaw>, Option<wgpu::Buffer>, usize)>,
}
impl InstanceGroups {
    fn new() -> Self {
        Self {
            groups: BTreeMap::new(),
        }
    }
    fn clear(&mut self) {
        for (_mr, (irs, _buf, _cap)) in self.groups.iter_mut() {
            irs.clear();
        }
    }
    fn update_buffers(&mut self, queue: &wgpu::Queue, device: &wgpu::Device, assets: &Assets) {
        for (mr, (irs, buf, cap)) in self.groups.iter_mut() {
            if buf.is_none() || *cap < irs.len() {
                buf.replace(
                    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some(assets.path_for_model_ref(*mr).to_str().unwrap()),
                        usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
                        contents: bytemuck::cast_slice(irs),
                    }),
                );
                *cap = irs.len();
            } else {
                queue.write_buffer(buf.as_ref().unwrap(), 0, bytemuck::cast_slice(irs));
            }
        }
    }
    pub fn render(&mut self, mr: ModelRef, ir: InstanceRaw) {
        self.render_batch(mr, std::iter::once(ir));
    }
    pub fn render_batch(&mut self, mr: ModelRef, ir: impl IntoIterator<Item = InstanceRaw>) {
        let ref mut groups = self.groups;
        groups
            .entry(mr)
            .or_insert((vec![], None, 0))
            .0
            .extend(ir.into_iter())
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    #[allow(dead_code)]
    pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We don't have to do this in code though.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float4,
                },
            ],
        }
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    view_position: [f32; 4],
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

impl Uniforms {
    fn new() -> Self {
        Self {
            view_position: [0.0; 4],
            view: cgmath::Matrix4::identity().into(),
            proj: cgmath::Matrix4::identity().into(),
        }
    }

    fn update_view_proj(&mut self, camera: &Camera) {
        self.view_position = camera.eye.to_homogeneous().into();
        let (view, proj) = camera.build_view_projection_matrix();
        self.view = view.into();
        self.proj = (OPENGL_TO_WGPU_MATRIX * proj).into();
    }
}

//Types
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: u16,
    pub h: u16,
}

impl Rect {
    pub fn translate(&mut self, x: i32, y: i32) {
        self.x += x;
        self.y += y;
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Vec2i(pub i32, pub i32);

#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub struct Rgba(pub u8, pub u8, pub u8, pub u8);
