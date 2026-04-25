CREATE TABLE IF NOT EXISTS board_meta (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    tagline TEXT NOT NULL,
    announcement_title TEXT NOT NULL DEFAULT '',
    announcement_body TEXT NOT NULL DEFAULT '',
    smtp_host TEXT NOT NULL DEFAULT '',
    smtp_port INTEGER NOT NULL DEFAULT 587,
    smtp_user TEXT NOT NULL DEFAULT '',
    smtp_pass TEXT NOT NULL DEFAULT '',
    smtp_from_email TEXT NOT NULL DEFAULT '',
    smtp_from_name TEXT NOT NULL DEFAULT '',
    smtp_enable BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS categories (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS forums (
    id SERIAL PRIMARY KEY,
    category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    moderators TEXT[] NOT NULL DEFAULT '{}',
    sort_order INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL DEFAULT 'Member',
    status TEXT NOT NULL DEFAULT 'Online',
    joined_at TEXT NOT NULL,
    post_count INTEGER NOT NULL DEFAULT 0,
    location TEXT NOT NULL DEFAULT '',
    about TEXT NOT NULL DEFAULT '',
    last_seen TEXT NOT NULL DEFAULT 'just now',
    email TEXT NOT NULL DEFAULT '',
    password_hash TEXT NOT NULL DEFAULT '',
    group_id INTEGER NOT NULL DEFAULT 4,
    registered_at BIGINT NOT NULL DEFAULT 0,
    last_visit BIGINT NOT NULL DEFAULT 0,
    registration_ip TEXT NOT NULL DEFAULT '127.0.0.1'
);

CREATE TABLE IF NOT EXISTS topics (
    id SERIAL PRIMARY KEY,
    forum_id INTEGER NOT NULL REFERENCES forums(id) ON DELETE CASCADE,
    author_id INTEGER NOT NULL REFERENCES users(id),
    subject TEXT NOT NULL,
    closed BOOLEAN NOT NULL DEFAULT false,
    views INTEGER NOT NULL DEFAULT 0,
    tags TEXT[] NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    activity_rank INTEGER NOT NULL DEFAULT 0,
    sticky BOOLEAN NOT NULL DEFAULT false,
    moved_to INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS posts (
    id SERIAL PRIMARY KEY,
    topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
    author_id INTEGER NOT NULL REFERENCES users(id),
    posted_at TEXT NOT NULL,
    edited_at TEXT,
    body TEXT[] NOT NULL DEFAULT '{}',
    signature TEXT,
    position INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS forum_sessions (
    token TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    last_seen BIGINT NOT NULL,
    csrf_token TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS groups (
    id SERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    read_board BOOLEAN NOT NULL DEFAULT true,
    post_topics BOOLEAN NOT NULL DEFAULT true,
    post_replies BOOLEAN NOT NULL DEFAULT true,
    edit_posts BOOLEAN NOT NULL DEFAULT true,
    delete_posts BOOLEAN NOT NULL DEFAULT false,
    is_moderator BOOLEAN NOT NULL DEFAULT false,
    is_admin BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE IF NOT EXISTS bans (
    id SERIAL PRIMARY KEY,
    username TEXT DEFAULT '',
    email TEXT DEFAULT '',
    ip TEXT DEFAULT '',
    message TEXT NOT NULL DEFAULT '',
    created_at BIGINT NOT NULL DEFAULT 0,
    expires_at BIGINT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS reports (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    reporter_id INTEGER NOT NULL REFERENCES users(id),
    reason TEXT NOT NULL DEFAULT '',
    created_at BIGINT NOT NULL DEFAULT 0,
    zapped BOOLEAN NOT NULL DEFAULT false
);

CREATE INDEX IF NOT EXISTS forum_sessions_user_id_idx ON forum_sessions (user_id);
CREATE UNIQUE INDEX IF NOT EXISTS users_email_unique_idx ON users (LOWER(email));
CREATE INDEX IF NOT EXISTS bans_username_idx ON bans (username);
CREATE INDEX IF NOT EXISTS bans_email_idx ON bans (LOWER(email));
CREATE INDEX IF NOT EXISTS reports_post_id_idx ON reports (post_id);
CREATE INDEX IF NOT EXISTS reports_zapped_idx ON reports (zapped) WHERE zapped = false;

CREATE TABLE IF NOT EXISTS password_resets (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token TEXT NOT NULL UNIQUE,
    created_at BIGINT NOT NULL DEFAULT 0,
    expires_at BIGINT NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS password_resets_token_idx ON password_resets (token);
CREATE INDEX IF NOT EXISTS password_resets_expires_idx ON password_resets (expires_at);
