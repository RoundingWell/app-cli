## Why

Users need a way to query artifacts from the API by type, path, and search term. This enables discovery and inspection of artifacts directly from the CLI without manual API calls.

## What Changes

- Add `artifacts list <type>` subcommand with required `--path` and `--term` options
- Fetch artifacts via `GET /artifacts?filter[type]=<type>&filter[path]=<path>&filter[term]=<term>`
- Display results as a table with columns: artifact, identifier, values

## Capabilities

### New Capabilities

- `artifacts-list`: List artifacts filtered by type, path, and search term with tabular output

### Modified Capabilities

## Impact

- New `artifacts` command group added to the CLI
- New `src/artifacts/` module with list subcommand
- Calls `GET /artifacts` endpoint with query filters
