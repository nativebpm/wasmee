use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DmnTable {
    pub id: String,
    #[serde(rename = "hitPolicy")]
    pub hit_policy: String,
    pub inputs: Vec<DmnInput>,
    pub outputs: Vec<DmnOutput>,
    pub rules: Vec<DmnRule>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DmnInput {
    pub expression: String,
    #[serde(rename = "type")]
    pub input_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DmnOutput {
    pub name: String,
    #[serde(rename = "type")]
    pub output_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DmnRule {
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
}

/// Evaluates the DmnTable decision table using the variables map.
pub fn evaluate(
    table: &DmnTable,
    variables: &HashMap<String, serde_json::Value>,
) -> Result<Option<HashMap<String, serde_json::Value>>, String> {
    let mut matched_rules = Vec::new();

    for rule in &table.rules {
        if rule.inputs.len() != table.inputs.len() {
            return Err(format!(
                "rule has mismatched inputs count: expected {}, got {}",
                table.inputs.len(),
                rule.inputs.len()
            ));
        }

        let mut is_match = true;
        for (i, entry_text) in rule.inputs.iter().enumerate() {
            let input_var_name = table.inputs[i].expression.trim();
            let val = variables
                .get(input_var_name)
                .cloned()
                .unwrap_or(serde_json::Value::Null);

            if !match_input_entry(&val, entry_text) {
                is_match = false;
                break;
            }
        }

        if is_match {
            matched_rules.push(rule);
        }
    }

    if matched_rules.is_empty() {
        return Ok(None);
    }

    // Apply Hit Policy
    let policy = table.hit_policy.to_uppercase();
    let policy = if policy.is_empty() { "UNIQUE" } else { &policy };

    match policy {
        "UNIQUE" => {
            if matched_rules.len() > 1 {
                return Err(format!(
                    "hit policy UNIQUE violated: multiple rules matched ({})",
                    matched_rules.len()
                ));
            }
            build_output_map(&table.outputs, &matched_rules[0].outputs).map(Some)
        }
        "FIRST" | "ANY" => {
            build_output_map(&table.outputs, &matched_rules[0].outputs).map(Some)
        }
        "COLLECT" => {
            let mut result = HashMap::new();
            for out in &table.outputs {
                result.insert(out.name.clone(), serde_json::Value::Array(Vec::new()));
            }
            for rule in matched_rules {
                let out_map = build_output_map(&table.outputs, &rule.outputs)?;
                for (k, v) in out_map {
                    if let Some(serde_json::Value::Array(arr)) = result.get_mut(&k) {
                        arr.push(v);
                    }
                }
            }
            Ok(Some(result))
        }
        _ => Err(format!("hit policy {} is not supported", policy)),
    }
}

fn build_output_map(
    outputs: &[DmnOutput],
    entries: &[String],
) -> Result<HashMap<String, serde_json::Value>, String> {
    if outputs.len() != entries.len() {
        return Err(format!(
            "outputs count mismatch: expected {}, got {}",
            outputs.len(),
            entries.len()
        ));
    }

    let mut result = HashMap::new();
    for (i, out) in outputs.iter().enumerate() {
        let val_str = entries[i].trim();
        let val_str = val_str.trim_matches(|c| c == '\'' || c == '"');

        if let Ok(b) = val_str.parse::<bool>() {
            result.insert(out.name.clone(), serde_json::Value::Bool(b));
        } else if let Ok(f) = val_str.parse::<f64>() {
            if let Some(num) = serde_json::Number::from_f64(f) {
                result.insert(out.name.clone(), serde_json::Value::Number(num));
            } else {
                result.insert(out.name.clone(), serde_json::Value::String(val_str.to_string()));
            }
        } else {
            result.insert(out.name.clone(), serde_json::Value::String(val_str.to_string()));
        }
    }

    Ok(result)
}

fn match_input_entry(val: &serde_json::Value, entry_text: &str) -> bool {
    let entry_text = entry_text.trim();
    if entry_text.is_empty() || entry_text == "-" {
        return true;
    }

    let str_val = match val {
        serde_json::Value::Null => "".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        _ => val.to_string(),
    };
    let str_val = str_val.trim();

    if entry_text.starts_with("<=") {
        let limit_str = entry_text["<=".len()..].trim();
        let v = str_val.parse::<f64>();
        let l = limit_str.parse::<f64>();
        return v.is_ok() && l.is_ok() && v.unwrap() <= l.unwrap();
    }
    if entry_text.starts_with(">=") {
        let limit_str = entry_text[">=.".len() - 1..].trim(); // 2 chars
        let v = str_val.parse::<f64>();
        let l = limit_str.parse::<f64>();
        return v.is_ok() && l.is_ok() && v.unwrap() >= l.unwrap();
    }
    if entry_text.starts_with('<') {
        let limit_str = entry_text[1..].trim();
        let v = str_val.parse::<f64>();
        let l = limit_str.parse::<f64>();
        return v.is_ok() && l.is_ok() && v.unwrap() < l.unwrap();
    }
    if entry_text.starts_with('>') {
        let limit_str = entry_text[1..].trim();
        let v = str_val.parse::<f64>();
        let l = limit_str.parse::<f64>();
        return v.is_ok() && l.is_ok() && v.unwrap() > l.unwrap();
    }
    if entry_text.starts_with("!=") {
        let limit_str = entry_text[2..].trim().trim_matches(|c| c == '\'' || c == '"');
        return str_val != limit_str;
    }
    if entry_text.starts_with("<>") {
        let limit_str = entry_text[2..].trim().trim_matches(|c| c == '\'' || c == '"');
        return str_val != limit_str;
    }

    let entry_text = entry_text.trim_matches(|c| c == '\'' || c == '"');
    str_val == entry_text
}
