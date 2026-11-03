# llm — serde

## Usage in llm

serde is the **validation and serialisation layer** for the entire API:

- All request/response bodies are typed serde structs/enums
- Axum derives the OpenAPI contract from these types via `utoipa`
- Models live in `crates/llm-api/src/schemas.rs`
- Every endpoint handler uses typed parameters and response types

## Model catalogue

| Model | Purpose | Endpoint |
|-------|---------|----------|
| `HealthResponse`, `ReadyResponse` | Liveness/readiness probes | `GET /healthz`, `GET /readyz` |
| `DatasetResponse`, `ColumnPreview` | CSV upload result + metadata | `POST/GET /api/v1/datasets` |
| `AlgorithmResponse`, `AlgorithmsListResponse` | Algorithm catalogue | `GET /api/v1/algorithms` |
| `CreateJobRequest` | Enqueue training (validated) | `POST /api/v1/jobs` |
| `JobAcceptedResponse` | 202 response | `POST /api/v1/jobs`, `GET /api/v1/jobs` (list) |
| `JobResponse` | Full job detail | `GET /api/v1/jobs/{jobId}` |
| `JobListResponse` | Paginated history | `GET /api/v1/jobs` |
| `ArtefactResponse` | Download metadata | `GET /api/v1/jobs/{jobId}/artefacts/{artefactId}` |
| `RLColumns` | RL experience columns | Nested in `CreateJobRequest` |
| `ErrorDetail`, `ErrorEnvelope` | Unified error shape | All error responses |

## Enums (mirror OpenAPI)

```rust
#[derive(Serialize, Deserialize)]
enum TaskType {
    Classification,
    Regression,
    SemiSupervised,
    Clustering,
    AnomalyDetection,
    ReinforcementLearning,
}

#[derive(Serialize, Deserialize)]
enum JobStatus {
    Queued,
    Validating,
    Training,
    Evaluating,
    Exporting,
    Completed,
    Failed,
}

#[derive(Serialize, Deserialize)]
enum ArtefactFormat {
    Onnx,
}
```

## Cross-field validation

`CreateJobRequest` uses custom validation to enforce label/rlColumns rules per task type:

```rust
impl Validate for CreateJobRequest {
    fn validate(&self) -> Result<(), ValidationError> {
        let label_required = [TaskType::Classification, TaskType::Regression, TaskType::SemiSupervised];
        let label_forbidden = [TaskType::Clustering, TaskType::AnomalyDetection, TaskType::ReinforcementLearning];

        if label_required.contains(&self.task_type) && self.label_column.is_none() {
            return Err(ValidationError::label_required(self.task_type));
        }
        if label_forbidden.contains(&self.task_type) && self.label_column.is_some() {
            return Err(ValidationError::label_forbidden(self.task_type));
        }
        // ... RL rules
        Ok(())
    }
}
```

This keeps validation **at the boundary** — invalid payloads are rejected before reaching business logic.

## Configuration

```rust
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateJobRequest {
    // ...
}
```

- `deny_unknown_fields` on `CreateJobRequest` and the nested `RLColumns` — unknown keys are rejected with `400 VALIDATION_ERROR`
- Field-level constraints via `validator` crate (`range`, `length`, etc.)
- `Uuid` and `DateTime` types auto-coerce from strings

## Error envelope

Request-validation failures are normalised into the same flat envelope as application errors by the handlers:

```json
{ "error": { "code": "VALIDATION_ERROR", "message": "...", "request_id": "uuid", "details": { "fields": [] } } }
```

## Testing

- Unit tests construct `CreateJobRequest` directly to verify validation
- Error messages are asserted via `assert_eq!`
- E2E tests hit the real Axum endpoints (OpenAPI contract verified)

## Version pin

`serde` in `crates/Cargo.toml` (via `Cargo.lock`).

## Rejected alternatives

| Alternative | Reason |
|-------------|--------|
| Manual validation | Error-prone, no contract |
| Raw Value + manual parsing | Verbose, no type safety |
| Pydantic | Python-only, not available in Rust |