## Context

The CLI already supports `teams list` via `GET /teams`. There is no `GET /teams/:id` endpoint, so showing a single team must use list + client-side match. This is the same pattern used by `workspaces show` (match by id or slug) and `roles show` (match by id or name).

## Goals / Non-Goals

**Goals:**
- Add `rw teams show <target>` matching on team `id` or `abbr`
- Display team `id`, `abbr`, and `name` in plain text
- Support `--json` output
- Follow existing show-command conventions

**Non-Goals:**
- Pagination (teams list is not paginated)
- Filtering or fuzzy matching

## Decisions

**List + match instead of direct fetch**
There is no `GET /teams/:id` endpoint. `GET /teams` returns all teams and is already used by `teams list`. We reuse the same fetch and match client-side — consistent with how `workspaces show` and `roles show` work.

**Match on `id` or `abbr`**
Team abbreviations (`abbr`) are the human-readable short identifiers users know. Matching on both `id` and `abbr` mirrors how `roles show` matches on id or name, and `workspaces show` matches on id or slug.

**Reuse existing `get_teams` / list infrastructure**
The teams module already has a fetch function. `teams show` will call it and filter, keeping list and show logic co-located in the teams module.

## Risks / Trade-offs

- **Extra API call vs. direct fetch** → Acceptable; teams lists are small and there is no alternative endpoint
