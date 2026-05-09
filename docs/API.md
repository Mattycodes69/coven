# Coven Local API

_Last updated: 2026-05-09_

Coven exposes a small HTTP API over the local Unix socket at `<covenHome>/coven.sock`. The Rust daemon is the authority boundary: clients may validate for UX, but the daemon still validates project roots, cwd, harness ids, session ids, input, and live-session state before acting.

## Versioning

The current public API contract is **`v1`**.

Versioned clients should use the `/api/v1` prefix:

| Endpoint | Purpose |
|---|---|
| `GET /api/v1/api-version` | Read the active API version and supported versions |
| `GET /api/v1/health` | Check daemon health and metadata |
| `GET /api/v1/sessions` | List active sessions |
| `POST /api/v1/sessions` | Launch a session |
| `GET /api/v1/sessions/:id` | Fetch one session |
| `GET /api/v1/events?sessionId=...` | Read session events |
| `POST /api/v1/sessions/:id/input` | Forward input to a live session |
| `POST /api/v1/sessions/:id/kill` | Kill a live session |

Unversioned routes currently remain as legacy aliases during the early MVP window, but new clients should not rely on them.

Unknown `/api/<version>/...` prefixes fail closed with an `unsupported API version` JSON response.

## Health response

`GET /api/v1/health` returns the API version alongside daemon status:

```json
{
  "apiVersion": "v1",
  "supportedApiVersions": ["v1"],
  "ok": true,
  "daemon": {
    "pid": 12345,
    "startedAt": "2026-05-09T12:00:00Z",
    "socket": "/Users/example/.coven/coven.sock"
  }
}
```

When no daemon metadata is available, `daemon` is `null`.

## Compatibility rules

- Additive JSON fields are allowed in `v1` responses.
- Existing required fields should not be removed or renamed inside `v1`.
- Breaking response-shape or behavior changes require a new API version prefix.
- External clients should call `/api/v1/health` before assuming compatibility.
- Daemon changes that affect `/api/v1/health`, `/api/v1/sessions`, `/api/v1/events`, input, or kill behavior should update client compatibility tests in the same repo.
