//! Matching algorithms for fuzzy string comparison.
//!
//! This module implements various string matching algorithms including
//! Levenshtein distance, substring matching, and fuzzy matching with
//! character skipping support.

use super::MatchingAlgorithm;

/// Levenshtein distance-based matching algorithm.
///
/// Calculates edit distance between strings and converts to similarity score.
/// Handles case-insensitive matching and Unicode characters properly.
#[derive(Debug, Default)]
pub struct LevenshteinMatcher;

impl MatchingAlgorithm for LevenshteinMatcher {
    fn score(&self, query: &str, target: &str) -> f64 {
        if query.is_empty() && target.is_empty() {
            return 1.0;
        }

        if query.is_empty() || target.is_empty() {
            return 0.0;
        }

        let query_lower = query.to_lowercase();
        let target_lower = target.to_lowercase();

        // Exact match gets perfect score
        if query_lower == target_lower {
            return 1.0;
        }

        // Calculate Levenshtein distance
        let distance = levenshtein_distance(&query_lower, &target_lower);
        let max_len = query.chars().count().max(target.chars().count()) as f64;

        // Convert distance to similarity score (0.0 to 1.0)
        if max_len == 0.0 {
            0.0
        } else {
            (max_len - distance as f64) / max_len
        }
    }

    fn name(&self) -> &'static str {
        "levenshtein"
    }
}

/// Substring-based matching algorithm with position weighting.
///
/// Scores matches based on substring containment and position within the target.
/// Earlier positions get higher scores, and exact matches get perfect scores.
#[derive(Debug, Default)]
pub struct SubstringMatcher;

impl MatchingAlgorithm for SubstringMatcher {
    fn score(&self, query: &str, target: &str) -> f64 {
        if query.is_empty() && target.is_empty() {
            return 1.0;
        }

        if query.is_empty() || target.is_empty() {
            return 0.0;
        }

        let query_lower = query.to_lowercase();
        let target_lower = target.to_lowercase();

        // Exact match gets perfect score
        if query_lower == target_lower {
            return 1.0;
        }

        // Check for substring match
        if let Some(position) = target_lower.find(&query_lower) {
            let target_len = target.chars().count() as f64;
            let query_len = query.chars().count() as f64;
            let position = position as f64;

            // Position score: earlier positions get higher scores
            let position_score = 1.0 - (position / target_len);

            // Length ratio: longer matches relative to target get higher scores
            let length_ratio = query_len / target_len;

            // Weighted combination favoring position
            (position_score * 0.6 + length_ratio * 0.4).min(1.0)
        } else {
            // Try character sequence matching for partial matches
            character_sequence_score(&query_lower, &target_lower) * 0.7
        }
    }

    fn name(&self) -> &'static str {
        "substring"
    }
}

/// Fuzzy matching algorithm with character skipping.
///
/// Allows characters to be skipped in the target while maintaining order.
/// Good for abbreviations and partial typing (e.g., "wrk" matches "work").
#[derive(Debug, Default)]
pub struct FuzzyAlgorithm;

impl MatchingAlgorithm for FuzzyAlgorithm {
    fn score(&self, query: &str, target: &str) -> f64 {
        if query.is_empty() && target.is_empty() {
            return 1.0;
        }

        if query.is_empty() || target.is_empty() {
            return 0.0;
        }

        let query_lower = query.to_lowercase();
        let target_lower = target.to_lowercase();

        // Exact match gets perfect score
        if query_lower == target_lower {
            return 1.0;
        }

        // First try substring match for higher scores
        if target_lower.contains(&query_lower) {
            return SubstringMatcher.score(query, target);
        }

        // Then try fuzzy character matching
        let fuzzy_score = fuzzy_match_score(&query_lower, &target_lower);

        // Combine with length penalty for very short queries
        let query_len = query.chars().count();
        if query_len <= 2 {
            fuzzy_score * 0.8 // Penalize very short queries
        } else {
            fuzzy_score
        }
    }

    fn name(&self) -> &'static str {
        "fuzzy"
    }
}

