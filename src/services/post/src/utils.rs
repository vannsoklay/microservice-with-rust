use chrono::Utc;

pub fn generate_permlink(title: Option<&String>) -> String {
    let base_slug = match title {
        Some(t) if !t.trim().is_empty() => process_title(t), // Process the title
        _ => String::from("untitled"), // Fallback if no title
    };

    // Get the current timestamp to append to the slug
    let timestamp = Utc::now().timestamp();

    // Combine the Khmer and English parts of the title with a timestamp for uniqueness
    format!("{}-{}", base_slug, timestamp)
}

/// **Process the title by keeping Khmer and adding hyphens for spaces**
fn process_title(text: &str) -> String {
    let mut result = String::new();

    // Split the text into words based on spaces
    for word in text.split_whitespace() {
        if is_khmer(word) {
            // Leave Khmer words as they are
            result.push_str(word);
        } else {
            // Transliterate English words (or other non-Khmer characters)
            result.push_str(&transliterate_english(word));
        }

        result.push('-'); // Add a hyphen to separate words
    }

    // Remove trailing hyphen if any
    if result.ends_with('-') {
        result.pop();
    }

    result.to_lowercase()
}

/// **Check if a word contains Khmer characters**
fn is_khmer(word: &str) -> bool {
    word.chars().any(|ch| ch >= '\u{1780}' && ch <= '\u{19FF}') // Khmer Unicode range
}

/// **Transliterate English words into URL-friendly format**
fn transliterate_english(word: &str) -> String {
    word.to_lowercase().replace(" ", "-") // Simple lowercase and hyphen for spaces
}
