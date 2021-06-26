use getset::Getters;
use gfx_hal::{
    device::Device,
    pass,
    pso::{self, AttributeDesc, EntryPoint, InputAssemblerDesc, Specialization, VertexBufferDesc},
    Backend,
};
use shaderc::ShaderKind;
use std::{borrow::Borrow, iter, mem::ManuallyDrop, ptr, sync::Arc};

macro_rules! return_option {
    ($a:expr) => {{
        match $a {
            Some(v) => return v,
            _ => panic!("Value already dropped!"),
        }
    }};
}

//
//  Pipeline
//

pub struct GPipelineBuilder<'a, B: Backend> {
    // Required
    buffers: &'a [VertexBufferDesc],
    attributes: &'a [AttributeDesc],
    vertex: EntryPoint<'a, B>,
    fragment: EntryPoint<'a, B>,
    layout: Arc<GPipelineLayout<B>>,
    subpass: pass::Subpass<'a, B>,

    // Optional
    tessellation: Option<(EntryPoint<'a, B>, EntryPoint<'a, B>)>,
    geometry: Option<EntryPoint<'a, B>>,
    input_assembler: InputAssemblerDesc,
}

impl<'a, B: Backend> GPipelineBuilder<'a, B> {
    pub fn new(
        buffers: &'a [VertexBufferDesc],
        attributes: &'a [AttributeDesc],
        vertex: EntryPoint<'a, B>,
        fragment: EntryPoint<'a, B>,
        layout: Arc<GPipelineLayout<B>>,
        subpass: pass::Subpass<'a, B>,
    ) -> GPipelineBuilder<'a, B> {
        let tessellation = None;
        let geometry = None;
        let input_assembler: InputAssemblerDesc =
            InputAssemblerDesc::new(pso::Primitive::TriangleList);

        GPipelineBuilder {
            buffers,
            attributes,
            vertex,
            fragment,
            layout,
            subpass,

            tessellation,
            geometry,
            input_assembler,
        }
    }

    pub fn build(self, device: Arc<B::Device>) -> GPipeline<B> {
        let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
            pso::PrimitiveAssemblerDesc::Vertex {
                buffers: &self.buffers,
                attributes: &self.attributes,
                input_assembler: self.input_assembler,
                vertex: self.vertex,
                geometry: self.geometry,
                tessellation: self.tessellation,
            },
            pso::Rasterizer::FILL,
            Some(self.fragment),
            self.layout.layout(),
            self.subpass,
        );

        pipeline_desc.blender.targets.push(pso::ColorBlendDesc {
            mask: pso::ColorMask::ALL,
            blend: Some(pso::BlendState::ALPHA),
        });

        let pipeline = unsafe { device.create_graphics_pipeline(&pipeline_desc, None) };

        GPipeline::<B>::new(device, pipeline.unwrap(), self.layout)
    }
}

pub struct GPipeline<B: Backend> {
    device: Arc<B::Device>,
    pipeline: Option<B::GraphicsPipeline>,
    layout: Arc<GPipelineLayout<B>>,
}

impl<B: Backend> GPipeline<B> {
    pub fn new(
        device: Arc<B::Device>,
        pipeline: B::GraphicsPipeline,
        layout: Arc<GPipelineLayout<B>>,
    ) -> GPipeline<B> {
        return GPipeline {
            device,
            pipeline: Some(pipeline),
            layout,
        };
    }

    pub fn pipeline(&self) -> &B::GraphicsPipeline {
        return_option!(&self.pipeline);
    }

    pub fn layout(&self) -> &Arc<GPipelineLayout<B>> {
        &self.layout
    }
}

impl<'a, B: Backend> Drop for GPipeline<B> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_graphics_pipeline(self.pipeline.take().unwrap());
        }
    }
}

//
//  Pipeline Layout
//

pub struct GPipelineLayout<B: Backend> {
    device: Arc<B::Device>,
    pipeline_layout: Option<B::PipelineLayout>,
    _set_layout: Arc<GDescriptorSetLayout<B>>,
}

impl<B: Backend> GPipelineLayout<B> {
    pub fn new(
        device: Arc<B::Device>,
        set_layout: Arc<GDescriptorSetLayout<B>>,
    ) -> GPipelineLayout<B> {
        let pipeline_layout = unsafe {
            device.create_pipeline_layout(iter::once(&*set_layout.layout()), iter::empty())
        }
        .expect("Can't create pipeline layout");

        return GPipelineLayout {
            device,
            pipeline_layout: Some(pipeline_layout),
            _set_layout: set_layout,
        };
    }

    pub fn layout(&self) -> &B::PipelineLayout {
        return_option!(&self.pipeline_layout);
    }
}

impl<'a, B: Backend> Drop for GPipelineLayout<B> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_pipeline_layout(self.pipeline_layout.take().unwrap());
        }
    }
}

//
//  Descriptor Set Layout
//

pub struct GDescriptorSetLayout<B: Backend> {
    device: Arc<B::Device>,
    layout: Option<B::DescriptorSetLayout>,
}

impl<B: Backend> GDescriptorSetLayout<B> {
    pub fn new<I: Iterator<Item = pso::DescriptorSetLayoutBinding>>(
        device: Arc<B::Device>,
        bindings: I,
    ) -> GDescriptorSetLayout<B> {
        let layout = unsafe { device.create_descriptor_set_layout(bindings, iter::empty()) }
            .expect("Can't create descriptor set layout");
        return GDescriptorSetLayout {
            device,
            layout: Some(layout),
        };
    }

    pub fn layout(&self) -> &B::DescriptorSetLayout {
        return_option!(&self.layout);
    }
}

impl<B: Backend> Drop for GDescriptorSetLayout<B> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_descriptor_set_layout(self.layout.take().unwrap());
        }
    }
}
