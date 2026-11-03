# llm — Infrastructure Best Practices

## Documents

| Document | Covers |
| -------- | ------ |
| [`docker.md`](./docker.md) | Dockerfiles, Compose |
| [`github-actions.md`](./github-actions.md) | CI |

## Shared rules

1. Pinned image tags. No `latest`.
2. Bind `127.0.0.1` for published ports.
3. Named volumes for Postgres and artefacts.
4. No secrets in git.
5. Healthchecks on all three services.
6. One process per container except the documented in-process worker.
7. Cloud/K8s/Terraform are out of scope.
