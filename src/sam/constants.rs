//! Shared constants for SAM

// Special phoneme markers
pub const PHONEME_PERIOD: usize = 1;
pub const PHONEME_QUESTION: usize = 2;

// Phoneme flags
pub const FLAG_NASAL: u16 = 0x0800;
pub const FLAG_VOWEL: u16 = 0x0080;
pub const FLAG_CONSONANT: u16 = 0x0040;
pub const FLAG_DIP_YX: u16 = 0x0020;
pub const FLAG_DIPHTHONG: u16 = 0x0010;
pub const FLAG_VOICED: u16 = 0x0004;
pub const FLAG_STOPCONS: u16 = 0x0002;
pub const FLAG_UNVOICED_STOPCONS: u16 = 0x0001;
