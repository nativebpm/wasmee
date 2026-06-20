fn main() {
    let mut config = prost_build::Config::new();
    config.type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]");
    config.type_attribute(".", "#[serde(default)]");

    // field attributes for standard bytes fields
    config.field_attribute("wasmee.OplogEntry.request_payload", "#[serde(with = \"crate::serde_bytes_base64\")]");
    config.field_attribute("wasmee.OplogEntry.response_payload", "#[serde(with = \"crate::serde_bytes_base64\")]");
    config.field_attribute("wasmee.ExecuteRequest.base_snapshot", "#[serde(with = \"crate::serde_bytes_base64\")]");
    config.field_attribute("wasmee.ExecuteRequest.exchange_buffer", "#[serde(with = \"crate::serde_bytes_base64\")]");
    config.field_attribute("wasmee.ExecuteRequest.wasm_bytes", "#[serde(with = \"crate::serde_bytes_base64\")]");
    config.field_attribute("wasmee.CheckpointData.memory", "#[serde(with = \"crate::serde_bytes_base64\")]");
    config.field_attribute("wasmee.ExecuteResponse.response_bytes", "#[serde(with = \"crate::serde_bytes_base64\")]");

    // field attributes for maps of bytes
    config.field_attribute("wasmee.ExecuteRequest.memory_deltas", "#[serde(with = \"crate::serde_map_bytes_base64\")]");
    config.field_attribute("wasmee.ExecuteResponse.final_deltas", "#[serde(with = \"crate::serde_map_bytes_base64\")]");

    config.compile_protos(&["wasmee.proto"], &["."]).unwrap();
}
