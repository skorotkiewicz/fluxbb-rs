INSERT INTO board_meta (id, title, tagline, announcement_title, announcement_body)
VALUES (
    1,
    'FluxBB RS',
    'FluxBB reimagined as a Dioxus 0.7 forum shell with Postgres-backed content.',
    'Migration alpha',
    'The board layout, topic browsing, member directory, and search now live in Rust. Posting and moderation flows are staged for the next slice.'
)
ON CONFLICT (id) DO UPDATE SET
    title = EXCLUDED.title,
    tagline = EXCLUDED.tagline,
    announcement_title = EXCLUDED.announcement_title,
    announcement_body = EXCLUDED.announcement_body;

INSERT INTO categories (id, name, description, sort_order) VALUES
    (1, 'Announcements', 'Project direction, release notes, and migration status.', 1),
    (2, 'Community', 'Discussion spaces that mirror the core public FluxBB experience.', 2),
    (3, 'Workshop', 'Implementation notes for the Dioxus and Rust rewrite.', 3)
ON CONFLICT (id) DO UPDATE SET
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    sort_order = EXCLUDED.sort_order;

INSERT INTO forums (id, category_id, name, description, moderators, sort_order) VALUES
    (1, 1, 'Release Notes', 'Track each migration milestone and the current web parity status.', ARRAY['nora'], 1),
    (2, 1, 'Migration Lab', 'Patterns for moving classic FluxBB screens into RSX and fullstack Rust.', ARRAY['nora', 'ellis'], 2),
    (3, 2, 'General Discussion', 'Open-ended board conversation and showcase threads.', ARRAY['rhea'], 1),
    (4, 2, 'Support', 'Configuration, deployment, and migration troubleshooting.', ARRAY['kai'], 2),
    (5, 3, 'Themes and Extensions', 'Style ports, plugin ideas, and frontend experiments.', ARRAY['mira'], 1),
    (6, 3, 'Backend Planning', 'Database, auth, and moderation architecture for the Rust stack.', ARRAY['ellis'], 2)
ON CONFLICT (id) DO UPDATE SET
    category_id = EXCLUDED.category_id,
    name = EXCLUDED.name,
    description = EXCLUDED.description,
    moderators = EXCLUDED.moderators,
    sort_order = EXCLUDED.sort_order;

INSERT INTO users (
    id, username, title, status, joined_at, post_count, location, about, last_seen,
    email, password_hash, group_id, registered_at, last_visit, registration_ip
) VALUES
    (1, 'nora', 'Administrator', 'Online', '2026-04-01', 482, 'Berlin', 'Maintains the Dioxus rewrite and release cadence.', 'a minute ago', 'nora@example.com', 'sha256$boardseed$9491640339c346d357a0451891793308c0057ac7e65f52d15cf38b62cccdb19b', 1, 1713744000, 1714074000, '127.0.0.1'),
    (2, 'ellis', 'Core Maintainer', 'Online', '2026-04-02', 318, 'Copenhagen', 'Focuses on schema design, hydration safety, and deployment ergonomics.', '5 minutes ago', 'ellis@example.com', 'sha256$boardseed$9491640339c346d357a0451891793308c0057ac7e65f52d15cf38b62cccdb19b', 4, 1713830400, 1714074000, '127.0.0.1'),
    (3, 'rhea', 'Moderator', 'Reviewing', '2026-04-03', 211, 'Amsterdam', 'Moderation lead and theme curator.', '20 minutes ago', 'rhea@example.com', 'sha256$boardseed$9491640339c346d357a0451891793308c0057ac7e65f52d15cf38b62cccdb19b', 2, 1713916800, 1714074000, '127.0.0.1'),
    (4, 'kai', 'Support Lead', 'Available', '2026-04-05', 177, 'Lisbon', 'Handles onboarding, FAQ cleanup, and migration support.', '12 minutes ago', 'kai@example.com', 'sha256$boardseed$9491640339c346d357a0451891793308c0057ac7e65f52d15cf38b62cccdb19b', 4, 1714089600, 1714074000, '127.0.0.1'),
    (5, 'mira', 'Theme Builder', 'Designing', '2026-04-09', 124, 'Prague', 'Ports classic board themes into the new CSS system.', 'an hour ago', 'mira@example.com', 'sha256$boardseed$9491640339c346d357a0451891793308c0057ac7e65f52d15cf38b62cccdb19b', 4, 1714435200, 1714074000, '127.0.0.1'),
    (6, 'sol', 'Member', 'Reading', '2026-04-20', 14, 'Warsaw', 'Testing the first public preview and reporting rough edges.', 'just now', 'sol@example.com', 'sha256$boardseed$9491640339c346d357a0451891793308c0057ac7e65f52d15cf38b62cccdb19b', 4, 1715472000, 1714074000, '127.0.0.1')
ON CONFLICT (id) DO UPDATE SET
    username = EXCLUDED.username,
    title = EXCLUDED.title,
    status = EXCLUDED.status,
    joined_at = EXCLUDED.joined_at,
    post_count = EXCLUDED.post_count,
    location = EXCLUDED.location,
    about = EXCLUDED.about,
    last_seen = EXCLUDED.last_seen,
    email = EXCLUDED.email,
    password_hash = EXCLUDED.password_hash,
    group_id = EXCLUDED.group_id,
    registered_at = EXCLUDED.registered_at,
    last_visit = EXCLUDED.last_visit,
    registration_ip = EXCLUDED.registration_ip;

