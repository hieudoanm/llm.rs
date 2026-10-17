# Security (local)

- Publish ports on `127.0.0.1` only.
- SQLite data directory permissions restrict access to the backend user only.
- 10 MiB upload cap (`MLRS_MAX_UPLOAD_MB`).
- Generated storage keys.
- Uploads validated by extension, content type, and sniffed bytes — a binary payload renamed to
  `.csv` is rejected with `UNSUPPORTED_FILE_TYPE`.
- CORS restricted to the frontend origin, configured through `MLRS_CORS_ORIGINS` (default
  `http://localhost:3000,http://127.0.0.1:3000`). `*` is rejected while credentialed requests
  are enabled. See
  [`11-deployment/environment-variables.md`](../11-deployment/environment-variables.md).
- The interactive API reference (`/docs`, `/redoc`) can be turned off with
  `MLRS_ENABLE_INTERACTIVE_DOCS=false` on a hardened deployment.
- Do not train on confidential CSVs you would not store on the laptop.
- Bind-mounts should not point at `$HOME` wholesale.
