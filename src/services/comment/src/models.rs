#[derive(Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub post_id: String,
    pub author_id: String,
    pub content: String,
    pub parent_comment_id: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
