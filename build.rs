use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=SKIP_MODEL_BUILD");

    // Check if we should skip model building (for testing or CI)
    if env::var("SKIP_MODEL_BUILD").is_ok() {
        create_stub_model();
        return;
    }

    // Only build the model if burn-import is available
    #[cfg(feature = "model-build")]
    {
        use burn_import::onnx::ModelGen;

        // Model configuration
        let model_dir = Path::new("src/uar/runtime/matching/models");
        let model_filename = "bg-small-en-v1.5.onnx";
        let model_path = model_dir.join(model_filename);
        let url =
            "https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/onnx/model_quantized.onnx";

        // Download model if it doesn't exist
        if !model_path.exists() {
            fs::create_dir_all(model_dir).expect("Failed to create model directory");

            // Use a simple command to download to avoid ureq version complexity in build script for now
            // since we know curl is available on the system (mac)
            let status = std::process::Command::new("curl")
                .arg("-L")
                .arg("-o")
                .arg(&model_path)
                .arg(url)
                .status()
                .expect("Failed to execute curl");

            assert!(status.success(), "Failed to download model");
        }

        // Download tokenizer files
        let tokenizer_files = vec![
            "tokenizer.json",
            "tokenizer_config.json",
            "special_tokens_map.json",
        ];
        for file in tokenizer_files {
            let file_path = model_dir.join(file);
            let file_url =
                format!("https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main/{file}");

            if !file_path.exists() {
                let status = std::process::Command::new("curl")
                    .arg("-L")
                    .arg("-o")
                    .arg(&file_path)
                    .arg(&file_url)
                    .status()
                    .expect("Failed to execute curl");

                if !status.success() {
                    // Non-fatal for config files.
                }
            }
            println!("cargo:rerun-if-changed={}", file_path.display());
        }

        println!("cargo:rerun-if-changed={}", model_path.display());

        // Generate Burn code from ONNX
        // The generated code will be placed in OUT_DIR/model.rs
        ModelGen::new()
            .input(model_path.to_str().expect("Valid path"))
            .out_dir(env::var("OUT_DIR").expect("OUT_DIR not set").as_str())
            .run_from_script();
    }

    #[cfg(not(feature = "model-build"))]
    {
        create_stub_model();
    }
}

fn create_stub_model() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let model_dir = Path::new(&out_dir).join("model");
    fs::create_dir_all(&model_dir).expect("Failed to create model directory");

    let stub_code = r"
use burn::module::Module;
use burn::tensor::backend::Backend;

#[derive(Module, Debug)]
pub struct Model<B: Backend> {
    _phantom: std::marker::PhantomData<B>,
}

impl<B: Backend> Model<B> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<B: Backend> Default for Model<B> {
    fn default() -> Self {
        Self::new()
    }
}
";

    let stub_path = model_dir.join("bg_small_en_v1_5.rs");
    fs::write(stub_path, stub_code).expect("Failed to write stub model");
}
