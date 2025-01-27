use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Tell cargo to rerun this build script if the proto files change
    println!("cargo:rerun-if-changed=proto/");

    // Create proto directory if it doesn't exist
    let proto_dir = PathBuf::from("proto");
    if !proto_dir.exists() {
        fs::create_dir(&proto_dir).expect("Failed to create proto directory");
    }

    // Find all proto files
    let proto_files = find_proto_files(&proto_dir);

    // Configure protobuf compilation
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");

    // Compile proto files
    config
        .compile_protos(&proto_files, &[proto_dir])
        .expect("Failed to compile proto files");

    // Generate registry file
    let registry_content = generate_registry_file(&proto_files);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    fs::write(out_dir.join("registry.rs"), registry_content)
        .expect("Failed to write registry file");
}

fn find_proto_files(dir: &PathBuf) -> Vec<PathBuf> {
    let mut proto_files = Vec::new();
    if dir.is_dir() {
        for entry in fs::read_dir(dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "proto") {
                proto_files.push(path);
            }
        }
    }
    proto_files
}

fn generate_registry_file(_proto_files: &[PathBuf]) -> String {
    let mut content = String::new();
    content.push_str("use crate::message_registry::MessageRegistry;\n");
    content.push_str("use crate::{Vector3, ImuMessage};\n\n");
    content.push_str("pub fn register_messages(registry: &mut MessageRegistry) {\n");
    content.push_str("    registry.register::<Vector3>(\"zspy.Vector3\");\n");
    content.push_str("    registry.register::<ImuMessage>(\"zspy.ImuMessage\");\n");
    content.push_str("}\n");
    content
}
