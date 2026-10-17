# mlrs — Testing Definition of Done

A change is not done until: tests for new logic (frontend Vitest units for
`src/lib` + `src/components`, backend pytest at the module and API boundary —
both under their 80% coverage gates), integration test if it touches
Postgres/files/ONNX, fixtures updated, CI green, docs if behaviour changed.