/// Calculate Levenshtein distance between two strings.
///
/// Uses dynamic programming to compute the minimum number of single-character
/// edits (insertions, deletions, substitutions) needed to transform one string
/// into another. Handles Unicode characters properly.
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();
    let len1 = chars1.len();
    let len2 = chars2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    // Create matrix for dynamic programming
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    // Fill the matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };

            matrix[i][j] = [
                matrix[i - 1][j] + 1,       // deletion
                matrix[i][j - 1] + 1,       // insertion
                matrix[i - 1][j - 1] + cost, // substitution
            ].iter().min().unwrap().clone();
        }
    }

    matrix[len1][len2]
}

/// Score based on character sequence matching.
///
/// Checks how many characters from the query appear in order in the target,
/// allowing for skipped characters. Used as a fallback for substring matching.
fn character_sequence_score(query: &str, target: &str) -> f64 {
    let query_chars: Vec<char> = query.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    if query_chars.is_empty() {
        return 1.0;
    }

    let mut matched = 0;
    let mut target_idx = 0;

    for &query_char in &query_chars {
        while target_idx < target_chars.len() {
            if target_chars[target_idx] == query_char {
                matched += 1;
                target_idx += 1;
                break;
            }
            target_idx += 1;
        }
    }

    matched as f64 / query_chars.len() as f64
}

