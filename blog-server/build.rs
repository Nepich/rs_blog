use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=proto/blog.proto");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("post_descriptor.bin"))
        .compile_protos(&["proto/blog.proto"], &["proto/"])
        .unwrap();
}
