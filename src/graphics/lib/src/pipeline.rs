use getset::Getters;
use gfx_hal::{
    device::Device,
    pso::{self, AttributeDesc, EntryPoint, InputAssemblerDesc, Specialization, VertexBufferDesc},
    Backend,
};
use shaderc::ShaderKind;
use std::{mem::ManuallyDrop, ptr};

pub struct PipelineBuilder<'a, B: Backend> {
    // Required
    buffers: &'a [VertexBufferDesc],
    attributes: &'a [AttributeDesc],
    vertex: EntryPoint<'a, B>,
    fragment: EntryPoint<'a, B>,

    // Optional
    tessellation: Option<(EntryPoint<'a, B>, EntryPoint<'a, B>)>,
    geometry: Option<EntryPoint<'a, B>>,
    input_assembler: InputAssemblerDesc,
}

pub struct GPipeline<'a, B: Backend> {
    device: &'a B::Device,
    pipeline: ManuallyDrop<<B as Backend>::GraphicsPipeline>,
}

impl<'a, B: Backend> PipelineBuilder<'a, B> {
    pub fn new(
        buffers: &'a [VertexBufferDesc],
        attributes: &'a [AttributeDesc],
        vertex: EntryPoint<'a, B>,
        fragment: EntryPoint<'a, B>,
    ) -> PipelineBuilder<'a, B> {
        let tessellation = None;
        let geometry = None;
        let input_assembler: InputAssemblerDesc =
            InputAssemblerDesc::new(pso::Primitive::TriangleList);

        PipelineBuilder {
            buffers,
            attributes,
            vertex,
            fragment,

            tessellation,
            geometry,
            input_assembler,
        }
    }

    // pub fn build(&self, device: &'a B::Device) -> GPipeline<B> {
    //     let mut pipeline_desc = pso::GraphicsPipelineDesc::new(
    //         pso::PrimitiveAssemblerDesc::Vertex {
    //             buffers: &self.buffers,
    //             attributes: &self.attributes,
    //             input_assembler: pso::InputAssemblerDesc {
    //                 primitive: pso::Primitive::TriangleList,
    //                 with_adjacency: false,
    //                 restart_index: None,
    //             },
    //             vertex: self.vertex,
    //             geometry: self.geometry,
    //             tessellation: self.tessellation,
    //         },
    //         pso::Rasterizer::FILL,
    //         Some(self.fragment),
    //         &*pipeline_layout,
    //         subpass,
    //     );

    //     pipeline_desc.blender.targets.push(pso::ColorBlendDesc {
    //         mask: pso::ColorMask::ALL,
    //         blend: Some(pso::BlendState::ALPHA),
    //     });

    //     let pipeline = unsafe { device.create_graphics_pipeline(&pipeline_desc, None) };

    //     GPipeline::<B>::new(device, pipeline.unwrap())
    // }
}

impl<'a, B: Backend> GPipeline<'a, B> {
    pub fn new(device: &'a B::Device, pipeline: B::GraphicsPipeline) -> GPipeline<'a, B> {
        return GPipeline {
            device,
            pipeline: ManuallyDrop::new(pipeline),
        };
    }
}

impl<'a, B: Backend> Drop for GPipeline<'a, B> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_graphics_pipeline(ManuallyDrop::into_inner(ptr::read(&self.pipeline)));
        }
    }
}
