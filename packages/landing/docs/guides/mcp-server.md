# MCP server

`llm` ships a [Model Context Protocol](https://modelcontextprotocol.io) server
over stdio. Agents can drive the full local training workflow — datasets,
algorithms, jobs, and ONNX export — through typed tools.

## Run

```sh
llm mcp
```

The server reads newline-delimited JSON-RPC on stdin and writes responses on
stdout. It delegates every tool call to the `llm` binary itself, so jobs land in
the same SQLite database and storage as CLI runs.

## Client configuration

```json
{
  "mcpServers": {
    "llm": {
      "command": "llm",
      "args": ["mcp"]
    }
  }
}
```

The data directory resolves from `LLM_DATA_DIR` exactly as the CLI.

## Tools

| Tool               | Description                                            |
| ------------------ | ------------------------------------------------------ |
| `list_algorithms`  | Available algorithms, optionally filtered by task type |
| `register_dataset` | Upload/register a CSV dataset                          |
| `train`            | Train a model from a dataset and export ONNX           |
| `get_job`          | Job status, metrics, and artefacts                     |
| `export_model`     | Export a job's artefact to a file                      |
| `list_jobs`        | List all training jobs                                 |
| `list_datasets`    | List all stored datasets                               |

See the [API reference](../api-reference/api.md) and
[Tasks and algorithms](../concepts/tasks-and-algorithms.md) for the underlying
model.
