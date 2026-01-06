//! SAM Reciter - converts English text to phonemes

pub mod tables;

use tables::*;

/// A parsed pronunciation rule
struct Rule {
    prefix: String,
    match_str: String,
    suffix: String,
    output: String,
}

impl Rule {
    /// Parse a rule string like "prefix(match)suffix=output"
    fn parse(rule_str: &str) -> Option<Self> {
        let parts: Vec<&str> = rule_str.split('=').collect();
        if parts.len() < 2 {
            return None;
        }

        // Handle '=' in the match (join all but last)
        let output = parts.last()?.to_string();
        let source = parts[..parts.len() - 1].join("=");

        // Parse prefix(match)suffix
        let open_paren = source.find('(')?;
        let close_paren = source.find(')')?;

        let prefix = source[..open_paren].to_string();
        let match_str = source[open_paren + 1..close_paren].to_string();
        let suffix = source[close_paren + 1..].to_string();

        Some(Rule {
            prefix,
            match_str,
            suffix,
            output,
        })
    }

    /// Check if this rule matches at the given position in text
    fn matches(&self, text: &str, pos: usize) -> bool {
        let chars: Vec<char> = text.chars().collect();

        // Check if match string matches at position
        let match_chars: Vec<char> = self.match_str.chars().collect();
        if pos + match_chars.len() > chars.len() {
            return false;
        }

        for (i, &mc) in match_chars.iter().enumerate() {
            if chars.get(pos + i) != Some(&mc) {
                return false;
            }
        }

        // Check prefix
        if !self.check_prefix(text, pos) {
            return false;
        }

        // Check suffix
        if !self.check_suffix(text, pos + self.match_str.len() - 1) {
            return false;
        }

        true
    }

    fn check_prefix(&self, text: &str, pos: usize) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let prefix_chars: Vec<char> = self.prefix.chars().collect();
        let mut check_pos = pos;

