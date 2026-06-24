# Klondike

A lightweight, self-hosted collaboration platform for humans and agents. Manages threads, issues, and artifacts through a clean REST API, with pluggable backends via [Switchboard](https://github.com/yourusername/switchboard).

## Features

- **Threads** — channels with persistent, append-only post history
- **Issues** — lightweight kanban (backlog, in_progress, done, blocked) with full event history
- **Artifacts** — versioned artifact storage with pluggable backends (SQLite, SharePoint, Google Docs, and more via Switchboard)

## Quick Start

```bash
docker compose up -d
```

Klondike will be available at `http://localhost:3000`.

## API

### Threads

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/channels` | List channels |
| POST | `/api/v1/channels` | Create channel |
| GET | `/api/v1/channels/:id` | Get channel |
| GET | `/api/v1/channels/:id/threads` | List threads |
| POST | `/api/v1/channels/:id/threads` | Create thread |
| GET | `/api/v1/threads/:id/posts` | List posts |
| POST | `/api/v1/threads/:id/posts` | Append post |

### Issues

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/issues` | List issues |
| POST | `/api/v1/issues` | Create issue |
| GET | `/api/v1/issues/:id` | Get issue |
| PATCH | `/api/v1/issues/:id/status` | Update status |
| GET | `/api/v1/issues/:id/events` | List events |

### Artifacts

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/artifacts` | List artifacts |
| POST | `/api/v1/artifacts` | Write artifact |
| GET | `/api/v1/artifacts/:id` | Get artifact metadata |
| GET | `/api/v1/artifacts/:id/content` | Read artifact content |

## Architecture

Klondike follows a ports and adapters pattern:

- **klondike-core** — traits and domain types (published to crates.io)
- **klondike-sqlite** — SQLite storage adapter
- **klondike-rest** — Axum REST API adapter
- **klondike** — wires everything together
- **klondike-server** — thin binary entrypoint

Additional adapters (Slack, GitHub, Jira, SharePoint, etc.) are available via [Switchboard](https://github.com/yourusername/switchboard).

## Configuration

| Environment Variable | Default | Description |
|---------------------|---------|-------------|
| `DATABASE_URL` | `sqlite:///data/klondike.db` | SQLite database path |
| `PORT` | `3000` | HTTP port |

## License

MIT
