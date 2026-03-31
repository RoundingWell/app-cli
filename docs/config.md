# Configuration

Configuration of the `rw` tool consists of multiple files:

| Path                 | Contents                                 |
|----------------------|------------------------------------------|
| `config.json`        | Tool configuration                       |
| `version_check.json` | Latest version check information         |
| `auth/*.json`        | Auth credentials per profile (mode 0600) |

By default, `rw` stores these files under `~/.config/rw/`.

### `config.json`

```json
{
  "version": "0.3.0",
  "default": "demo",
  "profiles": {
    "demo": {
      "organization": "demonstration",
      "stage": "prod"
    },
    "mercy": {
      "organization": "mercy",
      "stage": "dev"
    }
  }
}
```

### `auth/{profile}.json`

Bearer token (written after `rw auth login`):

```json
{
  "access_token": "<jwt>",
  "refresh_token": "<token>",
  "expires_at": 1234567890
}
```

Basic credentials (written using `rw config profile auth <name>`):

```json
{
  "username": "jane.doe@roundingwell.com",
  "password": "<plaintext-password>"
}
```
