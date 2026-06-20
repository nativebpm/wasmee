fn main() {
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.type_attribute(".", "#[serde(default)]");
    config.compile_protos(&["wasmee.proto"], &["."]).unwrap();
}
