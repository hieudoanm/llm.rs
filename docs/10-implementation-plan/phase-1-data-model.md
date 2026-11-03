# Phase 1: Data model

Week 1. Alembic to [`schema.sql`](../05-data-model/sqliteql/schema.sql). `/readyz` pings Postgres. Volume mount `llm_data`.

## Acceptance

Migrations apply on backend start. pytest can insert a dataset row.
