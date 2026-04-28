# JSON:API

The RoundingWell API always sends and receives data in [JSON:API format](https://jsonapi.org/).

## Examples

A single resource response example:

```json
{
  "data": {
    "type": "clinicians",
    "id": "<uuid>",
    "attributes": {
      "name": "<string>",
      "...": "..."
    },
    "relationships": {
      "team": {
        "data": {"type": "teams", "id": "<uuid>"}
      },
      "workspaces": {
        "data": [
          {"type": "workspaces", "id": "<uuid>"},
          {"type": "workspaces", "id": "..."}
        ]
      }
    }
  }
}
```

A multiple resource response example:

```json
{
  "data": [
    {
      "type": "workspaces",
      "id": "<uuid>",
      "attributes": { "...": "..." },
      "relationships": { "...": "..." }
    }
  ]
}
```
