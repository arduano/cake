use std::{mem::size_of, num::NonZeroU32};

use bytemuck::{Pod, Zeroable};
use midi::data::IntVector4;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
struct RenderUniform {
    width: f32,
    height: f32,
    start: i32,
    end: i32,
}

impl RenderUniform {
    pub fn default() -> Self {
        RenderUniform {
            end: 0,
            start: 0,
            width: 0.0,
            height: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 2],
    tex_coord: [f32; 2],
    key: i32,
}

fn vertex(pos: [f32; 2], tc: [f32; 2], key: i32) -> Vertex {
    Vertex {
        pos: [pos[0], pos[1]],
        tex_coord: [tc[0], tc[1]],
        key,
    }
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let mut vertex_data = Vec::new();
    let mut index_data = Vec::new();

    for i in 0..256 {
        let x1 = i as f32 / 128.0;
        let x2 = (i + 1) as f32 / 128.0;
        vertex_data.append(&mut vec![
            vertex([x1, 0.0], [x1, x2], i),
            vertex([x2, 0.0], [x1, x2], i),
            vertex([x2, 1.0], [x1, x2], i),
            vertex([x1, 1.0], [x1, x2], i),
        ]);
        index_data.append(&mut vec![
            (i * 4 + 0) as u16,
            (i * 4 + 1) as u16,
            (i * 4 + 2) as u16,
            (i * 4 + 2) as u16,
            (i * 4 + 3) as u16,
            (i * 4 + 0) as u16,
        ]);
    }

    (vertex_data, index_data)
}

pub struct MidiRender {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl MidiRender {
    pub fn init(format: wgpu::TextureFormat, device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        use std::mem;

        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<Vertex>();
        let (vertex_data, index_data) = create_vertices();

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsage::INDEX,
        });

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(16),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let mut midi = midi::midifile::MIDIFile::new(
            "D:\\Midis\\Clubstep.mid",
            true,
            Some(&|read| {
                println!("{}", read);
            }),
        )
        .unwrap();

        let mut vec = midi.parse_all_tracks(16384).expect("MIDI parse failed");

        let size = 8192u32;
        let height = vec.len() as u32 / size + 1;

        let full_len = size * height;
        let offset = full_len - vec.len() as u32;

        vec.extend((0..offset).map(|_| IntVector4::default()));

        // Create the texture
        // let texels = create_texels(size as usize);
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Sint,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &bytemuck::cast_slice(&vec),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(size_of::<IntVector4>() as u32 * size),
                rows_per_image: None,
            },
            texture_extent,
        );

        // Create other resources
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let data_total = RenderUniform::default();
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            contents: bytemuck::cast_slice(&[data_total, data_total, data_total, data_total]),
        });

        // let mut f = File::open("./cake-cache.dat").expect("no file found");
        // let metadata = fs::metadata("./cake-cache.dat").expect("unable to read metadata");
        // let mut buffer = vec![0; metadata.len() as usize];
        // f.read(&mut buffer).expect("buffer overflow");

        let cake_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            usage: wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
            contents: bytemuck::cast_slice(&vec),
        });

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: cake_buf.as_entire_binding(),
                },
            ],
            label: None,
        });

        // Create the render pipeline
        let vs_module = device.create_shader_module(&wgpu::include_spirv!("data\\cake.vert.spv"));
        let fs_module = device.create_shader_module(&wgpu::include_spirv!("data\\cake.frag.spv"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_size as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 2 * 4,
                            shader_location: 1,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Uint32,
                            offset: 4 * 4,
                            shader_location: 2,
                        },
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrite::ALL,
                }],
            }),
        });

        // Done
        MidiRender {
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            bind_group,
            uniform_buf,
            pipeline,
        }
    }

    pub fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        size: &[f32; 2],
    ) {
        let mx_total = RenderUniform {
            end: 1505340,
            start: 0,
            width: size[0],
            height: size[1],
        };
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::cast_slice(&[mx_total]));

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0, // semi-transparent background
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}
