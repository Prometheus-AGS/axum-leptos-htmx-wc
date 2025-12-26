# Configuration Guide

The Universal Agent Runtime (UAR) uses a hierarchical configuration system that allows you to configure the application via:
1.  **Command Line Arguments** (highest priority)
2.  **Environment Variables**
3.  **Configuration File** (`config.yaml`)
4.  **Defaults** (lowest priority)

## 1. Command Line Arguments

Quickly override active settings when running the binary.

| Argument | Environment Variable | Description |
| :--- | :--- | :--- |
| `--config <PATH>` | `CONFIG_FILE` | Path to the configuration file. Defaults to `$HOME/.uar/config.yaml`. |
| `--port <PORT>` | `PORT` | Override the server listening port. |
| `--jwt-required` | `JWT_REQUIRED` | explicit boolean flag (e.g. `--jwt-required=false`). |
| `--rate-limit-enabled` | `RATE_LIMIT_ENABLED` | explicit boolean flag. |

Example:
```bash
./axum-leptos-htmx-wc --port 8080 --config ./my-config.yaml
```

## 2. Environment Variables

Settings are mapped to environment variables using double underscores (`__`) to separate sections.

| Config Key | Environment Variable | Default |
| :--- | :--- | :--- |
| `server.port` | `UAR_SERVER__PORT` | `3000` |
| `server.host` | `UAR_SERVER__HOST` | `0.0.0.0` |
| `security.jwt_required` | `UAR_SECURITY__JWT_REQUIRED` | `true` |
| `security.jwt_secret` | `UAR_SECURITY__JWT_SECRET` | `secret...` |
| `resilience.rate_limit_enabled` | `UAR_RESILIENCE__RATE_LIMIT_ENABLED` | `true` |
| `persistence.provider` | `UAR_PERSISTENCE__PROVIDER` | `postgres` |
| `persistence.database_url` | `UAR_PERSISTENCE__DATABASE_URL` | `postgres://...` |

### LLM Settings (Special)
LLM configuration currently resides outside the main config structure and **must** be set via these environment variables:

- `LLM_BASE_URL` (Required): API endpoint (e.g., `https://api.openai.com/v1`).
- `LLM_MODEL` (Required): Model name (e.g., `gpt-4o`).
- `LLM_API_KEY`: API Key for the provider.
- `LLM_PROTOCOL`: Client behavior (`auto`, `chat`, `responses`).


You can also place a `.env` file in the current directory to set environment variables. Default `.env` loading is supported.

## 3. Configuration File

The application looks for a configuration file in the following order:
1.  Specified via `--config` or `CONFIG_FILE`.
2.  `./config.yaml` (Current working directory).
3.  `~/.uar/config.yaml` (User home directory).

You can provide a YAML, JSON, or TOML file.

See `example.config.yaml` in the root of the repository for a complete reference.

```yaml
server:
  port: 8080
security:
  jwt_required: false
persistence:
  provider: "surreal"
  database_url: "ws://localhost:8000"
```
