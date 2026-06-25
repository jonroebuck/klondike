CREATE TABLE IF NOT EXISTS channels (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS threads (
    id TEXT PRIMARY KEY NOT NULL,
    channel_id TEXT NOT NULL REFERENCES channels(id),
    title TEXT NOT NULL,
    author TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS posts (
    id TEXT PRIMARY KEY NOT NULL,
    thread_id TEXT NOT NULL REFERENCES threads(id),
    author TEXT NOT NULL,
    content TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS issues (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'backlog',
    assignee TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS issue_events (
    id TEXT PRIMARY KEY NOT NULL,
    issue_id TEXT NOT NULL REFERENCES issues(id),
    from_status TEXT NOT NULL,
    to_status TEXT NOT NULL,
    note TEXT,
    timestamp TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS artifacts (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    source_type TEXT NOT NULL,
    source_location TEXT NOT NULL,
    content_type TEXT NOT NULL,
    created_at TEXT NOT NULL
);