        for i in (0..prefix_chars.len()).rev() {
            let rule_byte = prefix_chars[i];

            if !has_flags(rule_byte, FLAG_ALPHA_OR_QUOT) {
                match rule_byte {
                    ' ' => {
                        // Previous char must not be alpha or quotation mark
                        if check_pos == 0 {
                            return true;
                        }
                        check_pos -= 1;
                        if has_flags(chars[check_pos], FLAG_ALPHA_OR_QUOT) {
                            return false;
                        }
                    }
                    '#' => {
                        // Previous char must be a vowel or Y
                        if check_pos == 0 {
                            return false;
                        }
                        check_pos -= 1;
                        if !has_flags(chars[check_pos], FLAG_VOWEL_OR_Y) {
                            return false;
                        }
                    }
                    '.' => {
                        // Previous char must have FLAG_0X08
                        if check_pos == 0 {
                            return false;
                        }
                        check_pos -= 1;
                        if !has_flags(chars[check_pos], FLAG_0X08) {
                            return false;
                        }
                    }
                    '&' => {
                        // Previous char must be diphthong or CH/SH
                        if check_pos == 0 {
                            return false;
                        }
                        check_pos -= 1;
                        if has_flags(chars[check_pos], FLAG_DIPHTHONG) {
                            continue;
                        }
                        if check_pos >= 1 {
                            let two: String = chars[check_pos - 1..=check_pos].iter().collect();
                            if two == "CH" || two == "SH" {
                                check_pos -= 1;
                                continue;
                            }
                        }
                        return false;
                    }
                    '@' => {
                        // Previous char must be voiced and not H
                        if check_pos == 0 {
                            return false;
                        }
                        check_pos -= 1;
                        if has_flags(chars[check_pos], FLAG_VOICED) {
                            continue;
                        }
                        if chars[check_pos] == 'H' {
                            continue; // Original code has a bug here, we match it
                        }
                        return false;
                    }
                    '^' => {
                        // Previous char must be a consonant
                        if check_pos == 0 {
                            return false;
                        }
                        check_pos -= 1;
                        if !has_flags(chars[check_pos], FLAG_CONSONANT) {
                            return false;
                        }
                    }
                    '+' => {
                        // Previous char must be E, I, or Y
                        if check_pos == 0 {
                            return false;
                        }
                        check_pos -= 1;
                        let c = chars[check_pos];
                        if c != 'E' && c != 'I' && c != 'Y' {
                            return false;
                        }
                    }
                    ':' => {
                        // Walk left until non-consonant
                        while check_pos > 0 && has_flags(chars[check_pos - 1], FLAG_CONSONANT) {
                            check_pos -= 1;
                        }
                    }
                    _ => return false,
                }
            } else {
                // Literal character match
                if check_pos == 0 {
                    return false;
                }
                check_pos -= 1;
                if chars[check_pos] != rule_byte {
                    return false;
                }
            }
        }
        true
    }

    fn check_suffix(&self, text: &str, pos: usize) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let suffix_chars: Vec<char> = self.suffix.chars().collect();
        let mut check_pos = pos;

        for rule_byte in suffix_chars {
            if !has_flags(rule_byte, FLAG_ALPHA_OR_QUOT) {
                match rule_byte {
                    ' ' => {
                        // Next char must not be alpha or quotation mark
                        check_pos += 1;
                        if check_pos >= chars.len() {
                            return true;
                        }
                        if has_flags(chars[check_pos], FLAG_ALPHA_OR_QUOT) {
                            return false;
                        }
                    }
                    '#' => {
                        // Next char must be a vowel or Y
                        check_pos += 1;
                        if check_pos >= chars.len() {
                            return false;
                        }
                        if !has_flags(chars[check_pos], FLAG_VOWEL_OR_Y) {
                            return false;
                        }
                    }
                    '.' => {
                        // Next char must have FLAG_0X08
                        check_pos += 1;
                        if check_pos >= chars.len() {
                            return false;
                        }
                        if !has_flags(chars[check_pos], FLAG_0X08) {
                            return false;
                        }
                    }
                    '&' => {
                        // Next char must be diphthong or HC/HS
                        check_pos += 1;
                        if check_pos >= chars.len() {
                            return false;
                        }
                        if has_flags(chars[check_pos], FLAG_DIPHTHONG) {
                            continue;
                        }
                        check_pos += 1;
                        if check_pos >= chars.len() {
                            return false;
                        }
                        let two: String = chars[check_pos - 1..=check_pos].iter().collect();
                        if two != "HC" && two != "HS" {
                            return false;
                        }
                    }
                    '@' => {
                        // Next char must be voiced
                        check_pos += 1;
                        if check_pos >= chars.len() {
                            return false;
                        }
                        if has_flags(chars[check_pos], FLAG_VOICED) {
                            continue;
                        }
                        if chars[check_pos] == 'H' {
                            continue;
                        }
                        return false;
                    }
                    '^' => {
                        // Next char must be a consonant
                        check_pos += 1;
                        if check_pos >= chars.len() {
                            return false;
                        }
                        if !has_flags(chars[check_pos], FLAG_CONSONANT) {
                            return false;
                        }
                    }
                    '+' => {
                        // Next char must be E, I, or Y
                        check_pos += 1;
                        if check_pos >= chars.len() {
                            return false;
                        }
                        let c = chars[check_pos];
                        if c != 'E' && c != 'I' && c != 'Y' {
                            return false;
                        }
                    }
                    ':' => {
                        // Walk right until non-consonant
                        while check_pos + 1 < chars.len()
                            && has_flags(chars[check_pos + 1], FLAG_CONSONANT)
                        {
                            check_pos += 1;
                        }
                    }
                    '%' => {
                        // Check for ING, E not followed by alpha, ER/ES/ED, EFUL, ELY
                        check_pos += 1;
                        if check_pos >= chars.len() {
                            return false;
                        }
                        if chars[check_pos] != 'E' {
                            // Check for ING
                            if check_pos + 2 < chars.len() {
                                let three: String =
                                    chars[check_pos..check_pos + 3].iter().collect();
                                if three == "ING" {
                                    check_pos += 2;
                                    continue;
                                }
                            }
                            return false;
                        }
                        // We have E
                        if check_pos + 1 >= chars.len()
                            || !has_flags(chars[check_pos + 1], FLAG_ALPHA_OR_QUOT)
                        {
                            continue;
                        }
                        // Check ER, ES, ED
                        let next = chars[check_pos + 1];
                        if next == 'R' || next == 'S' || next == 'D' {
                            check_pos += 1;
                            continue;
                        }
                        // Check EL -> ELY
                        if next == 'L' {
                            if check_pos + 2 < chars.len() && chars[check_pos + 2] == 'Y' {
                                check_pos += 2;
                                continue;
                            }
                            return false;
                        }
                        // Check EFUL
                        if check_pos + 3 < chars.len() {
                            let three: String =
                                chars[check_pos + 1..check_pos + 4].iter().collect();
                            if three == "FUL" {
                                check_pos += 3;
                                continue;
                            }
                        }
                        return false;
                    }
                    _ => return false,
                }
            } else {
                // Literal character match
                check_pos += 1;
                if check_pos >= chars.len() {
                    return false;
                }
                if chars[check_pos] != rule_byte {
                    return false;
                }
            }
        }
        true
    }
}

