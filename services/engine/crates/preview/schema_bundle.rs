use serde_json::{Map, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

pub fn check_reference(
    root: &Path,
    source: &Path,
    reference: &str,
) -> Result<(PathBuf, String), String> {
    let (file, fragment) = reference.split_once('#').unwrap_or((reference, ""));
    if file.contains(':')
        || file.contains('\\')
        || Path::new(file).is_absolute()
        || (!fragment.is_empty() && !fragment.starts_with('/'))
    {
        return Err("only local JSON pointer references are supported".into());
    }
    let path = if file.is_empty() {
        source.to_path_buf()
    } else {
        source
            .parent()
            .ok_or("missing schema directory")?
            .join(file)
    };
    let path = path
        .canonicalize()
        .map_err(|_| "cannot resolve local schema")?;
    if !path.starts_with(root.canonicalize().map_err(|_| "missing contract root")?) {
        return Err("schema reference escapes contracts".into());
    }
    Ok((path, fragment.to_owned()))
}

pub fn bundle_schema(root: &Path, source: &Path) -> Result<Value, String> {
    fn visit(
        root: &Path,
        source: &Path,
        value: Value,
        ids: &mut BTreeMap<(PathBuf, String), String>,
        definitions: &mut Map<String, Value>,
    ) -> Result<Value, String> {
        match value {
            Value::Object(object) => {
                if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                    let target = check_reference(root, source, reference)?;
                    let key = if let Some(key) = ids.get(&target) {
                        key.clone()
                    } else {
                        let content: Value = serde_json::from_slice(
                            &fs::read(&target.0).map_err(|_| "cannot read schema")?,
                        )
                        .map_err(|_| "invalid schema JSON")?;
                        let selected = if target.1.is_empty() {
                            &content
                        } else {
                            content
                                .pointer(&target.1)
                                .ok_or("schema pointer not found")?
                        };
                        let suggested = selected
                            .get("title")
                            .and_then(Value::as_str)
                            .map(str::to_owned);
                        let key = suggested
                            .filter(|name| !ids.values().any(|v| v == name))
                            .unwrap_or_else(|| format!("Ref{}", ids.len()));
                        ids.insert(target.clone(), key.clone());
                        let bundled = visit(root, &target.0, selected.clone(), ids, definitions)?;
                        definitions.insert(key.clone(), bundled);
                        key
                    };
                    return Ok(serde_json::json!({"$ref": format!("#/definitions/{key}")}));
                }
                let mut output = Map::new();
                for (key, value) in object {
                    if key != "definitions" && key != "$id" {
                        output.insert(key, visit(root, source, value, ids, definitions)?);
                    }
                }
                Ok(Value::Object(output))
            }
            Value::Array(values) => values
                .into_iter()
                .map(|v| visit(root, source, v, ids, definitions))
                .collect(),
            _ => Ok(value),
        }
    }
    let (source, _) = check_reference(root, source, "")?;
    let value = serde_json::from_slice(&fs::read(&source).map_err(|_| "cannot read schema")?)
        .map_err(|_| "invalid schema JSON")?;
    let mut definitions = Map::new();
    let mut result = visit(root, &source, value, &mut BTreeMap::new(), &mut definitions)?;
    result["definitions"] = Value::Object(definitions);
    Ok(result)
}
