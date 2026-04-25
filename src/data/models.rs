use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardMeta {
    pub title: String,
    pub tagline: String,
    pub announcement_title: String,
    pub announcement_body: String,
    pub smtp_host: String,
    pub smtp_port: i32,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_enable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Forum {
    pub id: i32,
    pub category_id: i32,
    pub name: String,
    pub description: String,
    pub moderators: Vec<String>,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: i32,
    pub username: String,
    pub title: String,
    pub status: String,
    pub joined_at: String,
    pub post_count: i32,
    pub location: String,
    pub about: String,
    pub last_seen: String,
    #[serde(default)]
    pub email: String,
    #[serde(default = "default_group_id")]
    pub group_id: i32,
    #[serde(default)]
    pub timezone: String,
    #[serde(default = "default_disp_topics")]
    pub disp_topics: i32,
    #[serde(default = "default_disp_posts")]
    pub disp_posts: i32,
    #[serde(default = "default_show_online")]
    pub show_online: bool,
}

fn default_group_id() -> i32 {
    4
}

fn default_disp_topics() -> i32 {
    25
}

fn default_disp_posts() -> i32 {
    20
}

fn default_show_online() -> bool {
    true
}

impl UserProfile {
    pub fn group_id(&self) -> i32 {
        self.group_id
    }

    pub fn email_display(&self) -> &str {
        if self.email.is_empty() {
            "no email"
        } else {
            &self.email
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Topic {
    pub id: i32,
    pub forum_id: i32,
    pub author_id: i32,
    pub subject: String,
    #[serde(default)]
    pub closed: bool,
    pub views: i32,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub activity_rank: i32,
    #[serde(default)]
    pub reply_count: i32,
    #[serde(default)]
    pub sticky: bool,
    #[serde(default)]
    pub moved_to: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Post {
    pub id: i32,
    pub topic_id: i32,
    pub author_id: i32,
    pub posted_at: String,
    pub edited_at: Option<String>,
    pub body: Vec<String>,
    pub signature: Option<String>,
    pub position: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardStats {
    pub members: usize,
    pub topics: usize,
    pub posts: usize,
    pub newest_member: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OnlineUser {
    pub id: i32,
    pub username: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShellData {
    pub meta: BoardMeta,
    pub stats: BoardStats,
    pub online_users: Vec<OnlineUser>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForumStats {
    pub forum_id: i32,
    pub topic_count: usize,
    pub post_count: usize,
    pub last_topic_id: i32,
    pub last_topic_subject: String,
    pub last_post_author: String,
    pub last_posted_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IndexData {
    pub meta: BoardMeta,
    pub categories: Vec<Category>,
    pub forums: Vec<Forum>,
    pub forum_stats: Vec<ForumStats>,
    pub stats: BoardStats,
    pub recent_topics: Vec<Topic>,
    pub recent_users: Vec<UserProfile>,
    #[serde(default)]
    pub last_visit: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForumData {
    pub forum: Forum,
    pub topics: Vec<Topic>,
    pub users: Vec<UserProfile>,
    pub total_topics: i32,
    pub page: i32,
    pub per_page: i32,
    #[serde(default)]
    pub last_visit: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TopicData {
    pub topic: Topic,
    pub posts: Vec<Post>,
    pub users: Vec<UserProfile>,
    pub forum: Option<Forum>,
    pub total_posts: i32,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProfileData {
    pub user: UserProfile,
    pub topics: Vec<Topic>,
    pub posts: Vec<Post>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminData {
    pub meta: BoardMeta,
    pub categories: Vec<Category>,
    pub forums: Vec<Forum>,
    pub users: Vec<UserProfile>,
    pub topics: Vec<Topic>,
    #[serde(default)]
    pub reports: Vec<Report>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct SearchResults {
    pub topics: Vec<Topic>,
    pub users: Vec<UserProfile>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: i32,
    pub username: String,
    #[serde(default)]
    pub email: String,
    pub title: String,
    pub group_id: i32,
    #[serde(default)]
    pub csrf_token: String,
    #[serde(default)]
    pub timezone: String,
    #[serde(default = "default_disp_topics")]
    pub disp_topics: i32,
    #[serde(default = "default_disp_posts")]
    pub disp_posts: i32,
    #[serde(default = "default_show_online")]
    pub show_online: bool,
    #[serde(default)]
    pub post_topics: bool,
    #[serde(default)]
    pub post_replies: bool,
    #[serde(default)]
    pub edit_posts: bool,
    #[serde(default)]
    pub delete_posts: bool,
    #[serde(default)]
    pub delete_topic: bool,
    #[serde(default)]
    pub move_topic: bool,
    #[serde(default)]
    pub sticky_topic: bool,
    #[serde(default)]
    pub close_topic: bool,
    #[serde(default)]
    pub manage_users: bool,
    #[serde(default)]
    pub manage_forums: bool,
    #[serde(default)]
    pub manage_categories: bool,
    #[serde(default)]
    pub manage_bans: bool,
    #[serde(default)]
    pub manage_groups: bool,
    #[serde(default)]
    pub manage_settings: bool,
    #[serde(default)]
    pub is_moderator: bool,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthResponse {
    pub user: SessionUser,
    pub session_token: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegisterForm {
    pub username: String,
    pub email: String,
    pub password: String,
    pub location: String,
    pub about: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
    pub remember: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InstallForm {
    pub board_title: String,
    pub board_tagline: String,
    pub admin_username: String,
    pub admin_email: String,
    pub admin_password: String,
    pub db_host: String,
    pub db_port: String,
    pub db_name: String,
    pub db_user: String,
    pub db_password: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewTopicForm {
    pub forum_id: i32,
    pub subject: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReplyForm {
    pub topic_id: i32,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NewTopicResult {
    pub topic_id: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EditPostForm {
    pub post_id: i32,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateProfileForm {
    pub user_id: i32,
    pub email: String,
    pub location: String,
    pub about: String,
    pub timezone: String,
    pub disp_topics: i32,
    pub disp_posts: i32,
    pub show_online: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangePasswordForm {
    pub user_id: i32,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RequestPasswordResetForm {
    pub email: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResetPasswordForm {
    pub token: String,
    pub password: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminCategoryForm {
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminForumForm {
    pub category_id: i32,
    pub name: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminUserUpdate {
    pub user_id: i32,
    pub group_id: i32,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminBoardSettings {
    pub title: String,
    pub tagline: String,
    pub announcement_title: String,
    pub announcement_body: String,
    pub smtp_host: String,
    pub smtp_port: i32,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from_email: String,
    pub smtp_from_name: String,
    pub smtp_enable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminDeleteItem {
    pub id: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminCategoryUpdate {
    pub id: i32,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AdminForumUpdate {
    pub id: i32,
    pub category_id: i32,
    pub name: String,
    pub description: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportPostForm {
    pub post_id: i32,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub id: i32,
    pub post_id: i32,
    pub reporter_id: i32,
    pub reporter_name: String,
    pub reason: String,
    pub created_at: i64,
    pub zapped: bool,
    pub post_body: Vec<String>,
    pub topic_id: i32,
    pub topic_subject: String,
    pub author_id: i32,
    pub author_name: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GroupUpdateForm {
    pub group_id: i32,
    pub title: String,
    pub read_board: bool,
    pub post_topics: bool,
    pub post_replies: bool,
    pub edit_posts: bool,
    pub delete_posts: bool,
    pub delete_topic: bool,
    pub move_topic: bool,
    pub sticky_topic: bool,
    pub close_topic: bool,
    pub manage_users: bool,
    pub manage_forums: bool,
    pub manage_categories: bool,
    pub manage_bans: bool,
    pub manage_groups: bool,
    pub manage_settings: bool,
    pub is_moderator: bool,
    pub is_admin: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TestSmtpForm {
    pub test_email: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MoveTopicForm {
    pub topic_id: i32,
    pub forum_id: i32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ban {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub ip: String,
    pub message: String,
    pub created_at: i64,
    pub expires_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub id: i32,
    pub title: String,
    pub read_board: bool,
    pub post_topics: bool,
    pub post_replies: bool,
    pub edit_posts: bool,
    pub delete_posts: bool,
    pub delete_topic: bool,
    pub move_topic: bool,
    pub sticky_topic: bool,
    pub close_topic: bool,
    pub manage_users: bool,
    pub manage_forums: bool,
    pub manage_categories: bool,
    pub manage_bans: bool,
    pub manage_groups: bool,
    pub manage_settings: bool,
    pub is_moderator: bool,
    pub is_admin: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BanForm {
    pub username: String,
    pub email: String,
    pub message: String,
    pub duration_days: Option<i32>,
}
