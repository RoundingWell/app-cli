# Configuration

Configuration of the `rw` tool consists of multiple files:

| Path          | Contents                                            |
|---------------|-----------------------------------------------------|
| `config.json` | Tool configuration                                  |
| `auth/*.json` | Auth credentials per organization+stage (mode 0600) |

By default, `rw` stores these files under `~/.config/rw/`.

### `config.json`

```json
{
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

### `auth/{organization}-{stage}.json`

Bearer token (written after `rw auth login`):

```json
{
  "access_token": "<jwt>",
  "refresh_token": "<token>",
  "expires_at": 1234567890
}
```

Basic credentials (written using `rw basic set`):

```json
{
  "username": "jane.doe@roundingwell.com",
  "password": "<plaintext-password>"
}
```
