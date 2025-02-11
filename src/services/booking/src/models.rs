use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Accommodation {
    #[serde(rename = "_id")]
    pub id: String,                     // Unique identifier for the accommodation
    pub owner_id: Option<String>,
    pub name: String,                   // Name of the accommodation
    pub description: Option<String>,    // Detailed description of the accommodation
    pub location: Location,             // Nested location details
    pub price_per_night: f64,           // Price per night in the specified currency
    pub currency_code: String,          // Currency code, e.g., "USD", "EUR"
    pub is_available: bool,             // Whether the accommodation is currently available
    pub max_guests: i32,                // Maximum number of guests allowed
    pub facilities: Vec<Facility>,      // List of facilities provided
    pub nearby_attractions: Vec<NearbyAttraction>, // List of nearby attractions
    pub images: Vec<Image>,             // List of image metadata
    pub reviews: Vec<Review>,           // Customer reviews
    pub ratings_average: Option<f64>,   // Average rating of the accommodation
    pub tags: Vec<String>,              // Categories or tags for filtering, e.g., "luxury", "beachfront"
    pub seasonal_pricing: Option<Vec<SeasonalPricing>>, // Optional seasonal pricing details
    pub promotions: Option<Vec<Promotion>>, // Optional promotional offers
    pub created_at: String,             // Creation timestamp (ISO 8601 format)
    pub updated_at: String,             // Last update timestamp (ISO 8601 format)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
    pub address: String,            // Full address
    pub latitude: f64,              // Latitude of the location
    pub longitude: f64,             // Longitude of the location
    pub city: String,               // City name
    pub country: String,            // Country name
    pub postal_code: Option<String>, // Postal code (optional)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Facility {
    pub name: String,               // Facility name, e.g., "Wi-Fi", "Swimming Pool"
    pub available: bool,            // Whether the facility is available
    pub description: Option<String>, // Optional description of the facility
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NearbyAttraction {
    pub name: String,               // Name of the attraction
    pub distance_km: f64,           // Distance in kilometers
    pub description: Option<String>, // Description of the attraction
    pub category: String,           // Category, e.g., "historical", "natural"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub url: String,                // URL of the image
    pub alt_text: Option<String>,   // Alternative text for accessibility (optional)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Review {
    pub user_id: String,            // ID of the user who wrote the review
    pub rating: f64,                // Rating out of 5
    pub comment: Option<String>,    // Optional comment about the accommodation
    pub created_at: String,         // Timestamp of the review (ISO 8601 format)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SeasonalPricing {
    pub season: String,             // Season name, e.g., "peak", "off-peak"
    pub price_per_night: f64,       // Price per night during this season
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Promotion {
    pub title: String,              // Title of the promotion
    pub description: Option<String>, // Description of the promotion
    pub discount_percentage: f64,   // Discount percentage
    pub valid_from: String,         // Start date of the promotion (ISO 8601 format)
    pub valid_to: String,           // End date of the promotion (ISO 8601 format)
}
