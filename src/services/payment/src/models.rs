use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub amount: f64, // Payment amount
    pub currency: String, // Currency type (e.g., "USD", "KHR")
    pub bank_code: String, // Bank identifier code
    pub account_number: String, // Customer's bank account number
    pub description: String,  // Payment description
    pub transaction_id: String, // Unique transaction ID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub status: String, // Status of the payment (e.g., "Success", "Failed")
    pub transaction_id: String, // Transaction ID for tracking
    pub message: String, // Additional response message
}

impl PaymentRequest {
    pub fn new(amount: f64, currency: &str, bank_code: &str, account_number: &str, description: &str) -> Self {
        Self {
            amount,
            currency: currency.to_string(),
            bank_code: bank_code.to_string(),
            account_number: account_number.to_string(),
            description: description.to_string(),
            transaction_id: uuid::Uuid::new_v4().to_string(), // Generate a unique transaction ID
        }
    }
}
