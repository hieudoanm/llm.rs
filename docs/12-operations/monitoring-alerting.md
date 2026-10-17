# Monitoring and Alerting

No Prometheus/Grafana in MVP.

- `docker compose ps` and healthchecks
- `docker compose logs -f backend`
- JSON logs with `jobId` / `requestId`
- `/healthz` and `/readyz` for Compose

Alerting is the operator watching logs. Optional later: cAdvisor. Do not claim 24/7 on-call.
