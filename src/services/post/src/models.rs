use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::utils::generate_permalink;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PostType {
    SocialMedia,
    Blog,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Post {
    #[serde(rename = "_id")]
    pub id: Option<String>,
    pub title: Option<String>,
    pub permalink: Option<String>,
    pub content: String,
    pub author_id: Option<String>,
    pub media_urls: Vec<String>,
    pub tags: Option<Vec<String>>,
    pub post_type: PostType,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

impl Post {
    pub fn new(
        content: String,
        author_id: Option<String>,
        post_type: PostType,
        title: Option<String>,
        media_urls: Vec<String>,
        tags: Option<Vec<String>>,
    ) -> Self {
        let permalink = generate_permalink(title.as_ref());
        Self {
            id: Some(ObjectId::new().to_hex()),
            title,
            content,
            permalink: Some(permalink),
            author_id,
            media_urls,
            tags,
            post_type,
            created_at: Some(DateTime::now().try_to_rfc3339_string().unwrap()),
            updated_at: None,
            deleted_at: None,
        }
    }

    // pub async fn find_with_author(
    //     post_id: &str,
    //     posts_collection: &Collection<Post>,
    //     users_collection: &Collection<User>,
    // ) -> Result<Option<PostWithAuthor>> {
    //     // Convert the string post_id to ObjectId if needed
    //     let filter = doc! { "_id": post_id };

    //     // Step 1: Find the post
    //     if let Some(post) = posts_collection.find_one(filter, None).await? {
    //         // Step 2: Find the author if author_id is present
    //         let author = if let Some(author_id) = &post.author_id {
    //             users_collection
    //                 .find_one(doc! { "_id": author_id }, None)
    //                 .await?
    //         } else {
    //             None
    //         };

    //         // Step 3: Return the combined result
    //         Ok(Some(PostWithAuthor { post, author }))
    //     } else {
    //         Ok(None)
    //     }
    // }

    // pub async fn find_all(
    //     posts_collection: &Collection<Post>,
    //     users_collection: &Collection<User>,
    // ) -> Result<Vec<PostWithAuthor>> {
    //     let mut posts_with_authors = Vec::new();

    //     // Step 1: Find all posts
    //     let cursor = posts_collection.find({}).await?;

    //     // Step 2: Iterate through each post
    //     for result in cursor {
    //         if let Ok(post) = result {
    //             // Step 3: Find the author if author_id is present
    //             let author = if let Some(author_id) = &post.author_id {
    //                 users_collection
    //                     .find_one(doc! { "_id": author_id }, None)
    //                     .await?
    //             } else {
    //                 None
    //             };

    //             // Step 4: Push the combined result to the vector
    //             posts_with_authors.push(PostWithAuthor { post, author });
    //         }
    //     }

    //     Ok(posts_with_authors)
    // }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PostResponse {
    pub id: String,
    pub title: Option<String>,
    pub permalink: Option<String>,
    pub content: String,
    pub post_type: String,
    pub media_urls: Vec<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<AuthorInfo>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthorInfo {
    #[serde(rename(serialize = "id"))]
    #[serde(rename(deserialize = "_id"))]
    pub id: String,
    pub username: String,
    pub avatar: Option<String>,
    pub is_verified: bool,
    pub follower_count: i32,
    pub following_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostWithAuthor {
    pub post: Post,
    pub author: Option<AuthorInfo>,
}

impl PostWithAuthor {
    pub fn to_response(self) -> PostResponse {
        PostResponse {
            id: self.post.id.clone().unwrap_or_default(),
            title: self.post.title.clone(),
            permalink: self.post.permalink.clone(),
            content: self.post.content.clone(),
            post_type: format!("{:?}", self.post.post_type),
            media_urls: self.post.media_urls.clone(),
            tags: self.post.tags.clone(),
            created_at: self.post.created_at.clone(),
            updated_at: self.post.updated_at.clone(),
            deleted_at: self.post.deleted_at.clone(),
            author: self.author.map(|a| AuthorInfo {
                id: a.id,
                username: a.username,
                avatar: a.avatar,
                is_verified: a.is_verified,
                follower_count: a.follower_count,
                following_count: a.following_count,
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Vote {
    pub permalink: String,
}