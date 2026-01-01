// Include the generated code
// burn-import generates a module structure based on the ONNX graph
pub mod model {
    #![allow(clippy::pub_underscore_fields, clippy::used_underscore_binding)]
    include!(concat!(env!("OUT_DIR"), "/model/bg_small_en_v1_5.rs"));
}

// Re-export the main model struct if needed, or use it directly
// The generated struct name usually matches the ONNX graph name or defaults to "Model"
// We'll inspect the generated code or update this wrapper after the first build attempt.
// For now, exposing the module is enough to let us see what's inside.
