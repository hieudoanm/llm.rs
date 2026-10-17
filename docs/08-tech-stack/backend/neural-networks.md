# mlrs — Neural networks (tabular MLPs)

The neural-net offering uses linfa/ndarray MLPs, exposed as the catalogue entries `mlp_classifier` (task `classification`) and `mlp_regressor` (task `regression`). They are multi-layer perceptrons for **tabular** data — depth, width, activation, regularisation, and early stopping are all configurable.

Riding on linfa keeps a single preprocessing + `onnx-protobuf` export path and needs no new dependency.

## Hyperparameters

Both entries share the same knobs (defaults form the shipped baseline):

| Hyperparameter        | Type            | Default        | Notes                                                        |
| --------------------- | --------------- | -------------- | ------------------------------------------------------------ |
| `hidden_layer_sizes`  | `int[]`         | `[100]`        | One integer per hidden layer; `[]` means no hidden layer     |
| `activation`          | string          | `"relu"`       | `relu`, `tanh`, `sigmoid`                                    |
| `alpha`               | number          | `0.0001`       | L2 penalty; regularises weights                              |
| `learning_rate_init`  | number          | `0.001`        | Initial step size for gradient descent                       |
| `max_iter`            | integer         | `300`          | Maximum epochs                                               |
| `batch_size`          | integer / auto  | `"auto"`       | `"auto"` = `min(200, n_samples)`                             |
| `early_stopping`      | boolean         | `false`        | Hold out `validation_fraction` to stop on no improvement     |
| `validation_fraction` | number          | `0.1`          | Fraction held out when `early_stopping=true`                 |
| `n_iter_no_change`    | integer         | `10`           | Patience for early stopping                                  |
| `tol`                 | number          | `0.0001`       | Optimisation tolerance                                       |
| `shuffle`             | boolean         | `true`         | Shuffle samples each epoch                                   |