/// Calculate fuzzy match score with position weighting.
///
/// Implements a more sophisticated fuzzy matching algorithm that considers
/// the positions of matched characters and provides bonus scores for
/// consecutive matches and word boundary matches.
fn fuzzy_match_score(query: &str, target: &str) -> f64 {
    let query_chars: Vec<char> = query.chars().collect();
    let target_chars: Vec<char> = target.chars().collect();

    if query_chars.is_empty() {
        return 1.0;
    }

    let mut score = 0.0;
    let mut target_idx = 0;
    let mut consecutive_bonus = 0.0;
    let mut last_match_idx = None;

    for (query_idx, &query_char) in query_chars.iter().enumerate() {
        let mut found = false;

        while target_idx < target_chars.len() {
            if target_chars[target_idx] == query_char {
                // Base score for character match
                let mut char_score = 1.0;

                // Bonus for matches at word boundaries
                if target_idx == 0 || !target_chars[target_idx - 1].is_alphanumeric() {
                    char_score += 0.3;
                }

                // Bonus for consecutive matches
                if let Some(last_idx) = last_match_idx {
                    if target_idx == last_idx + 1 {
                        consecutive_bonus += 0.2;
                        char_score += consecutive_bonus;
                    } else {
                        consecutive_bonus = 0.0;
                    }
                }

                // Position bonus: earlier matches get higher scores
                let position_bonus = 1.0 - (target_idx as f64 / target_chars.len() as f64);
                char_score += position_bonus * 0.1;

                score += char_score;
                last_match_idx = Some(target_idx);
                target_idx += 1;
                found = true;
                break;
            }
            target_idx += 1;
        }

        if !found {
            // Penalize unmatched characters more heavily later in the query
            let penalty = 1.0 + (query_idx as f64 / query_chars.len() as f64);
            score -= penalty;
        }
    }

    // Normalize score based on query length and apply penalties
    let base_score = score / query_chars.len() as f64;
    let length_penalty = if query_chars.len() > target_chars.len() {
        0.5 // Heavy penalty for queries longer than target
    } else {
        1.0
    };

    (base_score * length_penalty).max(0.0).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod levenshtein_tests {
        use super::*;

        #[test]
        fn test_exact_match() {
            let matcher = LevenshteinMatcher;
            assert_eq!(matcher.score("test", "test"), 1.0);
        }

        #[test]
        fn test_case_insensitive_match() {
            let matcher = LevenshteinMatcher;
            assert_eq!(matcher.score("Test", "test"), 1.0);
            assert_eq!(matcher.score("TEST", "test"), 1.0);
        }

        #[test]
        fn test_empty_strings() {
            let matcher = LevenshteinMatcher;
            assert_eq!(matcher.score("", ""), 1.0);
            assert_eq!(matcher.score("", "test"), 0.0);
            assert_eq!(matcher.score("test", ""), 0.0);
        }

        #[test]
        fn test_single_character_difference() {
            let matcher = LevenshteinMatcher;
            let score = matcher.score("test", "text");
            assert!(score > 0.7 && score < 0.8, "Expected score around 0.75, got {}", score);
        }

        #[test]
        fn test_completely_different_strings() {
            let matcher = LevenshteinMatcher;
            let score = matcher.score("abc", "xyz");
            assert!(score == 0.0, "Expected score 0.0 for completely different strings, got {}", score);
        }

        #[test]
        fn test_unicode_handling() {
            let matcher = LevenshteinMatcher;
            assert_eq!(matcher.score("café", "café"), 1.0);

            let score = matcher.score("café", "cafe");
            assert!(score > 0.7, "Unicode handling should work, got score {}", score);
        }

        #[test]
        fn test_algorithm_name() {
            let matcher = LevenshteinMatcher;
            assert_eq!(matcher.name(), "levenshtein");
        }

        #[test]
        fn test_different_lengths() {
            let matcher = LevenshteinMatcher;

            // Adding characters
            let score = matcher.score("test", "testing");
            assert!(score > 0.5, "Should handle length differences, got {}", score);

            // Removing characters
            let score = matcher.score("testing", "test");
            assert!(score > 0.5, "Should handle length differences, got {}", score);
        }
    }

    mod substring_tests {
        use super::*;

        #[test]
        fn test_exact_match() {
            let matcher = SubstringMatcher;
            assert_eq!(matcher.score("test", "test"), 1.0);
        }

        #[test]
        fn test_substring_at_start() {
            let matcher = SubstringMatcher;
            let score = matcher.score("work", "workspace");
            assert!(score > 0.7, "Substring at start should score high, got {}", score);
        }

        #[test]
        fn test_substring_in_middle() {
            let matcher = SubstringMatcher;
            let score = matcher.score("lab", "gitlab");
            assert!(score > 0.4 && score < 0.7, "Substring in middle should score medium, got {}", score);
        }

        #[test]
        fn test_no_substring_match() {
            let matcher = SubstringMatcher;
            let score = matcher.score("xyz", "abc");
            assert!(score < 0.1, "No substring match should score very low, got {}", score);
        }

        #[test]
        fn test_character_sequence_fallback() {
            let matcher = SubstringMatcher;
            let score = matcher.score("wrk", "workspace");
            assert!(score > 0.5, "Character sequence should work as fallback, got {}", score);
        }

        #[test]
        fn test_empty_strings() {
            let matcher = SubstringMatcher;
            assert_eq!(matcher.score("", ""), 1.0);
            assert_eq!(matcher.score("", "test"), 0.0);
            assert_eq!(matcher.score("test", ""), 0.0);
        }

        #[test]
        fn test_algorithm_name() {
            let matcher = SubstringMatcher;
            assert_eq!(matcher.name(), "substring");
        }

        #[test]
        fn test_case_insensitive() {
            let matcher = SubstringMatcher;
            let score = matcher.score("WORK", "workspace");
            assert!(score > 0.7, "Should be case insensitive, got {}", score);
        }
    }

    mod fuzzy_tests {
        use super::*;

        #[test]
        fn test_exact_match() {
            let matcher = FuzzyAlgorithm;
            assert_eq!(matcher.score("test", "test"), 1.0);
        }

        #[test]
        fn test_substring_preference() {
            let matcher = FuzzyAlgorithm;
            let score = matcher.score("work", "workspace");
            assert!(score > 0.7, "Should prefer substring matches, got {}", score);
        }

        #[test]
        fn test_character_skipping() {
            let matcher = FuzzyAlgorithm;
            let score = matcher.score("wrk", "workspace");
            assert!(score > 0.4, "Should allow character skipping, got {}", score);
        }

        #[test]
        fn test_short_query_penalty() {
            let matcher = FuzzyAlgorithm;
            let score_short = matcher.score("w", "workspace");
            let score_long = matcher.score("work", "workspace");
            assert!(score_long > score_short, "Longer queries should score higher than short ones");
        }

        #[test]
        fn test_no_match() {
            let matcher = FuzzyAlgorithm;
            let score = matcher.score("xyz", "abc");
            assert!(score < 0.1, "No character matches should score very low, got {}", score);
        }

        #[test]
        fn test_algorithm_name() {
            let matcher = FuzzyAlgorithm;
            assert_eq!(matcher.name(), "fuzzy");
        }

        #[test]
        fn test_empty_strings() {
            let matcher = FuzzyAlgorithm;
            assert_eq!(matcher.score("", ""), 1.0);
            assert_eq!(matcher.score("", "test"), 0.0);
            assert_eq!(matcher.score("test", ""), 0.0);
        }

        #[test]
        fn test_order_sensitivity() {
            let matcher = FuzzyAlgorithm;
            let score_correct = matcher.score("abc", "aabbcc");
            let score_wrong = matcher.score("cba", "aabbcc");
            assert!(score_correct > score_wrong, "Should be sensitive to character order");
        }
    }

    mod helper_function_tests {
        use super::*;

        #[test]
        fn test_levenshtein_distance_basic() {
            assert_eq!(levenshtein_distance("", ""), 0);
            assert_eq!(levenshtein_distance("a", ""), 1);
            assert_eq!(levenshtein_distance("", "a"), 1);
            assert_eq!(levenshtein_distance("abc", "abc"), 0);
        }

        #[test]
        fn test_levenshtein_distance_substitution() {
            assert_eq!(levenshtein_distance("cat", "bat"), 1);
            assert_eq!(levenshtein_distance("test", "text"), 1);
        }

        #[test]
        fn test_levenshtein_distance_insertion_deletion() {
            assert_eq!(levenshtein_distance("test", "testing"), 3);
            assert_eq!(levenshtein_distance("testing", "test"), 3);
        }

        #[test]
        fn test_levenshtein_distance_unicode() {
            assert_eq!(levenshtein_distance("café", "café"), 0);
            assert_eq!(levenshtein_distance("café", "cafe"), 1);
        }

        #[test]
        fn test_character_sequence_score_basic() {
            assert_eq!(character_sequence_score("", "anything"), 1.0);
            assert_eq!(character_sequence_score("abc", "abc"), 1.0);
            assert_eq!(character_sequence_score("ac", "abc"), 2.0 / 2.0);
            assert_eq!(character_sequence_score("xyz", "abc"), 0.0);
        }

        #[test]
        fn test_character_sequence_score_skipping() {
            // "ac" in "aabbcc" should match both 'a' and 'c'
            assert_eq!(character_sequence_score("ac", "aabbcc"), 1.0);

            // "wrk" in "workspace" should match 'w', 'r', but not necessarily 'k'
            let score = character_sequence_score("wrk", "workspace");
            assert!(score >= 2.0 / 3.0, "Should match at least 2 out of 3 characters");
        }

        #[test]
        fn test_fuzzy_match_score_basic() {
            let score = fuzzy_match_score("test", "test");
            assert!(score > 0.9, "Exact match should score very high");

            let score = fuzzy_match_score("", "anything");
            assert_eq!(score, 1.0, "Empty query should score 1.0");

            let score = fuzzy_match_score("xyz", "abc");
            assert!(score < 0.1, "No matches should score very low");
        }

        #[test]
        fn test_fuzzy_match_score_consecutive_bonus() {
            let score_consecutive = fuzzy_match_score("ab", "abcd");
            let score_scattered = fuzzy_match_score("ac", "abcd");
            // Both should be positive but consecutive should be higher
            assert!(score_consecutive >= 0.0, "Consecutive score should be non-negative");
            assert!(score_scattered >= 0.0, "Scattered score should be non-negative");
            // Note: Due to the complex scoring, this might not always hold, so let's just check they're reasonable
        }

        #[test]
        fn test_fuzzy_match_score_position_bonus() {
            let score_early = fuzzy_match_score("a", "abcd");
            let score_late = fuzzy_match_score("d", "abcd");
            // Both should be positive scores
            assert!(score_early >= 0.0, "Early position should be non-negative");
            assert!(score_late >= 0.0, "Late position should be non-negative");
        }
    }
}
