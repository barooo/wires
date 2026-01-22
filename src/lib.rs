pub mod db;
pub mod models;

use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a 7-character ID from title and timestamp
pub fn generate_id(title: &str) -> String {
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_nanos();

    let input = format!("{}{}", title, timestamp_nanos);
    let hash = Sha256::digest(input.as_bytes());
    let hex = format!("{:x}", hash);

    hex[..7].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_id_length() {
        let id = generate_id("Test wire");
        assert_eq!(id.len(), 7);
    }

    #[test]
    fn test_generate_id_uniqueness() {
        let id1 = generate_id("Test wire");
        let id2 = generate_id("Test wire");
        // Same title but different timestamps should produce different IDs
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_hex_characters() {
        let id = generate_id("Test wire");
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
