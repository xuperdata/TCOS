fn main() {
    protoc_rust_grpc::Codegen::new()
        .out_dir("src/protos")
        .input("src/protos/chainedbft.proto")
        .input("src/protos/xendorser.proto")
        .input("src/protos/xchain.proto")
        .includes(&["src/protos"])
        .rust_protobuf(true)
        .run()
        .expect("protoc-rust-grpc");
}
