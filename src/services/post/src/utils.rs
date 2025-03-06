use chrono::Utc;
use regex::Regex;

pub fn generate_permalink(title: Option<&String>) -> String {
    let base_slug = match title {
        Some(t) if !t.trim().is_empty() => {
            // Process the title, sanitize HTML and emojis, then truncate to 30 characters
            let processed_title = &sanitize_html_and_emojis(t);
            process_title(&truncate_title(processed_title, 50))
        }
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

/// **Sanitize HTML by removing any HTML tags from the string**
fn sanitize_html_and_emojis(input: &str) -> String {
    // Use a regex to strip HTML tags
    let html_re = Regex::new(r"<[^>]*>").unwrap();
    let sanitized_html = html_re.replace_all(input, "");

    // Remove emojis by filtering out non-alphanumeric characters (including symbols like emojis)
    let emoji_re = Regex::new(r"[^\x00-\x7F]+").unwrap(); // Match non-ASCII characters (emojis)
    emoji_re.replace_all(&sanitized_html, "").to_string()
}

/// **Truncate the title to a maximum length**
fn truncate_title(title: &str, max_length: usize) -> String {
    if title.len() > max_length {
        title.chars().take(max_length).collect::<String>()
    } else {
        title.to_string()
    }
}
