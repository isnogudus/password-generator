use rand::Rng;
use rand::seq::SliceRandom;

/// Standardlänge eines Passworts (Anzahl Zeichen ohne Trennzeichen)
pub const DEFAULT_LENGTH: usize = 12;
/// Standardlänge im Kleinbuchstaben-Modus (gleicht den kleineren Vorrat aus)
pub const DEFAULT_LOWERCASE_LENGTH: usize = 16;
/// Kürzestes erlaubtes Passwort
pub const MIN_LENGTH: usize = 4;
/// Längstes erlaubtes Passwort
pub const MAX_LENGTH: usize = 128;
/// Blockgröße für die Darstellung
pub const BLOCK_SIZE: usize = 4;
/// Standard-Trennzeichen zwischen den Blöcken
pub const DEFAULT_SEPARATOR: &str = ".";
/// Längstes erlaubtes Trennzeichen (in Zeichen)
pub const MAX_SEPARATOR_LEN: usize = 8;

/// Kleinbuchstaben ohne leicht verwechselbare Zeichen (l, o) und ohne y/z
/// (auf QWERTY- und QWERTZ-Tastaturen vertauscht).
const LOWER: &[u8] = b"abcdefghijkmnpqrstuvwx";
/// Großbuchstaben ohne leicht verwechselbare Zeichen (I, O) und ohne Y/Z.
const UPPER: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWX";
/// Ziffern ohne 0 (ähnlich O) und 1 (ähnlich l/I).
const DIGITS: &[u8] = b"23456789";
/// Sonderzeichen für den Strict-Modus: auf QWERTY und QWERTZ vorhanden, von
/// gängigen Passwortrichtlinien akzeptiert, ohne Quoting-Fallen (`'"\``) und
/// ohne verwechselbare Zeichen (`|`).
const SPECIAL: &[u8] = b"!#$%&*+=?@_";

/// Einstellungen für die Passwort-Erzeugung
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Anzahl Zeichen ohne Trennzeichen
    pub length: usize,
    /// Trennzeichen zwischen den Viererblöcken; leer = keine Blöcke
    pub separator: String,
    /// Gemischter Modus: Sonderzeichen hinzufügen und mindestens eines
    /// garantieren. Kleinbuchstaben-Modus: entspricht `upper`, `digit` und
    /// `special` zusammen.
    pub strict: bool,
    /// Kleinbuchstaben-Modus: Grundvorrat sind nur Kleinbuchstaben
    pub lowercase: bool,
    /// Kleinbuchstaben-Modus: genau ein Großbuchstabe
    pub upper: bool,
    /// Kleinbuchstaben-Modus: genau eine Ziffer
    pub digit: bool,
    /// Kleinbuchstaben-Modus: genau ein Sonderzeichen
    pub special: bool,
}

impl Options {
    /// Standardlänge für den jeweiligen Modus
    pub fn default_length(lowercase: bool) -> usize {
        if lowercase {
            DEFAULT_LOWERCASE_LENGTH
        } else {
            DEFAULT_LENGTH
        }
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            separator: DEFAULT_SEPARATOR.to_string(),
            strict: false,
            lowercase: false,
            upper: false,
            digit: false,
            special: false,
        }
    }
}

/// Fehler bei ungültigen Einstellungen
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidLength(usize),
    SeparatorTooLong(usize),
    RequiresLowercase,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidLength(n) => write!(
                f,
                "Ungültige Länge {n}: erlaubt sind {MIN_LENGTH} bis {MAX_LENGTH} Zeichen"
            ),
            Error::SeparatorTooLong(n) => write!(
                f,
                "Trennzeichen zu lang ({n}): erlaubt sind höchstens {MAX_SEPARATOR_LEN} Zeichen"
            ),
            Error::RequiresLowercase => write!(
                f,
                "upper, digit und special gelten nur im Kleinbuchstaben-Modus (lowercase)"
            ),
        }
    }
}

impl std::error::Error for Error {}

/// Gibt ein zufälliges Zeichen aus dem angegebenen Alphabet zurück
fn random_char<R: Rng>(rng: &mut R, alphabet: &[u8]) -> char {
    alphabet[rng.gen_range(0..alphabet.len())] as char
}

/// Sonderzeichen ohne die Zeichen, die auch im Trennzeichen vorkommen, damit
/// Blockgrenze und Inhalt unterscheidbar bleiben.
fn special_without_separator(separator: &str) -> Vec<u8> {
    SPECIAL
        .iter()
        .copied()
        .filter(|c| !separator.contains(*c as char))
        .collect()
}

