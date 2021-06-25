use getset::Getters;
use gfx_hal::{
    device::Device,
    pso::{self, Specialization},
    Backend,
};
use shaderc::ShaderKind;
use std::ptr;

const ENTRY_NAME: &str = "main";

/// A managed abstraction on top of Backend::ShaderModule
#[derive(Getters)]
pub struct GShaderModule<'a, B: Backend> {
    device: &'a B::Device,

    #[getset(get = "pub")]
    shader: B::ShaderModule,

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

impl<'a, B: Backend> GShaderModule<'a, B> {
    pub fn new(device: &'a B::Device, code: &str, kind: ShaderKind) -> GShaderModule<'a, B> {
        let spirv = &compile_shader(code, kind);
        let shader = unsafe { device.create_shader_module(&spirv) }.unwrap();
        GShaderModule {
            device,
            kind,
            shader,
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

impl<'a, B: Backend> Drop for GShaderModule<'a, B> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_shader_module(ptr::read(&self.shader));
        }
    }
}
