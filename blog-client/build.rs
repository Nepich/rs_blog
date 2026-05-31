fn main() {
    println!("cargo:rerun-if-changed=proto/blog.proto");
    tonic_prost_build::configure()
        .build_server(false)
        .compile_protos(&["proto/blog.proto"], &["proto"])
        .unwrap();
}
