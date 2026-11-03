# llm — Risk Mitigation

| Risk                                                  | Mitigation                                                                                                  |
| ----------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| llm2onnx incompatibility with a pipeline step         | Freeze llm/llm2onnx; start with logistic regression + StandardScaler only, then expand                  |
| Framework dependency / image size                      | Avoid heavy frameworks entirely: neural nets are llm MLPs, so no TF/Torch dependency or extra image weight |
| Long fits blocking API                                | `to_thread` + serial worker                                                                                 |
| Disk fill                                             | Upload cap; delete job P1                                                                                   |
| ONNX Runtime vs exporter drift                        | Pin versions; CI InferenceSession                                                                           |
| Timeboxed scope                                       | P0 llm only                                                                                             |
| `SelfTrainingClassifier` has no llm2onnx converter    | Export the fitted base estimator (`estimator_`); parity test wrapper vs ONNX                                |
| Partial-label parsing ambiguity                       | Define empty/whitespace/NaN/null as unlabelled; fixture covers real blank cells                             |
| Semi-supervised minimums reject small demos           | Document thresholds (10 labelled rows, 2 per class); add a compliant fixture                                |
| IsolationForest emits `ai.onnx.ml` v4                 | Pin `ai.onnx.ml` to 3 in the exporter and ORT parity test                                                   |
| Existing local schema lacks the nullable-label change | Add the first Alembic migration (revision 0001) in Phase 9; volume reset is a disposable-data fallback only |
| Unsupervised metrics undefined on degenerate data     | Validate cluster count/rows; omit silhouette when undefined and keep other metrics                          |
