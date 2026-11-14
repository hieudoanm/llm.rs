//! MCP server over stdio JSON-RPC.
//!
//! Tools delegate to the `llm` binary's own subcommands so MCP behaviour
//! always matches the CLI. Configure an MCP client with: `llm mcp`

use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::process::Command;

const SERVER_INFO: &str = "llm-mcp";
const PROTOCOL_VERSION: &str = "2024-11-05";

fn arg(params: &Value, key: &str) -> Option<String> {
    params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .filter(|s| !s.is_empty())
}

fn num(params: &Value, key: &str) -> Option<String> {
    params.get(key).and_then(|v| {
        v.as_u64()
            .map(|n| n.to_string())
            .or_else(|| v.as_i64().map(|n| n.to_string()))
            .or_else(|| v.as_f64().map(|n| n.to_string()))
    })
}

fn run_cli(args: &[String]) -> Value {
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            return json!({
                "content": [{"type": "text", "text": format!("cannot locate llm binary: {err}")}],
                "isError": true
            })
        }
    };
    match Command::new(exe).args(args).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if output.status.success() {
                let text = if stdout.is_empty() { stderr } else { stdout };
                json!({"content": [{"type": "text", "text": text}]})
            } else {
                json!({
                    "content": [{"type": "text", "text": stderr}],
                    "isError": true
                })
            }
        }
        Err(err) => json!({
            "content": [{"type": "text", "text": format!("failed to run llm: {err}")}],
            "isError": true
        }),
    }
}

fn build_args(name: &str, params: &Value) -> Option<Vec<String>> {
    match name {
        "list_algorithms" => {
            let mut args = vec!["algorithms".to_string()];
            if let Some(task) = arg(params, "task") {
                args.extend(["--task".to_string(), task]);
            }
            Some(args)
        }
        "register_dataset" => Some(vec!["dataset".to_string(), arg(params, "path")?]),
        "train" => {
            let mut args = vec![
                "train".to_string(),
                "--dataset-id".to_string(),
                arg(params, "dataset_id")?,
                "--task".to_string(),
                arg(params, "task")?,
                "--algorithm".to_string(),
                arg(params, "algorithm")?,
                "--features".to_string(),
                arg(params, "features")?,
            ];
            if let Some(v) = arg(params, "label") {
                args.extend(["--label".to_string(), v]);
            }
            if let Some(v) = arg(params, "hyperparameters") {
                args.extend(["--hyperparameters".to_string(), v]);
            }
            if let Some(v) = num(params, "split_ratio") {
                args.extend(["--split-ratio".to_string(), v]);
            }
            if let Some(v) = num(params, "random_seed") {
                args.extend(["--random-seed".to_string(), v]);
            }
            Some(args)
        }
        "get_job" => Some(vec!["job".to_string(), arg(params, "id")?]),
        "export_model" => {
            let mut args = vec![
                "export".to_string(),
                "--job-id".to_string(),
                arg(params, "job_id")?,
                "--output".to_string(),
                arg(params, "output")?,
            ];
            if let Some(v) = arg(params, "format") {
                args.extend(["--format".to_string(), v]);
            }
            Some(args)
        }
        "list_jobs" => Some(vec!["jobs".to_string()]),
        "list_datasets" => Some(vec!["datasets".to_string()]),
        _ => None,
    }
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "list_algorithms",
                "description": "List available ML algorithms, optionally filtered by task type",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task": {"type": "string", "description": "Task type filter (classification, regression, semi_supervised, clustering, anomaly_detection, reinforcement_learning)"}
                    },
                    "additionalProperties": false
                }
            },
            {
                "name": "register_dataset",
                "description": "Upload/register a CSV dataset",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string", "description": "Path to CSV file"}
                    },
                    "required": ["path"],
                    "additionalProperties": false
                }
            },
            {
                "name": "train",
                "description": "Train a model from a registered dataset and export ONNX",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "dataset_id": {"type": "string", "description": "Dataset ID (from register_dataset)"},
                        "task": {"type": "string", "description": "Task type (classification, regression, semi_supervised, clustering, anomaly_detection, reinforcement_learning)"},
                        "algorithm": {"type": "string", "description": "Algorithm ID (see list_algorithms)"},
                        "features": {"type": "string", "description": "Feature column names (comma-separated)"},
                        "label": {"type": "string", "description": "Label column name (required for classification/regression/semi_supervised)"},
                        "hyperparameters": {"type": "string", "description": "Hyperparameters as JSON string"},
                        "split_ratio": {"type": "number", "description": "Train/test split ratio (0.0-1.0, default 0.2)"},
                        "random_seed": {"type": "integer", "description": "Random seed (default 42)"}
                    },
                    "required": ["dataset_id", "task", "algorithm", "features"],
                    "additionalProperties": false
                }
            },
            {
                "name": "get_job",
                "description": "Show job status, metrics, and artefacts",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": {"type": "string", "description": "Job ID"}
                    },
                    "required": ["id"],
                    "additionalProperties": false
                }
            },
            {
                "name": "export_model",
                "description": "Export a job's artefact to a file",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "job_id": {"type": "string", "description": "Job ID"},
                        "output": {"type": "string", "description": "Output file path"},
                        "format": {"type": "string", "description": "Format: onnx, pkl, joblib (default onnx)"}
                    },
                    "required": ["job_id", "output"],
                    "additionalProperties": false
                }
            },
            {
                "name": "list_jobs",
                "description": "List all training jobs",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }
            },
            {
                "name": "list_datasets",
                "description": "List all stored datasets",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }
            }
        ]
    })
}

fn handle_request(request: &Value) -> Option<Value> {
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let has_id = request.get("id").is_some();

    let result = match method {
        "initialize" => json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": {"tools": {}},
            "serverInfo": {"name": SERVER_INFO, "version": env!("CARGO_PKG_VERSION")}
        }),
        "ping" => Value::Null,
        "tools/list" => tools_list(),
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or(Value::Null);
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));
            match build_args(name, &arguments) {
                Some(args) => run_cli(&args),
                None => json!({
                    "content": [{"type": "text", "text": format!("Unknown tool or missing required arguments: {name}")}],
                    "isError": true
                }),
            }
        }
        _ => {
            if !has_id {
                return None;
            }
            return Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("Method not found: {method}")}
            }));
        }
    };

    if !has_id {
        return None;
    }
    Some(json!({"jsonrpc": "2.0", "id": id, "result": result}))
}

/// Run the MCP server on stdin/stdout until EOF.
pub fn run() -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(trimmed) {
            Ok(request) => {
                if let Some(response) = handle_request(&request) {
                    writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                    stdout.flush()?;
                }
            }
            Err(err) => {
                let response = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {"code": -32700, "message": format!("Parse error: {err}")}
                });
                writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
                stdout.flush()?;
            }
        }
    }
    Ok(())
}
