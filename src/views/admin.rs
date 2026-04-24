use dioxus::prelude::*;

use crate::{
    components::SectionHeader,
    data::{
        add_ban, admin_add_category, admin_add_forum, admin_delete_category, admin_delete_forum,
        admin_delete_topic, admin_delete_user, admin_update_board, admin_update_topic,
        admin_update_user, clean_error, load_admin_data, load_bans, load_groups, remove_ban,
        update_group, AdminBoardSettings, AdminCategoryForm, AdminData, AdminDeleteItem,
        AdminForumForm, AdminTopicUpdate, AdminUserUpdate, Ban, BanForm, Group, GroupUpdateForm,
        SessionUser,
    },
    Route,
};

#[component]
pub fn Admin() -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let refresh = use_context::<Signal<()>>();

    let data_resource = use_resource(move || async move {
        refresh();
        load_admin_data().await
    });

    let is_admin = current_user()
        .as_ref()
        .is_some_and(|u| u.group_id == 1);

    let mut tab = use_signal(|| "structure");
    let status = use_signal(String::new);
    let is_error = use_signal(|| false);

    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Admin".to_string(),
                    title: "Control panel".to_string(),
                    subtitle: "Board administration and moderation tools.".to_string(),
                }
            }

            if !is_admin {
                article { class: "empty-state",
                    h3 { "Access denied" }
                    p { "You must be an administrator to view this page." }
                }
            } else if let Some(Ok(data)) = data_resource() {
                nav { class: "admin-tabs",
                    button {
                        class: if tab() == "structure" { "admin-tab active" } else { "admin-tab" },
                        onclick: move |_| tab.set("structure"),
                        "Structure"
                    }
                    button {
                        class: if tab() == "users" { "admin-tab active" } else { "admin-tab" },
                        onclick: move |_| tab.set("users"),
                        "Users"
                    }
                    button {
                        class: if tab() == "moderation" { "admin-tab active" } else { "admin-tab" },
                        onclick: move |_| tab.set("moderation"),
                        "Moderation"
                    }
                    button {
                        class: if tab() == "settings" { "admin-tab active" } else { "admin-tab" },
                        onclick: move |_| tab.set("settings"),
                        "Settings"
                    }
                    button {
                        class: if tab() == "bans" { "admin-tab active" } else { "admin-tab" },
                        onclick: move |_| tab.set("bans"),
                        "Bans"
                    }
                    button {
                        class: if tab() == "groups" { "admin-tab active" } else { "admin-tab" },
                        onclick: move |_| tab.set("groups"),
                        "Groups"
                    }
                }

                if !status().is_empty() {
                    p { class: if is_error() { "form-message form-error" } else { "form-message form-success" },
                        "{status}"
                    }
                }

                if tab() == "structure" {
                    StructurePanel {
                        data: data.clone(),
                        status,
                        is_error,
                        refresh,
                    }
                } else if tab() == "users" {
                    UsersPanel {
                        data: data.clone(),
                        status,
                        is_error,
                        refresh,
                    }
                } else if tab() == "moderation" {
                    ModerationPanel {
                        data: data.clone(),
                        status,
                        is_error,
                        refresh,
                    }
                } else if tab() == "settings" {
                    SettingsPanel {
                        data: data.clone(),
                        status,
                        is_error,
                        refresh,
                    }
                } else if tab() == "bans" {
                    BansPanel { status, is_error, refresh }
                } else if tab() == "groups" {
                    GroupsPanel { status, is_error, refresh }
                }
            } else {
                article { class: "empty-state",
                    h3 { "Loading admin data…" }
                }
            }
        }
    }
}

// ── Structure panel: categories & forums ──

