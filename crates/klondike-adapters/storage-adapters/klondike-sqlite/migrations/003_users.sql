CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS channel_subscriptions (
    user_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    subscribed_at TEXT NOT NULL,
    PRIMARY KEY (user_id, channel_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (channel_id) REFERENCES channels(id)
);

CREATE TABLE IF NOT EXISTS thread_subscriptions (
    user_id TEXT NOT NULL,
    thread_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    subscribed_at TEXT NOT NULL,
    last_read_post_id TEXT,
    PRIMARY KEY (user_id, thread_id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (thread_id) REFERENCES threads(id)
);
