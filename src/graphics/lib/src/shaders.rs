use getset::Getters;
use gfx_hal::{Backend, device::Device, pso::{self, EntryPoint, Specialization}};
use shaderc::ShaderKind;
use std::{cell::RefCell, mem::ManuallyDrop, ptr, sync::Arc};

use crate::device::GDevice;

const ENTRY_NAME: &str = "main";

/// A managed abstraction on top of Backend::ShaderModule
#[derive(Getters)]
pub struct GShaderModule<B: Backend> {
    device: Arc<B::Device>,

    #[getset(get = "pub")]
    shader: ManuallyDrop<B::ShaderModule>,

    #[getset(get = "pub")]
    kind: ShaderKind,
}

fn compile_shader(glsl: &str, shader_kind: ShaderKind) -> Vec<u32> {
    let mut compiler = shaderc::Compiler::new().unwrap();

    let compiled_shader = compiler
        .compile_into_spirv(glsl, shader_kind, "unnamed", "main", None)
        .expect("Failed to compile shader");

    compiled_shader.as_binary().to_vec()
}

impl<B: Backend> GShaderModule<B> {
    pub fn new(gdevice: &GDevice<B>, code: &str, kind: ShaderKind) -> GShaderModule<B> {
        let device = gdevice.logical.clone();

        let spirv = &compile_shader(code, kind);
        let shader = unsafe { device.create_shader_module(&spirv) }.unwrap();
        GShaderModule {
            device,
            kind,
            shader: ManuallyDrop::new(shader),
        }
    }

    pub fn entrypoint<'s>(&'s self) -> pso::EntryPoint<'s, B> {
        self.entrypoint_with(None, None)
    }

    pub fn entrypoint_with<'s>(
        &'s self,
        entry: Option<&'s str>,
        specialization: Option<Specialization<'s>>,
    ) -> pso::EntryPoint<'s, B> {
        pso::EntryPoint::<'s, B> {
            entry: entry.unwrap_or(ENTRY_NAME),
            module: &self.shader,
            specialization: specialization.unwrap_or(pso::Specialization::default()),
        }
    }
}

impl<B: Backend> Drop for GShaderModule<B> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(ManuallyDrop::into_inner(ptr::read(&self.shader)));
        }
    }
}

// #[derive(Getters)]
// pub struct GShaderEntrypoint<'s, B: Backend> {
//     #[getset(get = "pub")]
//     entrypoint: Option<pso::EntryPoint<'s, B>>,

//     #[getset(get = "pub")]
//     shader: Arc<GShaderModule<B>>,
// }

// impl<'s, B: Backend> GShaderEntrypoint<'s, B> {
//     // pub fn from(shader: Arc<GShaderModule<B>>) -> GShaderEntrypoint<'s, B> {
//     //     GShaderEntrypoint::<B>::from_with(shader, None, None)
//     // }

//     pub fn from_with(
//         shader: Arc<GShaderModule<B>>,
//         entry: Option<&'s str>,
//         specialization: Option<Specialization<'s>>,
//     ) -> GShaderEntrypoint<'s, B> {
//         let shader_copy = shader.clone();

//         let gentry = RefCell::new(GShaderEntrypoint {
//             entrypoint: None,
//             shader: shader_copy,
//         });

//         let entrypoint = GShaderEntrypoint::insert(&gentry.borrow(), entry, specialization);

//         gentry.borrow_mut().entrypoint = Some(entrypoint);

//         // let ret = RefCell::into_inner(gentry);
        
//         // ret
//     }

//     fn insert(
//         parent: &'s GShaderEntrypoint<'s, B>,
//         entry: Option<&'s str>,
//         specialization: Option<Specialization<'s>>,
//     ) -> EntryPoint<'s, B> {
//         let entrypoint = pso::EntryPoint::<'s, B> {
//             entry: entry.unwrap_or(ENTRY_NAME),
//             module: &parent.shader.shader(),
//             specialization: specialization.unwrap_or(pso::Specialization::default()),
//         };

//         entrypoint
//     }
// }
