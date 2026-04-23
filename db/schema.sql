CREATE TABLE IF NOT EXISTS board_meta (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    tagline TEXT NOT NULL,
    announcement_title TEXT NOT NULL,
    announcement_body TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS forums (
    id INTEGER PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories(id),
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    moderators TEXT[] NOT NULL DEFAULT '{}',
    sort_order INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    joined_at TEXT NOT NULL,
    post_count INTEGER NOT NULL DEFAULT 0,
    location TEXT NOT NULL,
    about TEXT NOT NULL,
    last_seen TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS topics (
    id INTEGER PRIMARY KEY,
    forum_id INTEGER NOT NULL REFERENCES forums(id),
    author_id INTEGER NOT NULL REFERENCES users(id),
    subject TEXT NOT NULL,
    status TEXT NOT NULL,
    views INTEGER NOT NULL DEFAULT 0,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    activity_rank INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS posts (
    id INTEGER PRIMARY KEY,
    topic_id INTEGER NOT NULL REFERENCES topics(id),
    author_id INTEGER NOT NULL REFERENCES users(id),
    posted_at TEXT NOT NULL,
    edited_at TEXT,
    body TEXT[] NOT NULL DEFAULT '{}',
    signature TEXT,
    position INTEGER NOT NULL
);
