//! SAM Parser - converts phoneme strings to phoneme data for rendering

pub mod tables;

use tables::{
    find_phoneme, phoneme_length, phoneme_stressed_length, PHONEME_FLAGS, STRESS_TABLE,
};

use crate::sam::constants::*;

/// Parsed phoneme: (phoneme_index, length, stress)
pub type ParsedPhoneme = (usize, usize, usize);

/// Parse a phoneme string into phoneme data for rendering
///
/// # Arguments
/// * `input` - Phoneme string (uppercase, e.g., "HEHLOW WERLD")
///
/// # Returns
/// Vector of (phoneme_index, length, stress) tuples, or None if parsing fails
pub fn parse(input: &str) -> Option<Vec<ParsedPhoneme>> {
    if input.is_empty() {
        return None;
    }

    let input = input.to_uppercase();
    let mut phoneme_index: Vec<usize> = Vec::new();
    let mut phoneme_len: Vec<usize> = Vec::new();
    let mut stress: Vec<usize> = Vec::new();

    // Parse input into phonemes
    parse_phonemes(&input, &mut phoneme_index, &mut phoneme_len, &mut stress);

    if phoneme_index.is_empty() {
        return None;
    }

    // Apply parser transformations
    apply_parser2(&mut phoneme_index, &mut phoneme_len, &mut stress);
    copy_stress(&phoneme_index, &mut stress);
    set_phoneme_length(&phoneme_index, &stress, &mut phoneme_len);
    adjust_lengths(&phoneme_index, &mut phoneme_len);
    prolong_plosives(&mut phoneme_index, &mut phoneme_len, &mut stress);

    // Build result
    let result: Vec<ParsedPhoneme> = phoneme_index
        .iter()
        .enumerate()
        .filter_map(|(i, &p)| {
            if p > 0 {
                Some((p, phoneme_len[i], stress[i]))
            } else {
                None
            }
        })
        .collect();

    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}

/// Parse input string into phoneme indices
fn parse_phonemes(
    input: &str,
    phoneme_index: &mut Vec<usize>,
    phoneme_len: &mut Vec<usize>,
    stress: &mut Vec<usize>,
) {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Check for stress marker
        if let Some(stress_pos) = STRESS_TABLE.iter().position(|&s| s == c as u8) {
            if !phoneme_index.is_empty() {
                let last_idx = stress.len() - 1;
                stress[last_idx] = stress_pos;
            }
            i += 1;
            continue;
        }

        // Try to match 2-character phoneme first
        if i + 1 < chars.len() {
            let two_char: String = chars[i..=i + 1].iter().collect();
            if let Some(idx) = find_phoneme(&two_char) {
                phoneme_index.push(idx);
                phoneme_len.push(0);
                stress.push(0);
                i += 2;
                continue;
            }
        }

        // Try single character with wildcard
        let one_char = format!("{}*", c);
        if let Some(idx) = find_phoneme(&one_char) {
            phoneme_index.push(idx);
            phoneme_len.push(0);
            stress.push(0);
            i += 1;
            continue;
        }

        // Handle space as silence
        if c == ' ' {
            phoneme_index.push(0); // ' *' phoneme
            phoneme_len.push(0);
            stress.push(0);
        }

        i += 1;
    }
}

/// Apply parser2 transformations (insert phonemes, handle diphthongs, etc.)
fn apply_parser2(
    phoneme_index: &mut Vec<usize>,
    phoneme_len: &mut Vec<usize>,
    stress: &mut Vec<usize>,
) {
    let mut i = 0;
    while i < phoneme_index.len() {
        let phoneme = phoneme_index[i];
        if phoneme >= PHONEME_FLAGS.len() {
            i += 1;
            continue;
        }

        let flags = PHONEME_FLAGS[phoneme];

        // Handle diphthongs - insert second part
        if (flags & FLAG_DIPHTHONG) != 0 {
            // Diphthongs need a second phoneme inserted
            let second = if (flags & FLAG_DIP_YX) != 0 { 21 } else { 20 }; // YX or WX
            insert_phoneme(phoneme_index, phoneme_len, stress, i + 1, second, stress[i]);
            i += 1;
        }

        i += 1;
    }
}

