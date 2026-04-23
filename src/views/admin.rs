use dioxus::prelude::*;

use crate::{
    components::SectionHeader,
    data::*,
    Route,
};

#[component]
pub fn Admin() -> Element {
    let current_user = use_context::<Signal<Option<SessionUser>>>();
    let board = use_context::<AppData>();

    let is_admin = current_user()
        .as_ref()
        .is_some_and(|u| u.group_id == 1);

    if !is_admin {
        return rsx! {
            section { class: "page",
                article { class: "empty-state",
                    h3 { "Access denied" }
                    p { "You must be an administrator to view this page." }
                }
            }
        };
    }

    let mut tab = use_signal(|| "structure");
    let status = use_signal(String::new);
    let is_error = use_signal(|| false);
    let mut refresh = use_context::<Signal<()>>();

    rsx! {
        section { class: "page",
            article { class: "hero-card compact-hero",
                SectionHeader {
                    kicker: "Admin".to_string(),
                    title: "Control panel".to_string(),
                    subtitle: "Board administration and moderation tools.".to_string(),
                }
            }

            nav { class: "admin-tabs",
                button { class: if tab() == "structure" { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set("structure"), "Structure" }
                button { class: if tab() == "users" { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set("users"), "Users" }
                button { class: if tab() == "moderation" { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set("moderation"), "Moderation" }
                button { class: if tab() == "settings" { "admin-tab active" } else { "admin-tab" },
                    onclick: move |_| tab.set("settings"), "Settings" }
            }

            if !status().is_empty() {
                p {
                    class: if is_error() { "form-message form-error" } else { "form-message form-success" },
                    "{status}"
                }
            }

            match tab().as_ref() {
                "structure" => rsx! { StructurePanel { board: board.clone(), status, is_error, refresh } },
                "users" => rsx! { UsersPanel { board: board.clone(), status, is_error, refresh } },
                "moderation" => rsx! { ModerationPanel { board: board.clone(), status, is_error, refresh } },
                "settings" => rsx! { SettingsPanel { board: board.clone(), status, is_error, refresh } },
                _ => rsx! {},
            }
        }
    }
}

// ── Structure panel: categories & forums ──

#[component]
fn StructurePanel(board: AppData, mut status: Signal<String>, mut is_error: Signal<bool>, mut refresh: Signal<()>) -> Element {
    let mut cat_name = use_signal(String::new);
    let mut cat_desc = use_signal(String::new);
    let mut forum_name = use_signal(String::new);
    let mut forum_desc = use_signal(String::new);
    let mut forum_cat_id = use_signal(|| 0_i32);

    rsx! {
        // ── Existing categories & forums ──
        for cat in board.categories_sorted() {
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
                                        Ok(_) => { is_error.set(false); status.set(format!("Category deleted. Refresh to see changes.")); }
                                        Err(e) => { is_error.set(true); status.set(e.to_string()); }
                                    }
                                });
                            }
                        },
                        "Delete category"
                    }
                }
                for forum in board.forums_in_category(cat.id) {
                    div { class: "forum-row",
                        div { class: "forum-main",
                            Link { class: "forum-link", to: Route::Forum { id: forum.id }, "{forum.name}" }
                            p { class: "forum-description", "{forum.description}" }
                        }
                        button {
                            class: "danger-button small-button",
                            onclick: {
                                let fid = forum.id;
                                move |_| {
                                    spawn(async move {
                                        match admin_delete_forum(AdminDeleteItem { id: fid }).await {
                                            Ok(_) => { is_error.set(false); status.set("Forum deleted. Refresh to see changes.".into()); }
                                            Err(e) => { is_error.set(true); status.set(e.to_string()); }
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
            label { "Name"
                input { class: "text-input", value: "{cat_name}", oninput: move |e| cat_name.set(e.value()), placeholder: "Category name" }
            }
            label { "Description"
                input { class: "text-input", value: "{cat_desc}", oninput: move |e| cat_desc.set(e.value()), placeholder: "Short description" }
            }
            button {
                class: "primary-button",
                onclick: move |_| {
                    let form = AdminCategoryForm { name: cat_name(), description: cat_desc() };
                    spawn(async move {
                        match admin_add_category(form).await {
                            Ok(_) => { is_error.set(false); status.set("Category created. Refresh to see it.".into()); cat_name.set(String::new()); cat_desc.set(String::new()); }
                            Err(e) => { is_error.set(true); status.set(e.to_string()); }
                        }
                    });
                },
                "Create category"
            }
        }

        // ── Add forum ──
        article { class: "form-card",
            h3 { "Add forum" }
            label { "Category"
                select {
                    class: "text-input",
                    onchange: move |e| { if let Ok(v) = e.value().parse::<i32>() { forum_cat_id.set(v); } },
                    option { value: "0", "Select category…" }
                    for cat in board.categories_sorted() {
                        option { value: "{cat.id}", "{cat.name}" }
                    }
                }
            }
            label { "Forum name"
                input { class: "text-input", value: "{forum_name}", oninput: move |e| forum_name.set(e.value()), placeholder: "Forum name" }
            }
            label { "Description"
                input { class: "text-input", value: "{forum_desc}", oninput: move |e| forum_desc.set(e.value()), placeholder: "Short description" }
            }
            button {
                class: "primary-button",
                onclick: move |_| {
                    let form = AdminForumForm { category_id: forum_cat_id(), name: forum_name(), description: forum_desc() };
                    spawn(async move {
                        match admin_add_forum(form).await {
                            Ok(_) => { is_error.set(false); status.set("Forum created. Refresh to see it.".into()); forum_name.set(String::new()); forum_desc.set(String::new()); }
                            Err(e) => { is_error.set(true); status.set(e.to_string()); }
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
fn UsersPanel(board: AppData, mut status: Signal<String>, mut is_error: Signal<bool>, mut refresh: Signal<()>) -> Element {
    fn group_label(gid: i32) -> &'static str {
        match gid { 1 => "Admin", 2 => "Moderator", 3 => "Guest", _ => "Member" }
    }

    rsx! {
        article { class: "panel",
            div { class: "panel-heading",
                h3 { "All members" }
                p { "{board.users.len()} users" }
            }
            div { class: "topic-table",
                div { class: "topic-table-head",
                    span { "User" }
                    span { "Role" }
                    span { "Posts" }
                    span { "Actions" }
                }
                for user in board.users.iter() {
                    div { class: "topic-row",
                        div { class: "topic-main",
                            Link { class: "topic-link", to: Route::Profile { id: user.id }, "{user.username}" }
                            p { class: "topic-meta", "Joined {user.joined_at} · {user.email_display()}" }
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
                                                match admin_update_user(AdminUserUpdate { user_id: uid, group_id: 1, title: "Administrator".into() }).await {
                                                    Ok(_) => { is_error.set(false); status.set(format!("{uname} promoted to admin. Refresh.")); }
                                                    Err(e) => { is_error.set(true); status.set(e.to_string()); }
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
                                                match admin_update_user(AdminUserUpdate { user_id: uid, group_id: 4, title: "Member".into() }).await {
                                                    Ok(_) => { is_error.set(false); status.set(format!("{uname} demoted to member. Refresh.")); }
                                                    Err(e) => { is_error.set(true); status.set(e.to_string()); }
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
                                                Ok(_) => { is_error.set(false); status.set("User deleted. Refresh.".into()); }
                                                Err(e) => { is_error.set(true); status.set(e.to_string()); }
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
fn ModerationPanel(board: AppData, mut status: Signal<String>, mut is_error: Signal<bool>, mut refresh: Signal<()>) -> Element {
    let statuses = ["pinned", "hot", "resolved", "fresh"];

    rsx! {
        article { class: "panel",
            div { class: "panel-heading",
                h3 { "All topics" }
                p { "{board.topics.len()} topics" }
            }
            div { class: "topic-table",
                div { class: "topic-table-head",
                    span { "Topic" }
                    span { "Status" }
                    span { "Views" }
                    span { "Actions" }
                }
                for topic in board.topics.iter() {
                    div { class: "topic-row",
                        div { class: "topic-main",
                            Link { class: "topic-link", to: Route::Topic { id: topic.id }, "{topic.subject}" }
                            p { class: "topic-meta",
                                if let Some(author) = board.user(topic.author_id) {
                                    "by {author.username} · {topic.created_at}"
                                }
                            }
                        }
                        p { class: "topic-metric", "{topic.status.label()}" }
                        p { class: "topic-metric", "{topic.views}" }
                        div { class: "admin-actions",
                            for st in statuses {
                                button {
                                    class: if topic.status.label().to_lowercase() == st { "small-button active" } else { "small-button" },
                                    onclick: {
                                        let tid = topic.id;
                                        let st = st.to_string();
                                        move |_| {
                                            let st = st.clone();
                                            spawn(async move {
                                                match admin_update_topic(AdminTopicUpdate { topic_id: tid, status: st }).await {
                                                    Ok(_) => { is_error.set(false); status.set("Topic status updated. Refresh.".into()); }
                                                    Err(e) => { is_error.set(true); status.set(e.to_string()); }
                                                }
                                            });
                                        }
                                    },
                                    "{st}"
                                }
                            }
                            button {
                                class: "danger-button small-button",
                                onclick: {
                                    let tid = topic.id;
                                    move |_| {
                                        spawn(async move {
                                            match admin_delete_topic(AdminDeleteItem { id: tid }).await {
                                                Ok(_) => { is_error.set(false); status.set("Topic deleted. Refresh.".into()); }
                                                Err(e) => { is_error.set(true); status.set(e.to_string()); }
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
fn SettingsPanel(board: AppData, mut status: Signal<String>, mut is_error: Signal<bool>, mut refresh: Signal<()>) -> Element {
    let mut title = use_signal(|| board.meta.title.clone());
    let mut tagline = use_signal(|| board.meta.tagline.clone());
    let mut ann_title = use_signal(|| board.meta.announcement_title.clone());
    let mut ann_body = use_signal(|| board.meta.announcement_body.clone());

    rsx! {
        article { class: "form-card",
            h3 { "Board settings" }
            label { "Board title"
                input { class: "text-input", value: "{title}", oninput: move |e| title.set(e.value()) }
            }
            label { "Tagline"
                input { class: "text-input", value: "{tagline}", oninput: move |e| tagline.set(e.value()) }
            }
            label { "Announcement title"
                input { class: "text-input", value: "{ann_title}", oninput: move |e| ann_title.set(e.value()) }
            }
            label { "Announcement body"
                textarea { class: "text-area", rows: "4", value: "{ann_body}", oninput: move |e| ann_body.set(e.value()) }
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
                            Ok(_) => { is_error.set(false); status.set("Board settings saved. Refresh to see changes.".into()); }
                            Err(e) => { is_error.set(true); status.set(e.to_string()); }
                        }
                    });
                },
                "Save settings"
            }
        }
    }
}