/// Erzeugt die rohen Passwortzeichen (ohne Trennzeichen).
///
/// Gemischter Modus: garantiert je mindestens ein Klein-, ein Großbuchstabe
/// und eine Ziffer, im Strict-Modus zusätzlich ein Sonderzeichen; die
/// restlichen Zeichen kommen aus dem Gesamtalphabet.
///
/// Kleinbuchstaben-Modus: alle Zeichen sind Kleinbuchstaben, bis auf genau
/// einen Großbuchstaben, eine Ziffer bzw. ein Sonderzeichen, sofern
/// zugeschaltet.
///
/// Anschließend wird gemischt, damit die Pflichtzeichen keine feste Position haben.
fn generate_chars<R: Rng>(rng: &mut R, opts: &Options) -> Vec<char> {
    let special = special_without_separator(&opts.separator);
    let mut chars: Vec<char> = Vec::with_capacity(opts.length);

    if opts.lowercase {
        if opts.upper || opts.strict {
            chars.push(random_char(rng, UPPER));
        }
        if opts.digit || opts.strict {
            chars.push(random_char(rng, DIGITS));
        }
        if opts.special || opts.strict {
            chars.push(random_char(rng, &special));
        }
        while chars.len() < opts.length {
            chars.push(random_char(rng, LOWER));
        }
    } else {
        let mut all: Vec<u8> = [LOWER, UPPER, DIGITS].concat();
        if opts.strict {
            all.extend_from_slice(&special);
        }
        chars.push(random_char(rng, LOWER));
        chars.push(random_char(rng, UPPER));
        chars.push(random_char(rng, DIGITS));
        if opts.strict {
            chars.push(random_char(rng, &special));
        }
        while chars.len() < opts.length {
            chars.push(random_char(rng, &all));
        }
    }

    chars.shuffle(rng);
    chars
}

/// Formatiert die Zeichen in Blöcken zu je `BLOCK_SIZE` Zeichen, getrennt durch `separator`
fn format_blocks(chars: &[char], separator: &str) -> String {
    if separator.is_empty() {
        return chars.iter().collect();
    }
    chars
        .chunks(BLOCK_SIZE)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(separator)
}

/// Prüft die Einstellungen auf Gültigkeit
pub fn validate(opts: &Options) -> Result<(), Error> {
    if !(MIN_LENGTH..=MAX_LENGTH).contains(&opts.length) {
        return Err(Error::InvalidLength(opts.length));
    }
    let sep_len = opts.separator.chars().count();
    if sep_len > MAX_SEPARATOR_LEN {
        return Err(Error::SeparatorTooLong(sep_len));
    }
    if !opts.lowercase && (opts.upper || opts.digit || opts.special) {
        return Err(Error::RequiresLowercase);
    }
    Ok(())
}