/// Copy stress to following phonemes
fn copy_stress(phoneme_index: &[usize], stress: &mut [usize]) {
    for i in 0..phoneme_index.len().saturating_sub(1) {
        if stress[i] != 0 {
            let phoneme = phoneme_index[i];
            let next_phoneme = phoneme_index[i + 1];

            if phoneme < PHONEME_FLAGS.len() && next_phoneme < PHONEME_FLAGS.len() {
                let flags = PHONEME_FLAGS[phoneme];
                let next_flags = PHONEME_FLAGS[next_phoneme];

                // Copy stress to following vowels/consonants
                if (flags & FLAG_VOWEL) != 0 && (next_flags & (FLAG_VOWEL | FLAG_CONSONANT)) != 0 {
                    if stress[i + 1] == 0 {
                        stress[i + 1] = stress[i] + 1;
                    }
                }
            }
        }
    }
}

/// Set phoneme lengths based on stress
fn set_phoneme_length(
    phoneme_index: &[usize],
    stress: &[usize],
    phoneme_len: &mut [usize],
) {
    for (i, &phoneme) in phoneme_index.iter().enumerate() {
        if phoneme == 0 {
            continue;
        }

        let length = if stress[i] == 0 || stress[i] > 0x7F {
            phoneme_length(phoneme)
        } else {
            phoneme_stressed_length(phoneme)
        };

        phoneme_len[i] = length as usize;
    }
}

/// Adjust phoneme lengths based on context
fn adjust_lengths(phoneme_index: &[usize], phoneme_len: &mut [usize]) {
    for i in 0..phoneme_index.len().saturating_sub(1) {
        let phoneme = phoneme_index[i];
        let next_phoneme = phoneme_index[i + 1];

        if phoneme >= PHONEME_FLAGS.len() || next_phoneme >= PHONEME_FLAGS.len() {
            continue;
        }

        let flags = PHONEME_FLAGS[phoneme];
        let next_flags = PHONEME_FLAGS[next_phoneme];

        // Shorten vowels before unvoiced consonants
        if (flags & FLAG_VOWEL) != 0 && (next_flags & FLAG_CONSONANT) != 0 {
            if (next_flags & FLAG_VOICED) == 0 {
                // Unvoiced consonant following vowel - shorten vowel
                phoneme_len[i] = (phoneme_len[i] * 3) / 4;
            }
        }

        // Nasal before stop consonant
        if (flags & FLAG_NASAL) != 0 && (next_flags & FLAG_STOPCONS) != 0 {
            phoneme_len[i] = phoneme_len[i].saturating_add(2);
        }
    }
}

/// Prolong plosive stop consonants
fn prolong_plosives(
    phoneme_index: &mut Vec<usize>,
    phoneme_len: &mut Vec<usize>,
    stress: &mut Vec<usize>,
) {
    let mut i = 0;
    while i < phoneme_index.len() {
        let phoneme = phoneme_index[i];

        if phoneme >= PHONEME_FLAGS.len() {
            i += 1;
            continue;
        }

        let flags = PHONEME_FLAGS[phoneme];

        // Insert plosive variants for stop consonants
        if (flags & FLAG_STOPCONS) != 0 && (flags & FLAG_UNVOICED_STOPCONS) != 0 {
            // Insert aspirated version after unvoiced stop
            insert_phoneme(phoneme_index, phoneme_len, stress, i + 1, phoneme + 1, stress[i]);
            insert_phoneme(phoneme_index, phoneme_len, stress, i + 2, phoneme + 2, stress[i]);
            i += 2;
        }

        i += 1;
    }
}

/// Insert a phoneme at the given position
fn insert_phoneme(
    phoneme_index: &mut Vec<usize>,
    phoneme_len: &mut Vec<usize>,
    stress: &mut Vec<usize>,
    pos: usize,
    value: usize,
    stress_value: usize,
) {
    phoneme_index.insert(pos, value);
    phoneme_len.insert(pos, 0);
    stress.insert(pos, stress_value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hello() {
        let result = parse("HEHLOW");
        assert!(result.is_some());
        let phonemes = result.unwrap();
        assert!(!phonemes.is_empty());
    }

    #[test]
    fn test_parse_empty() {
        let result = parse("");
        assert!(result.is_none());
    }
}
