use rand::Rng;
use rand::seq::SliceRandom;

/// Standardlänge eines Passworts (Anzahl Zeichen ohne Trennzeichen)
pub const DEFAULT_LENGTH: usize = 12;
/// Kürzestes erlaubtes Passwort
pub const MIN_LENGTH: usize = 4;
/// Längstes erlaubtes Passwort
pub const MAX_LENGTH: usize = 128;
/// Blockgröße für die Darstellung
pub const BLOCK_SIZE: usize = 4;
/// Trennzeichen zwischen den Blöcken
pub const SEPARATOR: char = '.';

/// Kleinbuchstaben ohne leicht verwechselbare Zeichen (l, o) und ohne y/z
/// (auf QWERTY- und QWERTZ-Tastaturen vertauscht).
const LOWER: &[u8] = b"abcdefghijkmnpqrstuvwx";
/// Großbuchstaben ohne leicht verwechselbare Zeichen (I, O) und ohne Y/Z.
const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWX";
/// Ziffern ohne 0 (ähnlich O) und 1 (ähnlich l/I).
const DIGITS: &[u8] = b"23456789";

/// Fehler bei ungültiger Passwortlänge
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidLength(pub usize);

impl std::fmt::Display for InvalidLength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Ungültige Länge {}: erlaubt sind {} bis {} Zeichen",
            self.0, MIN_LENGTH, MAX_LENGTH
        )
    }
}

/// Gibt ein zufälliges Zeichen aus dem angegebenen Alphabet zurück
fn random_char<R: Rng>(rng: &mut R, alphabet: &[u8]) -> char {
    alphabet[rng.gen_range(0..alphabet.len())] as char
}

/// Erzeugt die rohen Passwortzeichen (ohne Trennzeichen).
///
/// Es ist garantiert je mindestens ein Klein-, ein Großbuchstabe und eine
/// Ziffer enthalten; die restlichen Zeichen kommen aus dem Gesamtalphabet.
/// Anschließend wird gemischt, damit die Pflichtzeichen keine feste Position haben.
fn generate_chars<R: Rng>(rng: &mut R, length: usize) -> Vec<char> {
    let all: Vec<u8> = [LOWER, UPPER, DIGITS].concat();

    let mut chars: Vec<char> = Vec::with_capacity(length);
    chars.push(random_char(rng, LOWER));
    chars.push(random_char(rng, UPPER));
    chars.push(random_char(rng, DIGITS));
    while chars.len() < length {
        chars.push(random_char(rng, &all));
    }
    chars.shuffle(rng);
    chars
}

/// Formatiert die Zeichen in Blöcken zu je `BLOCK_SIZE` Zeichen, getrennt durch `SEPARATOR`
fn format_blocks(chars: &[char]) -> String {
    chars
        .chunks(BLOCK_SIZE)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(&SEPARATOR.to_string())
}

/// Erzeugt ein zufälliges Passwort mit `length` Zeichen, dargestellt in
/// Viererblöcken, z.B. `aB3d.Ef7g.H9jk`
pub fn generate_password(length: usize) -> Result<String, InvalidLength> {
    if !(MIN_LENGTH..=MAX_LENGTH).contains(&length) {
        return Err(InvalidLength(length));
    }
    let mut rng = rand::thread_rng();
    Ok(format_blocks(&generate_chars(&mut rng, length)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strip(password: &str) -> String {
        password.chars().filter(|c| *c != SEPARATOR).collect()
    }

    #[test]
    fn default_length_has_three_blocks() {
        let pw = generate_password(DEFAULT_LENGTH).unwrap();
        assert_eq!(pw.len(), 14);
        assert_eq!(pw.split(SEPARATOR).count(), 3);
        assert!(pw.split(SEPARATOR).all(|b| b.len() == BLOCK_SIZE));
    }

    #[test]
    fn odd_length_has_short_last_block() {
        let pw = generate_password(10).unwrap();
        let blocks: Vec<&str> = pw.split(SEPARATOR).collect();
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[2].len(), 2);
        assert_eq!(strip(&pw).len(), 10);
    }

    #[test]
    fn excludes_ambiguous_and_yz() {
        let forbidden = "0O1lIyzYZ";
        for _ in 0..500 {
            let pw = strip(&generate_password(32).unwrap());
            assert!(
                !pw.chars().any(|c| forbidden.contains(c)),
                "verbotenes Zeichen in {pw}"
            );
            assert!(pw.chars().all(|c| c.is_ascii_alphanumeric()));
        }
    }

    #[test]
    fn contains_each_class() {
        for _ in 0..200 {
            let pw = strip(&generate_password(MIN_LENGTH).unwrap());
            assert!(pw.chars().any(|c| c.is_ascii_lowercase()), "{pw}");
            assert!(pw.chars().any(|c| c.is_ascii_uppercase()), "{pw}");
            assert!(pw.chars().any(|c| c.is_ascii_digit()), "{pw}");
        }
    }

    #[test]
    fn rejects_out_of_range() {
        assert_eq!(generate_password(0), Err(InvalidLength(0)));
        assert_eq!(generate_password(MIN_LENGTH - 1), Err(InvalidLength(3)));
        assert_eq!(generate_password(MAX_LENGTH + 1), Err(InvalidLength(129)));
        assert!(generate_password(MAX_LENGTH).is_ok());
    }
}