#[component]
fn StructurePanel(
    data: AdminData,
    mut status: Signal<String>,
    mut is_error: Signal<bool>,
    mut refresh: Signal<()>,
) -> Element {
    let mut cat_name = use_signal(String::new);
    let mut cat_desc = use_signal(String::new);
    let mut forum_name = use_signal(String::new);
    let mut forum_desc = use_signal(String::new);
    let mut forum_cat_id = use_signal(|| 0_i32);

    let mut categories = data.categories.clone();
    categories.sort_by_key(|c| c.sort_order);
    let forums = data.forums.clone();

    let cat_items: Vec<_> = categories
        .iter()
        .map(|cat| {
            let cat_forums: Vec<_> = forums
                .iter()
                .filter(|f| f.category_id == cat.id)
                .cloned()
                .collect();
            (cat.clone(), cat_forums)
        })
        .filter(|(_, cat_forums)| !cat_forums.is_empty())
        .collect();

    rsx! {
        // ── Existing categories & forums ──
        for (cat, cat_forums) in cat_items {
            article { class: "panel",
                div { class: "panel-heading",
                    h3 { "{cat.name}" }
                    p { "{cat.description}" }
                    button {
                        class: "danger-button small-button",
                        onclick: {
                            let cid = cat.id;
                            move |_| {
                                spawn(async move {
                                    match admin_delete_category(AdminDeleteItem { id: cid }).await {
                                        Ok(_) => {
                                            is_error.set(false);
                                            status.set("Category deleted. Refresh to see changes.".into());
                                        }
                                        Err(e) => {
                                            is_error.set(true);
                                            status.set(clean_error(e));
                                        }
                                    }
                                });
                            }
                        },
                        "Delete category"
                    }
                }
                for forum in cat_forums {
                    div { class: "forum-row",
                        div { class: "forum-main",
                            Link {
                                class: "forum-link",
                                to: Route::ForumPage {
                                    id: forum.id,
                                    page: 1,
                                },
                                "{forum.name}"
                            }
                            p { class: "forum-description", "{forum.description}" }
                        }
                        button {
                            class: "danger-button small-button",
                            onclick: {
                                let fid = forum.id;
                                move |_| {
                                    spawn(async move {
                                        match admin_delete_forum(AdminDeleteItem { id: fid }).await {
                                            Ok(_) => {
                                                is_error.set(false);
                                                status.set("Forum deleted. Refresh to see changes.".into());
                                            }
                                            Err(e) => {
                                                is_error.set(true);
                                                status.set(clean_error(e));
                                            }
                                        }
                                    });
                                }
                            },
                            "Delete"
                        }
                    }
                }
            }
        }

        // ── Add category ──
        article { class: "form-card",
            h3 { "Add category" }
            label {
                "Name"
                input {
                    class: "text-input",
                    value: "{cat_name}",
                    oninput: move |e| cat_name.set(e.value()),
                    placeholder: "Category name",
                }
            }
            label {
                "Description"
                input {
                    class: "text-input",
                    value: "{cat_desc}",
                    oninput: move |e| cat_desc.set(e.value()),
                    placeholder: "Short description",
                }
            }
            button {
                class: "primary-button",
                onclick: move |_| {
                    let form = AdminCategoryForm {
                        name: cat_name(),
                        description: cat_desc(),
                    };
                    spawn(async move {
                        match admin_add_category(form).await {
                            Ok(_) => {
                                is_error.set(false);
                                status.set("Category created. Refresh to see it.".into());
                                cat_name.set(String::new());
                                cat_desc.set(String::new());
                            }
                            Err(e) => {
                                is_error.set(true);
                                status.set(clean_error(e));
                            }
                        }
                    });
                },
                "Create category"
            }
        }

        // ── Add forum ──
        article { class: "form-card",
            h3 { "Add forum" }
            label {
                "Category"
                select {
                    class: "text-input",
                    onchange: move |e| {
                        if let Ok(v) = e.value().parse::<i32>() {
                            forum_cat_id.set(v);
                        }
                    },
                    option { value: "0", "Select category…" }
                    for cat in categories.clone() {
                        option { value: "{cat.id}", "{cat.name}" }
                    }
                }
            }
            label {
                "Forum name"
                input {
                    class: "text-input",
                    value: "{forum_name}",
                    oninput: move |e| forum_name.set(e.value()),
                    placeholder: "Forum name",
                }
            }
            label {
                "Description"
                input {
                    class: "text-input",
                    value: "{forum_desc}",
                    oninput: move |e| forum_desc.set(e.value()),
                    placeholder: "Short description",
                }
            }
            button {
                class: "primary-button",
                onclick: move |_| {
                    let form = AdminForumForm {
                        category_id: forum_cat_id(),
                        name: forum_name(),
                        description: forum_desc(),
                    };
                    spawn(async move {
                        match admin_add_forum(form).await {
                            Ok(_) => {
                                is_error.set(false);
                                status.set("Forum created. Refresh to see it.".into());
                                forum_name.set(String::new());
                                forum_desc.set(String::new());
                            }
                            Err(e) => {
                                is_error.set(true);
                                status.set(clean_error(e));
                            }
                        }
                    });
                },
                "Create forum"
            }
        }
    }
}

