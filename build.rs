fn main() {
    prost_build::compile_protos(&["wasmee.proto"], &["."]).unwrap();
}