CREATE UNIQUE INDEX IF NOT EXISTS users_email_unique_idx ON users (LOWER(email));

INSERT INTO topics (id, forum_id, author_id, subject, status, views, tags, created_at, updated_at, activity_rank) VALUES
    (101, 1, 1, '0.1 migration alpha is live', 'pinned', 934, ARRAY['release', 'migration'], '2026-04-20 09:15 UTC', '2026-04-23 19:40 UTC', 400),
    (102, 2, 2, 'Mapping FluxBB templates to RSX', 'hot', 486, ARRAY['rsx', 'layout'], '2026-04-19 13:10 UTC', '2026-04-23 18:05 UTC', 390),
    (201, 3, 3, 'Show your forum theme experiments', 'fresh', 212, ARRAY['theme', 'showcase'], '2026-04-18 17:45 UTC', '2026-04-22 16:55 UTC', 360),
    (202, 4, 4, 'Session strategy for web-only deploys', 'resolved', 305, ARRAY['auth', 'deployment'], '2026-04-17 10:20 UTC', '2026-04-22 11:35 UTC', 340),
    (301, 5, 5, 'Theme pack wishlist', 'hot', 267, ARRAY['css', 'theme'], '2026-04-16 15:00 UTC', '2026-04-21 20:20 UTC', 320),
    (302, 6, 2, 'Persistence layer options for a post-PHP stack', 'pinned', 522, ARRAY['postgres', 'architecture'], '2026-04-15 08:55 UTC', '2026-04-23 17:25 UTC', 380)
ON CONFLICT (id) DO UPDATE SET
    forum_id = EXCLUDED.forum_id,
    author_id = EXCLUDED.author_id,
    subject = EXCLUDED.subject,
    status = EXCLUDED.status,
    views = EXCLUDED.views,
    tags = EXCLUDED.tags,
    created_at = EXCLUDED.created_at,
    updated_at = EXCLUDED.updated_at,
    activity_rank = EXCLUDED.activity_rank;

INSERT INTO posts (id, topic_id, author_id, posted_at, edited_at, body, signature, position) VALUES
    (1001, 101, 1, '2026-04-20 09:15 UTC', '2026-04-23 19:05 UTC', ARRAY['The first Rust and Dioxus web slice is up. Board index, forum view, topic view, search, and member directory are all wired.', 'Next up are posting workflows, session-backed authentication, and moderation actions.'], 'Keep migrations boring.', 1),
    (1002, 101, 6, '2026-04-23 19:40 UTC', NULL, ARRAY['The browsing flow feels solid. The search page made it much easier to see where the PHP information architecture landed in the new app.'], NULL, 2),
    (1101, 102, 2, '2026-04-19 13:10 UTC', NULL, ARRAY['The old header and body templates map cleanly to a shared shell plus route pages. The tricky part is keeping table-heavy layouts readable on mobile.'], NULL, 1),
    (1102, 102, 5, '2026-04-23 18:05 UTC', NULL, ARRAY['I had the best results by preserving the forum-table mental model but shifting the last-post metadata into a stacked layout on narrow widths.'], 'Visual parity matters, but not at the expense of scan speed.', 2),
    (1201, 201, 3, '2026-04-18 17:45 UTC', NULL, ARRAY['If you are testing theme ideas, post screenshots and note whether you are optimizing for nostalgia, readability, or moderation-heavy workflows.'], NULL, 1),
    (1202, 201, 5, '2026-04-22 16:55 UTC', NULL, ARRAY['Copper accents plus parchment cards ended up closer to the old FluxBB tone than another dark dashboard treatment.'], NULL, 2),
    (1301, 202, 4, '2026-04-17 10:20 UTC', NULL, ARRAY['For the first web deploy, I would keep auth server-backed and session cookie based. JWT would only add edge cases we do not need yet.'], NULL, 1),
    (1302, 202, 2, '2026-04-22 11:35 UTC', NULL, ARRAY['Agreed. The app shell already centralizes enough data that classic session handling will be the lowest-risk path for the next increment.'], NULL, 2),
    (1401, 301, 5, '2026-04-16 15:00 UTC', NULL, ARRAY['Current shortlist: Air-inspired light theme, a sharper editorial variant, and a contrast-first moderation skin.'], NULL, 1),
    (1402, 301, 3, '2026-04-21 20:20 UTC', NULL, ARRAY['Please keep moderation cues strong. Reports, locked topics, and read state should stay obvious at a glance.'], NULL, 2),
    (1501, 302, 2, '2026-04-15 08:55 UTC', '2026-04-23 16:10 UTC', ARRAY['The migration stack needs normalized categories, forums, topics, posts, and users first. Anything more can layer on after the public flows are stable.', 'Postgres is enough for the first backend pass; the schema should stay friendly to future search indexing and moderation queues.'], NULL, 1),
    (1502, 302, 1, '2026-04-23 17:25 UTC', NULL, ARRAY['That is the direction I am landing in this slice as well. Keep the schema obvious, then layer richer server functions on top.'], 'Correctness before cleverness.', 2)
ON CONFLICT (id) DO UPDATE SET
    topic_id = EXCLUDED.topic_id,
    author_id = EXCLUDED.author_id,
    posted_at = EXCLUDED.posted_at,
    edited_at = EXCLUDED.edited_at,
    body = EXCLUDED.body,
    signature = EXCLUDED.signature,
    position = EXCLUDED.position;
