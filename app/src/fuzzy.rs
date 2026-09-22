//! Fuzzy search utilities for user notes.

#[derive(Debug, Clone)]
pub struct SearchItem {
    pub id: i64,
    pub title: String,
    #[allow(dead_code)]
    pub snippet: String,
    pub score: i64,
}

/// Computes a fuzzy match score between needle and haystack.
/// Returns Some(score) if needle is a subsequence of haystack, None otherwise.
pub fn fuzzy_match(needle: &str, haystack: &str) -> Option<i64> {
    if needle.is_empty() {
        return Some(0);
    }
    let needle_chars: Vec<char> = needle.to_lowercase().chars().collect();
    let haystack_chars: Vec<char> = haystack.to_lowercase().chars().collect();

    let mut n_idx = 0;
    let mut score = 0i64;
    let mut consecutive = 0i64;

    for (h_idx, &hc) in haystack_chars.iter().enumerate() {
        if hc == needle_chars[n_idx] {
            score += 10;
            if consecutive > 0 {
                score += consecutive * 5; // bonus for consecutive letters
            }
            if h_idx == 0 || haystack_chars[h_idx - 1].is_whitespace() || haystack_chars[h_idx - 1] == '_' || haystack_chars[h_idx - 1] == '-' {
                score += 15; // word boundary bonus
            }
            consecutive += 1;
            n_idx += 1;
            if n_idx == needle_chars.len() {
                if needle_chars.len() == haystack_chars.len() {
                    score += 50; // exact match bonus
                }
                score -= (haystack_chars.len() as i64).min(30);
                return Some(score);
            }
        } else {
            consecutive = 0;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match() {
        assert!(fuzzy_match("rust", "The Rust Programming Language").is_some());
        assert!(fuzzy_match("rpl", "Rust Programming Language").is_some());
        assert!(fuzzy_match("xyz", "Rust Language").is_none());

        let score_prefix = fuzzy_match("rust", "Rust").unwrap();
        let score_sub = fuzzy_match("rust", "A long text about Rust").unwrap();
        assert!(score_prefix > score_sub);
    }
}
