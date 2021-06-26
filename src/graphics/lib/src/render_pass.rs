use getset::Getters;
use gfx_hal::{
    device::Device,
    format::Format,
    image, pass,
    pso::{self, AttributeDesc, EntryPoint, InputAssemblerDesc, Specialization, VertexBufferDesc},
    Backend,
};
use shaderc::ShaderKind;
use std::{borrow::Borrow, iter, mem::ManuallyDrop, ptr, sync::Arc};

use crate::{device::GDevice, return_option};

pub struct GRenderPassBuilder<'a> {
    // Required
    format: Format,

    // Optional
    colors: &'a [(usize, image::Layout)],
    depth_stencil: Option<&'a (usize, image::Layout)>,
}

impl<'a> GRenderPassBuilder<'a> {
    pub fn new(format: Format) -> GRenderPassBuilder<'a> {
        let colors = &[(0, image::Layout::ColorAttachmentOptimal)];
        let depth_stencil = None;

        GRenderPassBuilder {
            format,
            colors,
            depth_stencil,
        }
    }

    pub fn build<B: Backend>(self, gdevice: &GDevice<B>) -> GRenderPass<B> {
        let device = gdevice.logical.clone();
        
        let render_pass = {
            let attachment = pass::Attachment {
                format: Some(self.format),
                samples: 1,
                ops: pass::AttachmentOps::new(
                    pass::AttachmentLoadOp::Clear,
                    pass::AttachmentStoreOp::Store,
                ),
                stencil_ops: pass::AttachmentOps::DONT_CARE,
                layouts: image::Layout::Undefined..image::Layout::Present,
            };

            let subpass = pass::SubpassDesc {
                colors: self.colors,
                depth_stencil: self.depth_stencil,
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };

            unsafe {
                device.create_render_pass(
                    iter::once(attachment),
                    iter::once(subpass),
                    iter::empty(),
                )
            }
            .expect("Can't create render pass")
        };

        GRenderPass::<B>::new(gdevice, render_pass)
    }
}

pub struct GRenderPass<B: Backend> {
    device: Arc<B::Device>,
    render_pass: Option<B::RenderPass>,
}

impl<B: Backend> GRenderPass<B> {
    pub fn new(gdevice: &GDevice<B>, render_pass: B::RenderPass) -> GRenderPass<B> {
        let device = gdevice.logical.clone();

        return GRenderPass {
            device,
            render_pass: Some(render_pass),
        };
    }

    pub fn render_pass(&self) -> &B::RenderPass {
        return_option!(&self.render_pass);
    }
}

impl<'a, B: Backend> Drop for GRenderPass<B> {
    fn drop(&mut self) {
        unsafe {
            self.device
                .destroy_render_pass(self.render_pass.take().unwrap());
        }
    }
}
