#[cfg(feature = "server")]
use dioxus::prelude::ServerFnError;

mod admin;
mod auth;
mod bbcode;
mod db;
mod forum;
mod messages;
mod models;
mod polls;
mod rss;
mod security;

pub use admin::{
    add_ban, admin_add_category, admin_add_forum, admin_clean_sessions, admin_delete_category,
    admin_delete_forum, admin_delete_user, admin_update_board,
    admin_update_category, admin_update_forum, admin_update_user, dismiss_report, load_admin_data,
    load_bans, load_groups, remove_ban, report_post, test_smtp_settings, update_group, zap_report,
};
pub use auth::{
    check_installed, current_session_user, install_board, login_account, logout_account,
    register_account, request_password_reset, reset_password,
};
pub use forum::{
    change_password, create_reply, create_topic, delete_attachment, delete_post, delete_topic,
    edit_post, increment_topic_views, load_attachments, load_forum_data, load_forums,
    load_index_data, load_post, load_profile_data, load_shell_data, load_topic_data,
    load_users_data, mark_all_read, move_topic, search_server, toggle_sticky, toggle_topic_status,
    update_profile, upload_attachment,
};

pub use bbcode::render_paragraph;
pub use messages::{
    delete_conversation, load_conversation, load_inbox, reply_message, send_message,
};
pub use models::*;
pub use polls::{cast_vote, close_poll, create_poll, delete_poll, get_poll_with_user_vote};
pub use security::{clean_error, cookie_max_age, cookie_name, csrf_cookie_name};

#[cfg(feature = "server")]
pub async fn generate_rss_feed() -> Result<String, ServerFnError> {
    rss::generate_rss_feed().await
}
