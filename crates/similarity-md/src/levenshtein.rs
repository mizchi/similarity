/// Calculate Levenshtein distance between two strings
/// This is optimized for natural language text comparison
pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.chars().count();
    let len2 = s2.chars().count();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let chars1: Vec<char> = s1.chars().collect();
    let chars2: Vec<char> = s2.chars().collect();

    // Create a matrix to store distances
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    // Initialize first row and column
    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *cell = j;
    }

    // Fill the matrix
    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(
                    matrix[i - 1][j] + 1, // deletion
                    matrix[i][j - 1] + 1, // insertion
                ),
                matrix[i - 1][j - 1] + cost, // substitution
            );
        }
    }

    matrix[len1][len2]
}

/// Calculate normalized Levenshtein similarity (0.0 to 1.0)
pub fn levenshtein_similarity(s1: &str, s2: &str) -> f64 {
    let distance = levenshtein_distance(s1, s2);
    let max_len = s1.chars().count().max(s2.chars().count());

    if max_len == 0 {
        return 1.0;
    }

    1.0 - (distance as f64 / max_len as f64)
}

/// Calculate word-level Levenshtein distance
/// This treats words as units instead of characters
/// For Japanese text, uses character-based comparison instead of whitespace splitting
pub fn word_levenshtein_distance(s1: &str, s2: &str) -> usize {
    // Check if text contains Japanese characters
    let has_japanese = |text: &str| {
        text.chars().any(|c| {
            // ひらがな (U+3040-U+309F)
            ('\u{3040}'..='\u{309F}').contains(&c) ||
            // カタカナ (U+30A0-U+30FF)
            ('\u{30A0}'..='\u{30FF}').contains(&c) ||
            // 漢字 (U+4E00-U+9FAF)
            ('\u{4E00}'..='\u{9FAF}').contains(&c)
        })
    };

    // For Japanese text, use character-based comparison
    if has_japanese(s1) || has_japanese(s2) {
        return levenshtein_distance(s1, s2);
    }

    // For non-Japanese text, use traditional word-based comparison
    let words1: Vec<&str> = s1.split_whitespace().collect();
    let words2: Vec<&str> = s2.split_whitespace().collect();

    let len1 = words1.len();
    let len2 = words2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(len1 + 1) {
        row[0] = i;
    }
    for (j, cell) in matrix[0].iter_mut().enumerate().take(len2 + 1) {
        *cell = j;
    }

    for i in 1..=len1 {
        for j in 1..=len2 {
            let cost = if words1[i - 1] == words2[j - 1] { 0 } else { 1 };

            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[len1][len2]
}

/// Calculate normalized word-level Levenshtein similarity
/// For Japanese text, uses character-based comparison instead of whitespace splitting
pub fn word_levenshtein_similarity(s1: &str, s2: &str) -> f64 {
    let distance = word_levenshtein_distance(s1, s2);

    // Check if text contains Japanese characters
    let has_japanese = |text: &str| {
        text.chars().any(|c| {
            // ひらがな (U+3040-U+309F)
            ('\u{3040}'..='\u{309F}').contains(&c) ||
            // カタカナ (U+30A0-U+30FF)
            ('\u{30A0}'..='\u{30FF}').contains(&c) ||
            // 漢字 (U+4E00-U+9FAF)
            ('\u{4E00}'..='\u{9FAF}').contains(&c)
        })
    };

    let max_len = if has_japanese(s1) || has_japanese(s2) {
        // For Japanese text, use character count
        s1.chars().count().max(s2.chars().count())
    } else {
        // For non-Japanese text, use word count
        let words1_count = s1.split_whitespace().count();
        let words2_count = s2.split_whitespace().count();
        words1_count.max(words2_count)
    };

    if max_len == 0 {
        return 1.0;
    }

    1.0 - (distance as f64 / max_len as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", "ab"), 1);
        assert_eq!(levenshtein_distance("abc", "abcd"), 1);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
    }

    #[test]
    fn test_levenshtein_similarity() {
        assert_eq!(levenshtein_similarity("abc", "abc"), 1.0);
        assert_eq!(levenshtein_similarity("", ""), 1.0);
        assert!((levenshtein_similarity("abc", "ab") - 0.6666666666666667).abs() < 0.0001);
    }

    #[test]
    fn test_word_levenshtein_distance() {
        assert_eq!(word_levenshtein_distance("hello world", "hello world"), 0);
        assert_eq!(word_levenshtein_distance("hello world", "hello"), 1);
        assert_eq!(word_levenshtein_distance("hello world", "world hello"), 2);
    }

    #[test]
    fn test_word_levenshtein_similarity() {
        assert_eq!(word_levenshtein_similarity("hello world", "hello world"), 1.0);
        assert_eq!(word_levenshtein_similarity("hello world", "hello"), 0.5);
    }
}
