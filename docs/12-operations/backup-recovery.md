# Backup and Recovery

## SQLite (local)

The backend persists jobs, datasets, metrics, and artefacts in a single SQLite file
(`LLM_DATA_DIR/llm.db`) plus CSV files and ONNX artefacts under `LLM_DATA_DIR/`.

### Backup

```bash
# Stop the stack
docker compose stop backend

# Backup the SQLite database and artefacts volume
docker run --rm -v llm_data:/data -v "$(pwd)":/backup alpine tar czf /backup/llm_backup.tar.gz -C /data .

# Restart
docker compose start backend
```

### Restore

```bash
# Stop the stack
docker compose stop backend

# Wipe and restore
docker volume rm llm_data
docker volume create llm_data
docker run --rm -v llm_data:/data -v "$(pwd)":/backup alpine tar xzf /backup/llm_backup.tar.gz -C /data .

# Restart
docker compose start backend
```

### Notes

- SQLite data is portable across machines as long as the same SQLite version is used.
- The backup includes both the database file and all CSV/artefact files.
- Artefacts (ONNX files) are stored under `LLM_DATA_DIR/artefacts/` and are included in the volume backup.