// ── Users panel ──

#[component]
fn UsersPanel(
    data: AdminData,
    mut status: Signal<String>,
    mut is_error: Signal<bool>,
    mut refresh: Signal<()>,
) -> Element {
    fn group_label(gid: i32) -> &'static str {
        match gid {
            1 => "Admin",
            2 => "Moderator",
            3 => "Guest",
            _ => "Member",
        }
    }

    rsx! {
        article { class: "panel",
            div { class: "panel-heading",
                h3 { "All members" }
                p { "{data.users.len()} users" }
            }
            div { class: "topic-table",
                div { class: "topic-table-head",
                    span { "User" }
                    span { "Role" }
                    span { "Posts" }
                    span { "Actions" }
                }
                for user in data.users.iter() {
                    div { class: "topic-row",
                        div { class: "topic-main",
                            Link {
                                class: "topic-link",
                                to: Route::Profile { id: user.id },
                                "{user.username}"
                            }
                            p { class: "topic-meta",
                                "Joined {user.joined_at} · {user.email_display()}"
                            }
                        }
                        p { class: "topic-metric", "{group_label(user.group_id())}" }
                        p { class: "topic-metric", "{user.post_count}" }
                        div { class: "admin-actions",
                            // Promote to admin
                            if user.group_id() != 1 {
                                button {
                                    class: "small-button",
                                    onclick: {
                                        let uid = user.id;
                                        let uname = user.username.clone();
                                        move |_| {
                                            let uname = uname.clone();
                                            spawn(async move {
                                                match admin_update_user(AdminUserUpdate {
                                                        user_id: uid,
                                                        group_id: 1,
                                                        title: "Administrator".into(),
                                                    })
                                                    .await
                                                {
                                                    Ok(_) => {
                                                        is_error.set(false);
                                                        status.set(format!("{uname} promoted to admin. Refresh."));
                                                    }
                                                    Err(e) => {
                                                        is_error.set(true);
                                                        status.set(clean_error(e));
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    "Make admin"
                                }
                            }
                            // Demote to member
                            if user.group_id() == 1 {
                                button {
                                    class: "small-button",
                                    onclick: {
                                        let uid = user.id;
                                        let uname = user.username.clone();
                                        move |_| {
                                            let uname = uname.clone();
                                            spawn(async move {
                                                match admin_update_user(AdminUserUpdate {
                                                        user_id: uid,
                                                        group_id: 4,
                                                        title: "Member".into(),
                                                    })
                                                    .await
                                                {
                                                    Ok(_) => {
                                                        is_error.set(false);
                                                        status.set(format!("{uname} demoted to member. Refresh."));
                                                    }
                                                    Err(e) => {
                                                        is_error.set(true);
                                                        status.set(clean_error(e));
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    "Demote"
                                }
                            }
                            // Delete
                            button {
                                class: "danger-button small-button",
                                onclick: {
                                    let uid = user.id;
                                    move |_| {
                                        spawn(async move {
                                            match admin_delete_user(AdminDeleteItem { id: uid }).await {
                                                Ok(_) => {
                                                    is_error.set(false);
                                                    status.set("User deleted. Refresh.".into());
                                                }
                                                Err(e) => {
                                                    is_error.set(true);
                                                    status.set(clean_error(e));
                                                }
                                            }
                                        });
                                    }
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Moderation panel ──

#[component]
fn ModerationPanel(
    data: AdminData,
    mut status: Signal<String>,
    mut is_error: Signal<bool>,
    mut refresh: Signal<()>,
) -> Element {
    let users_map: std::collections::HashMap<i32, crate::data::UserProfile> =
        data.users.iter().map(|u| (u.id, u.clone())).collect();

    rsx! {
        article { class: "panel",
            div { class: "panel-heading",
                h3 { "All topics" }
                p { "{data.topics.len()} topics" }
            }
            div { class: "topic-table",
                div { class: "topic-table-head",
                    span { "Topic" }
                    span { "Sticky" }
                    span { "Closed" }
                    span { "Views" }
                    span { "Actions" }
                }
                for topic in data.topics.iter() {
                    div { class: "topic-row",
                        div { class: "topic-main",
                            Link {
                                class: "topic-link",
                                to: Route::TopicPage {
                                    id: topic.id,
                                    page: 1,
                                },
                                "{topic.subject}"
                            }
                            p { class: "topic-meta",
                                if let Some(author) = users_map.get(&topic.author_id) {
                                    "by {author.username} · {topic.created_at}"
                                }
                            }
                        }
                        p { class: "topic-metric", if topic.sticky { "Yes" } else { "No" } }
                        p { class: "topic-metric", if topic.closed { "Yes" } else { "No" } }
                        p { class: "topic-metric", "{topic.views}" }
                        div { class: "admin-actions",
                            button {
                                class: "small-button",
                                onclick: {
                                    let tid = topic.id;
                                    let closed = !topic.closed;
                                    move |_| {
                                        spawn(async move {
                                            match admin_update_topic(AdminTopicUpdate {
                                                topic_id: tid,
                                                closed,
                                            }).await
                                            {
                                                Ok(_) => {
                                                    is_error.set(false);
                                                    status.set("Topic status updated. Refresh.".into());
                                                }
                                                Err(e) => {
                                                    is_error.set(true);
                                                    status.set(clean_error(e));
                                                }
                                            }
                                        });
                                    }
                                },
                                if topic.closed { "Open" } else { "Close" }
                            }
                            button {
                                class: "danger-button small-button",
                                onclick: {
                                    let tid = topic.id;
                                    move |_| {
                                        spawn(async move {
                                            match admin_delete_topic(AdminDeleteItem { id: tid }).await {
                                                Ok(_) => {
                                                    is_error.set(false);
                                                    status.set("Topic deleted. Refresh.".into());
                                                }
                                                Err(e) => {
                                                    is_error.set(true);
                                                    status.set(clean_error(e));
                                                }
                                            }
                                        });
                                    }
                                },
                                "Delete"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── Settings panel ──

#[component]
fn SettingsPanel(
    data: AdminData,
    mut status: Signal<String>,
    mut is_error: Signal<bool>,
    mut refresh: Signal<()>,
) -> Element {
    let mut title = use_signal(|| data.meta.title.clone());
    let mut tagline = use_signal(|| data.meta.tagline.clone());
    let mut ann_title = use_signal(|| data.meta.announcement_title.clone());
    let mut ann_body = use_signal(|| data.meta.announcement_body.clone());

    rsx! {
        article { class: "form-card",
            h3 { "Board settings" }
            label {
                "Board title"
                input {
                    class: "text-input",
                    value: "{title}",
                    oninput: move |e| title.set(e.value()),
                }
            }
            label {
                "Tagline"
                input {
                    class: "text-input",
                    value: "{tagline}",
                    oninput: move |e| tagline.set(e.value()),
                }
            }
            label {
                "Announcement title"
                input {
                    class: "text-input",
                    value: "{ann_title}",
                    oninput: move |e| ann_title.set(e.value()),
                }
            }
            label {
                "Announcement body"
                textarea {
                    class: "text-area",
                    rows: "4",
                    value: "{ann_body}",
                    oninput: move |e| ann_body.set(e.value()),
                }
            }
            button {
                class: "primary-button",
                onclick: move |_| {
                    let form = AdminBoardSettings {
                        title: title(),
                        tagline: tagline(),
                        announcement_title: ann_title(),
                        announcement_body: ann_body(),
                    };
                    spawn(async move {
                        match admin_update_board(form).await {
                            Ok(_) => {
                                is_error.set(false);
                                status.set("Board settings saved. Refresh to see changes.".into());
                            }
                            Err(e) => {
                                is_error.set(true);
                                status.set(clean_error(e));
                            }
                        }
                    });
                },
                "Save settings"
            }
        }
    }
}

// ── Bans panel ──

#[component]
fn BansPanel(
    mut status: Signal<String>,
    mut is_error: Signal<bool>,
    mut refresh: Signal<()>,
) -> Element {
    let bans_resource = use_resource(move || async move {
        refresh();
        load_bans().await
    });

    let mut ban_user = use_signal(String::new);
    let mut ban_email = use_signal(String::new);
    let mut ban_message = use_signal(String::new);
    let mut ban_days = use_signal(|| 0_i32);

    rsx! {
        article { class: "panel",
            div { class: "panel-heading",
                h3 { "Active bans" }
            }
            if let Some(Ok(bans)) = bans_resource() {
                if bans.is_empty() {
                    p { "No active bans." }
                } else {
                    div { class: "topic-table",
                        div { class: "topic-table-head",
                            span { "User / Email" }
                            span { "Reason" }
                            span { "Expires" }
                            span { "Actions" }
                        }
                        for ban in bans {
                            div { class: "topic-row",
                                div { class: "topic-main",
                                    p { "{ban.username}" }
                                    if !ban.email.is_empty() {
                                        p { class: "topic-meta", "{ban.email}" }
                                    }
                                }
                                p { class: "topic-metric", "{ban.message}" }
                                p { class: "topic-metric",
                                    if let Some(exp) = ban.expires_at {
                                        "{(exp - ban.created_at) / 86400} days"
                                    } else {
                                        "Permanent"
                                    }
                                }
                                button {
                                    class: "danger-button small-button",
                                    onclick: {
                                        let bid = ban.id;
                                        move |_| {
                                            spawn(async move {
                                                match remove_ban(bid).await {
                                                    Ok(_) => refresh.set(()),
                                                    Err(e) => {
                                                        is_error.set(true);
                                                        status.set(clean_error(e));
                                                    }
                                                }
                                            });
                                        }
                                    },
                                    "Remove"
                                }
                            }
                        }
                    }
                }
            }
        }

        article { class: "form-card",
            h3 { "Add ban" }
            label {
                "Username (optional)"
                input {
                    class: "text-input",
                    value: "{ban_user}",
                    oninput: move |e| ban_user.set(e.value()),
                    placeholder: "Exact username",
                }
            }
            label {
                "Email (optional)"
                input {
                    class: "text-input",
                    value: "{ban_email}",
                    oninput: move |e| ban_email.set(e.value()),
                    placeholder: "Email address",
                }
            }
            label {
                "Reason / message"
                input {
                    class: "text-input",
                    value: "{ban_message}",
                    oninput: move |e| ban_message.set(e.value()),
                    placeholder: "Why is this user banned?",
                }
            }
            label {
                "Duration (days, 0 = permanent)"
                input {
                    class: "text-input",
                    r#type: "number",
                    value: "{ban_days}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<i32>() {
                            ban_days.set(v.max(0));
                        }
                    },
                }
            }
            button {
                class: "primary-button",
                onclick: move |_| {
                    let form = BanForm {
                        username: ban_user(),
                        email: ban_email(),
                        message: ban_message(),
                        duration_days: if ban_days() > 0 { Some(ban_days()) } else { None },
                    };
                    spawn(async move {
                        match add_ban(form).await {
                            Ok(_) => {
                                is_error.set(false);
                                status.set("Ban added.".to_string());
                                ban_user.set(String::new());
                                ban_email.set(String::new());
                                ban_message.set(String::new());
                                ban_days.set(0);
                                refresh.set(());
                            }
                            Err(e) => {
                                is_error.set(true);
                                status.set(clean_error(e));
                            }
                        }
                    });
                },
                "Add ban"
            }
        }
    }
}

// ── Groups panel ──

#[component]
fn GroupsPanel(
    mut status: Signal<String>,
    mut is_error: Signal<bool>,
    mut refresh: Signal<()>,
) -> Element {
    let groups_resource = use_resource(move || async move {
        refresh();
        load_groups().await
    });

    rsx! {
        if let Some(Ok(groups)) = groups_resource() {
            for group in groups {
                article { class: "form-card",
                    h3 { "{group.title}" }
                    div { class: "topic-table",
                        div { class: "topic-table-head",
                            span { "Permission" }
                            span { "Enabled" }
                        }
                        PermissionRow {
                            label: "Read board".to_string(),
                            value: group.read_board,
                            onchange: {
                                let gid = group.id;
                                let g = group.clone();
                                move |v: bool| {
                                    let g = g.clone();
                                    spawn(async move {
                                        let _ = update_group(GroupUpdateForm {
                                                group_id: gid,
                                                title: g.title.clone(),
                                                read_board: v,
                                                post_topics: g.post_topics,
                                                post_replies: g.post_replies,
                                                edit_posts: g.edit_posts,
                                                delete_posts: g.delete_posts,
                                                is_moderator: g.is_moderator,
                                                is_admin: g.is_admin,
                                            })
                                            .await;
                                        refresh.set(());
                                    });
                                }
                            },
                        }
                        PermissionRow {
                            label: "Post topics".to_string(),
                            value: group.post_topics,
                            onchange: {
                                let gid = group.id;
                                let g = group.clone();
                                move |v: bool| {
                                    let g = g.clone();
                                    spawn(async move {
                                        let _ = update_group(GroupUpdateForm {
                                                group_id: gid,
                                                title: g.title.clone(),
                                                read_board: g.read_board,
                                                post_topics: v,
                                                post_replies: g.post_replies,
                                                edit_posts: g.edit_posts,
                                                delete_posts: g.delete_posts,
                                                is_moderator: g.is_moderator,
                                                is_admin: g.is_admin,
                                            })
                                            .await;
                                        refresh.set(());
                                    });
                                }
                            },
                        }
                        PermissionRow {
                            label: "Post replies".to_string(),
                            value: group.post_replies,
                            onchange: {
                                let gid = group.id;
                                let g = group.clone();
                                move |v: bool| {
                                    let g = g.clone();
                                    spawn(async move {
                                        let _ = update_group(GroupUpdateForm {
                                                group_id: gid,
                                                title: g.title.clone(),
                                                read_board: g.read_board,
                                                post_topics: g.post_topics,
                                                post_replies: v,
                                                edit_posts: g.edit_posts,
                                                delete_posts: g.delete_posts,
                                                is_moderator: g.is_moderator,
                                                is_admin: g.is_admin,
                                            })
                                            .await;
                                        refresh.set(());
                                    });
                                }
                            },
                        }
                        PermissionRow {
                            label: "Edit posts".to_string(),
                            value: group.edit_posts,
                            onchange: {
                                let gid = group.id;
                                let g = group.clone();
                                move |v: bool| {
                                    let g = g.clone();
                                    spawn(async move {
                                        let _ = update_group(GroupUpdateForm {
                                                group_id: gid,
                                                title: g.title.clone(),
                                                read_board: g.read_board,
                                                post_topics: g.post_topics,
                                                post_replies: g.post_replies,
                                                edit_posts: v,
                                                delete_posts: g.delete_posts,
                                                is_moderator: g.is_moderator,
                                                is_admin: g.is_admin,
                                            })
                                            .await;
                                        refresh.set(());
                                    });
                                }
                            },
                        }
                        PermissionRow {
                            label: "Delete posts".to_string(),
                            value: group.delete_posts,
                            onchange: {
                                let gid = group.id;
                                let g = group.clone();
                                move |v: bool| {
                                    let g = g.clone();
                                    spawn(async move {
                                        let _ = update_group(GroupUpdateForm {
                                                group_id: gid,
                                                title: g.title.clone(),
                                                read_board: g.read_board,
                                                post_topics: g.post_topics,
                                                post_replies: g.post_replies,
                                                edit_posts: g.edit_posts,
                                                delete_posts: v,
                                                is_moderator: g.is_moderator,
                                                is_admin: g.is_admin,
                                            })
                                            .await;
                                        refresh.set(());
                                    });
                                }
                            },
                        }
                        PermissionRow {
                            label: "Moderator".to_string(),
                            value: group.is_moderator,
                            onchange: {
                                let gid = group.id;
                                let g = group.clone();
                                move |v: bool| {
                                    let g = g.clone();
                                    spawn(async move {
                                        let _ = update_group(GroupUpdateForm {
                                                group_id: gid,
                                                title: g.title.clone(),
                                                read_board: g.read_board,
                                                post_topics: g.post_topics,
                                                post_replies: g.post_replies,
                                                edit_posts: g.edit_posts,
                                                delete_posts: g.delete_posts,
                                                is_moderator: v,
                                                is_admin: g.is_admin,
                                            })
                                            .await;
                                        refresh.set(());
                                    });
                                }
                            },
                        }
                        PermissionRow {
                            label: "Admin".to_string(),
                            value: group.is_admin,
                            onchange: {
                                let gid = group.id;
                                let g = group.clone();
                                move |v: bool| {
                                    let g = g.clone();
                                    spawn(async move {
                                        let _ = update_group(GroupUpdateForm {
                                                group_id: gid,
                                                title: g.title.clone(),
                                                read_board: g.read_board,
                                                post_topics: g.post_topics,
                                                post_replies: g.post_replies,
                                                edit_posts: g.edit_posts,
                                                delete_posts: g.delete_posts,
                                                is_moderator: g.is_moderator,
                                                is_admin: v,
                                            })
                                            .await;
                                        refresh.set(());
                                    });
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PermissionRow(label: String, value: bool, onchange: EventHandler<bool>) -> Element {
    rsx! {
        div { class: "topic-row",
            span { "{label}" }
            input {
                r#type: "checkbox",
                checked: value,
                onchange: move |e| onchange.call(e.data.value() == "true" || e.data.checked()),
            }
        }
    }
}
