use mongodb::bson::{oid::ObjectId, DateTime};
use serde::{Deserialize, Serialize};

use crate::utils::generate_permalink;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Product {
    pub _id: String,
    pub owner_id: String,
    pub name: String,
    pub description: String,
    pub permalink: String,
    pub price: f64,
    pub currency: String,
    pub thumb_url: String,
    pub images: Vec<String>,
    pub category: String,
    pub subcategories: Vec<String>,
    pub tags: Vec<String>,
    pub detail: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RequestProduct {
    pub name: String,
    pub owner_id: String,
    pub description: String,
    pub price: f64,
    pub currency: String,
    pub thumb_url: String,
    pub images: Vec<String>,
    pub category: String,
    pub subcategories: Vec<String>,
    pub tags: Vec<String>,
    pub detail: Option<String>,
}

impl Product {
    pub fn new(req: RequestProduct, owner_id: String) -> Self {
        let permalink = generate_permalink(Some(&req.name.clone()));
        Self {
            _id: ObjectId::new().to_hex(),
            name: req.name.clone(),
            owner_id,
            description: req.description,
            permalink: permalink,
            price: req.price,
            currency: req.currency,
            thumb_url: req.thumb_url,
            images: req.images,
            category: req.category,
            subcategories: req.subcategories,
            tags: req.tags,
            detail: req.detail,
            created_at: Some(DateTime::now().try_to_rfc3339_string().unwrap()),
            updated_at: Some(DateTime::now().try_to_rfc3339_string().unwrap()),
            deleted_at: None,
        }
    }
}
