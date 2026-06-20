use serde::{Deserializer, Serializer};
use std::collections::HashMap;

pub fn serialize<S>(map: &HashMap<i32, Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use serde::ser::SerializeMap;
    use base64::prelude::*;
    let mut map_ser = serializer.serialize_map(Some(map.len()))?;
    for (k, v) in map {
        let s = BASE64_STANDARD.encode(v);
        map_ser.serialize_entry(k, &s)?;
    }
    map_ser.end()
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<i32, Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error, MapAccess, Visitor};
    use base64::prelude::*;

    struct MapVisitor;

    impl<'de> Visitor<'de> for MapVisitor {
        type Value = HashMap<i32, Vec<u8>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map of integer keys to base64 strings or sequences")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = HashMap::new();
            while let Some(key_str) = access.next_key::<String>()? {
                let key = key_str.parse::<i32>().map_err(|e| M::Error::custom(format!("invalid map key: {}", e)))?;
                let value_val: serde_json::Value = access.next_value()?;
                let bytes = match value_val {
                    serde_json::Value::String(s) => {
                        let trimmed = s.trim();
                        if trimmed.is_empty() {
                            Vec::new()
                        } else {
                            BASE64_STANDARD.decode(trimmed).map_err(|e| M::Error::custom(format!("failed to decode base64: {}", e)))?
                        }
                    }
                    serde_json::Value::Array(arr) => {
                        let mut b = Vec::with_capacity(arr.len());
                        for v in arr {
                            if let Some(n) = v.as_u64() {
                                b.push(n as u8);
                            } else {
                                return Err(M::Error::custom("expected integer array for bytes"));
                            }
                        }
                        b
                    }
                    serde_json::Value::Null => Vec::new(),
                    _ => return Err(M::Error::custom("expected string or array for bytes")),
                };
                map.insert(key, bytes);
            }
            Ok(map)
        }
    }

    deserializer.deserialize_map(MapVisitor)
}