/// Convert English text to SAM phonemes
///
/// # Arguments
/// * `input` - English text to convert
///
/// # Returns
/// Phoneme string suitable for the SAM parser
pub fn text_to_phonemes(input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }

    let text = format!(" {} ", input.to_uppercase());
    let chars: Vec<char> = text.chars().collect();

    // Parse all rules once
    let rules: Vec<Rule> = RULES
        .split('|')
        .filter_map(|r| Rule::parse(r))
        .collect();

    let rules2: Vec<Rule> = RULES2
        .split('|')
        .filter_map(|r| Rule::parse(r))
        .collect();

    // Group rules by first character of match
    let mut rules_by_char: std::collections::HashMap<char, Vec<&Rule>> =
        std::collections::HashMap::new();
    for rule in &rules {
        if let Some(c) = rule.match_str.chars().next() {
            rules_by_char.entry(c).or_default().push(rule);
        }
    }

    let mut output = String::new();
    let mut pos = 0;

    while pos < chars.len() {
        let current_char = chars[pos];

        // Check for period not followed by number
        if current_char == '.'
            && (pos + 1 >= chars.len() || !has_flags(chars[pos + 1], FLAG_NUMERIC))
        {
            output.push('.');
            pos += 1;
            continue;
        }

        // Use ruleset2 for special characters
        if has_flags(current_char, FLAG_RULESET2) {
            let mut matched = false;
            for rule in &rules2 {
                if rule.matches(&text, pos) {
                    output.push_str(&rule.output);
                    pos += rule.match_str.len();
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
        }

        // Check if character has any flags
        if char_flags(current_char) != 0 {
            if !has_flags(current_char, FLAG_ALPHA_OR_QUOT) {
                // Unknown character type
                pos += 1;
                continue;
            }

            // Try rules for this character
            if let Some(char_rules) = rules_by_char.get(&current_char) {
                let mut matched = false;
                for rule in char_rules {
                    if rule.matches(&text, pos) {
                        output.push_str(&rule.output);
                        pos += rule.match_str.len();
                        matched = true;
                        break;
                    }
                }
                if matched {
                    continue;
                }
            }
        }

        // No rule matched - output space
        output.push(' ');
        pos += 1;
    }

    if output.trim().is_empty() {
        None
    } else {
        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        let result = text_to_phonemes("HELLO");
        assert!(result.is_some());
        let phonemes = result.unwrap();
        assert!(!phonemes.is_empty());
        // Should contain H, EH, L sounds
        assert!(phonemes.contains("/H") || phonemes.contains("HH"));
    }

    #[test]
    fn test_empty() {
        let result = text_to_phonemes("");
        assert!(result.is_none());
    }

    #[test]
    fn test_the() {
        let result = text_to_phonemes("THE");
        assert!(result.is_some());
        let phonemes = result.unwrap();
        // THE should produce DH sounds
        assert!(phonemes.contains("DH"));
    }

    #[test]
    fn test_numbers() {
        let result = text_to_phonemes("123");
        assert!(result.is_some());
    }
}
