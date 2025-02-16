use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Booking {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,  // MongoDB assigns ID automatically

    pub user_id: Option<String>,      // Reference to the user who made the booking
    pub property_id: String,  // Reference to the accommodation being booked

    pub check_in: String,   // Check-in date
    pub check_out: String,  // Check-out date
    pub guests: u32,               // Number of guests
    pub total_price: f64,          // Total price for the booking

    #[serde(default = "default_status")]
    pub status: BookingStatus, // Default: Pending

    #[serde(default = "default_payment_status")]
    pub payment_status: PaymentStatus, // Default: Pending

    pub created_at: Option<String>,  // Default: Now
    pub updated_at: Option<String>,  // Default: Now

    pub special_requests: Option<String>,    // Optional: Special requests from the user
    pub cancellation_policy: Option<String>, // Optional: Policy info
    pub payment_method: PaymentMethod,       // Payment method used
    pub transaction_id: Option<String>,      // Optional: Payment transaction ID
}

// Default functions for serde
fn default_status() -> BookingStatus {
    BookingStatus::Pending
}

fn default_payment_status() -> PaymentStatus {
    PaymentStatus::Pending
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BookingStatus {
    Pending,
    Confirmed,
    Canceled,
    Completed,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Paid,
    Failed,
    Refunded,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    CreditCard,
    PayPal,
    BankTransfer,
    Cash,
}
