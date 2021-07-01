use shaderc::{self, ShaderKind};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Tell the build script to only run again if we change our source shaders
    println!("cargo:rerun-if-changed=src/data");

    // Create destination path if necessary
    std::fs::create_dir_all("src/data")?;

    for entry in std::fs::read_dir("src/data")? {
        let entry = entry?;

        if entry.file_type()?.is_file() {
            let in_path = entry.path();

            // Support only vertex and fragment shaders currently
            let shader_type =
                in_path
                    .extension()
                    .and_then(|ext| match ext.to_string_lossy().as_ref() {
                        "vert" => Some(ShaderKind::Vertex),
                        "frag" => Some(ShaderKind::Fragment),
                        _ => None,
                    });

            if let Some(shader_type) = shader_type {
                let source = std::fs::read_to_string(&in_path)?;

                let mut compiler = shaderc::Compiler::new().unwrap();

                let compiled_shader = compiler
                    .compile_into_spirv(&source, shader_type, "unnamed", "main", None)
                    .expect("Failed to compile shader");

                let compiled_bytes = compiled_shader.as_binary_u8();

                let out_path = format!(
                    "src/data/{}.spv",
                    in_path.file_name().unwrap().to_string_lossy()
                );

                std::fs::write(&out_path, &compiled_bytes)?;
            }
        }
    }

    Ok(())
}
