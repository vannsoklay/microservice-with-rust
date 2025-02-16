use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum PropertyType {
    Apartment,
    House,
    Hotel,
    Hostel,
    Villa,
    Resort,
    Guesthouse,
    Other,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PropertyStatus {
    Active,
    Inactive,
    PendingApproval,
    Banned,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub street: String,
    pub city: String,
    pub state: Option<String>,
    pub country: String,
    pub zip_code: Option<String>,
    pub latitude: Option<f64>, // GPS coordinates for mapping
    pub longitude: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Review {
    pub user_id: String,            // Reference to the user who left the review
    pub rating: f32,                // Rating out of 5
    pub comment: Option<String>,    // Optional review text
    pub created_at: Option<String>, // Timestamp
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Availability {
    pub start_date: String, // Available start date (YYYY-MM-DD)
    pub end_date: String,   // Available end date (YYYY-MM-DD)
    pub is_available: bool, // If the property is available during this period
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Property {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>, // MongoDB auto-generates this field

    pub owner_id: Option<String>, // User ID of the property owner

    pub name: String,                // Name of the property
    pub description: String,         // Description of the property
    pub property_type: PropertyType, // Type of property

    pub address: Address, // Address struct

    pub price_per_night: Decimal, // High-precision price handling
    pub max_guests: u32,          // Maximum guests allowed
    pub bedrooms: u32,            // Number of bedrooms
    pub bathrooms: u32,           // Number of bathrooms

    pub amenities: Vec<String>, // List of amenities (WiFi, Pool, AC, etc.)
    pub images: Vec<String>,    // URLs of property images

    pub rules: Vec<String>, // List of property rules (e.g., No Smoking, No Pets)

    pub availability: Vec<Availability>, // Available dates

    pub reviews: Vec<Review>, // User reviews

    pub status: PropertyStatus, // Status of the property (Active, Inactive, etc.)

    pub created_at: Option<String>, // ISO timestamp
    pub updated_at: Option<String>, // Last update timestamp
}