/// Erzeugt ein zufälliges Passwort gemäß `opts`, z.B. `aB3d.Ef7g.H9jk`
pub fn generate_password(opts: &Options) -> Result<String, Error> {
    validate(opts)?;
    let mut rng = rand::thread_rng();
    Ok(format_blocks(
        &generate_chars(&mut rng, opts),
        &opts.separator,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(length: usize) -> Options {
        Options {
            length,
            ..Options::default()
        }
    }

    fn strip(password: &str, separator: &str) -> String {
        if separator.is_empty() {
            return password.to_string();
        }
        password.replace(separator, "")
    }

    #[test]
    fn default_length_has_three_blocks() {
        let pw = generate_password(&opts(DEFAULT_LENGTH)).unwrap();
        assert_eq!(pw.len(), 14);
        assert_eq!(pw.split('.').count(), 3);
        assert!(pw.split('.').all(|b| b.len() == BLOCK_SIZE));
    }

    #[test]
    fn odd_length_has_short_last_block() {
        let pw = generate_password(&opts(10)).unwrap();
        let blocks: Vec<&str> = pw.split('.').collect();
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[2].len(), 2);
        assert_eq!(strip(&pw, ".").len(), 10);
    }

    #[test]
    fn custom_separator() {
        let o = Options {
            length: 8,
            separator: "--".to_string(),
            strict: false,
            ..Options::default()
        };
        let pw = generate_password(&o).unwrap();
        assert_eq!(pw.len(), 10);
        assert_eq!(pw.split("--").count(), 2);
    }

    #[test]
    fn empty_separator_has_no_blocks() {
        let o = Options {
            length: 12,
            separator: String::new(),
            strict: false,
            ..Options::default()
        };
        let pw = generate_password(&o).unwrap();
        assert_eq!(pw.len(), 12);
        assert!(pw.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn excludes_ambiguous_and_yz() {
        let forbidden = "0O1lIyzYZ";
        for _ in 0..500 {
            let pw = strip(&generate_password(&opts(32)).unwrap(), ".");
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
            let pw = strip(&generate_password(&opts(MIN_LENGTH)).unwrap(), ".");
            assert!(pw.chars().any(|c| c.is_ascii_lowercase()), "{pw}");
            assert!(pw.chars().any(|c| c.is_ascii_uppercase()), "{pw}");
            assert!(pw.chars().any(|c| c.is_ascii_digit()), "{pw}");
        }
    }

    #[test]
    fn strict_contains_special_and_all_classes() {
        let o = Options {
            length: MIN_LENGTH,
            separator: String::new(),
            strict: true,
            ..Options::default()
        };
        let special: Vec<char> = SPECIAL.iter().map(|c| *c as char).collect();
        for _ in 0..200 {
            let pw = generate_password(&o).unwrap();
            assert!(pw.chars().any(|c| c.is_ascii_lowercase()), "{pw}");
            assert!(pw.chars().any(|c| c.is_ascii_uppercase()), "{pw}");
            assert!(pw.chars().any(|c| c.is_ascii_digit()), "{pw}");
            assert!(pw.chars().any(|c| special.contains(&c)), "{pw}");
            assert!(
                pw.chars()
                    .all(|c| c.is_ascii_alphanumeric() || special.contains(&c)),
                "{pw}"
            );
        }
    }

    #[test]
    fn strict_never_uses_separator_char() {
        let o = Options {
            length: 64,
            separator: "_".to_string(),
            strict: true,
            ..Options::default()
        };
        for _ in 0..100 {
            let pw = generate_password(&o).unwrap();
            // 64 Zeichen in 16 Blöcken = 15 Trennzeichen, keines im Inhalt
            assert_eq!(pw.matches('_').count(), 15, "{pw}");
        }
    }

    fn count_classes(pw: &str) -> (usize, usize, usize, usize) {
        let special: Vec<char> = SPECIAL.iter().map(|c| *c as char).collect();
        (
            pw.chars().filter(|c| c.is_ascii_lowercase()).count(),
            pw.chars().filter(|c| c.is_ascii_uppercase()).count(),
            pw.chars().filter(|c| c.is_ascii_digit()).count(),
            pw.chars().filter(|c| special.contains(c)).count(),
        )
    }

    #[test]
    fn lowercase_only() {
        let o = Options {
            length: 16,
            separator: String::new(),
            lowercase: true,
            ..Options::default()
        };
        for _ in 0..200 {
            let pw = generate_password(&o).unwrap();
            assert_eq!(pw.len(), 16);
            assert_eq!(count_classes(&pw), (16, 0, 0, 0), "{pw}");
            assert!(!pw.chars().any(|c| "loyz".contains(c)), "{pw}");
        }
    }

    #[test]
    fn lowercase_with_single_extras() {
        let base = Options {
            length: 16,
            separator: String::new(),
            lowercase: true,
            ..Options::default()
        };
        let cases = [
            (true, false, false, (15, 1, 0, 0)),
            (false, true, false, (15, 0, 1, 0)),
            (false, false, true, (15, 0, 0, 1)),
            (true, true, true, (13, 1, 1, 1)),
        ];
        for (upper, digit, special, expected) in cases {
            let o = Options {
                upper,
                digit,
                special,
                ..base.clone()
            };
            for _ in 0..100 {
                let pw = generate_password(&o).unwrap();
                assert_eq!(count_classes(&pw), expected, "{pw}");
            }
        }
    }

    #[test]
    fn lowercase_strict_equals_all_extras() {
        let o = Options {
            length: MIN_LENGTH,
            separator: String::new(),
            lowercase: true,
            strict: true,
            ..Options::default()
        };
        for _ in 0..100 {
            let pw = generate_password(&o).unwrap();
            assert_eq!(count_classes(&pw), (1, 1, 1, 1), "{pw}");
        }
    }

    #[test]
    fn default_length_per_mode() {
        assert_eq!(Options::default_length(false), 12);
        assert_eq!(Options::default_length(true), 16);
    }

    #[test]
    fn extras_require_lowercase() {
        let o = Options {
            upper: true,
            ..Options::default()
        };
        assert_eq!(generate_password(&o), Err(Error::RequiresLowercase));
    }

    #[test]
    fn rejects_invalid_options() {
        assert_eq!(generate_password(&opts(0)), Err(Error::InvalidLength(0)));
        assert_eq!(
            generate_password(&opts(MIN_LENGTH - 1)),
            Err(Error::InvalidLength(3))
        );
        assert_eq!(
            generate_password(&opts(MAX_LENGTH + 1)),
            Err(Error::InvalidLength(129))
        );
        assert!(generate_password(&opts(MAX_LENGTH)).is_ok());

        let long_sep = Options {
            separator: "-".repeat(MAX_SEPARATOR_LEN + 1),
            ..Options::default()
        };
        assert_eq!(
            generate_password(&long_sep),
            Err(Error::SeparatorTooLong(MAX_SEPARATOR_LEN + 1))
        );
    }
}
