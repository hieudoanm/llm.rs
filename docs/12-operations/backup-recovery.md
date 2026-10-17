# Backup and Recovery

## SQLite (local)

The backend persists jobs, datasets, metrics, and artefacts in a single SQLite file
(`MLRS_DATA_DIR/mlrs.db`) plus CSV files and ONNX artefacts under `MLRS_DATA_DIR/`.

### Backup

```bash
# Stop the stack
docker compose stop backend

# Backup the SQLite database and artefacts volume
docker run --rm -v mlrs_data:/data -v "$(pwd)":/backup alpine tar czf /backup/mlrs_backup.tar.gz -C /data .

# Restart
docker compose start backend
```

### Restore

```bash
# Stop the stack
docker compose stop backend

# Wipe and restore
docker volume rm mlrs_data
docker volume create mlrs_data
docker run --rm -v mlrs_data:/data -v "$(pwd)":/backup alpine tar xzf /backup/mlrs_backup.tar.gz -C /data .

# Restart
docker compose start backend
```

### Notes

- SQLite data is portable across machines as long as the same SQLite version is used.
- The backup includes both the database file and all CSV/artefact files.
- Artefacts (ONNX files) are stored under `MLRS_DATA_DIR/artefacts/` and are included in the volume backup.
