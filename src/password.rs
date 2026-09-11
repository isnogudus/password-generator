use rand::Rng;
use rand::seq::SliceRandom;

/// Standardlänge eines Passworts (Anzahl Zeichen ohne Trennzeichen)
pub const DEFAULT_LENGTH: usize = 16;
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
pub const LOWER: &str = "abcdefghijkmnpqrstuvwx";
/// Großbuchstaben ohne leicht verwechselbare Zeichen (I, O) und ohne Y/Z.
pub const UPPER: &str = "ABCDEFGHJKLMNPQRSTUVWX";
/// Alle Ziffern: da l, o, I und O in keinem Vorrat vorkommen, sind 0 und 1
/// nicht verwechselbar.
pub const DIGITS: &str = "0123456789";
/// Sonderzeichen: auf QWERTY und QWERTZ vorhanden, von gängigen
/// Passwortrichtlinien akzeptiert, ohne Quoting-Fallen (`'"\``) und ohne
/// verwechselbare Zeichen (`|`).
pub const SPECIAL: &str = "!#$%&*+=?@_";

/// Zeichenklasse
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Lower,
    Upper,
    Digits,
    Special,
}

impl Class {
    /// Alle Klassen in fester Reihenfolge
    pub const ALL: [Class; 4] = [Class::Lower, Class::Upper, Class::Digits, Class::Special];

    /// Name der Klasse, wie er in Optionen und Fehlermeldungen auftaucht
    pub fn name(self) -> &'static str {
        match self {
            Class::Lower => "lower",
            Class::Upper => "upper",
            Class::Digits => "digits",
            Class::Special => "special",
        }
    }
}

/// Eine Auswahl von Zeichenklassen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Classes {
    pub lower: bool,
    pub upper: bool,
    pub digits: bool,
    pub special: bool,
}

impl Classes {
    /// Grundvorrat, wenn keine Klasse angegeben ist: Klein-, Großbuchstaben und Ziffern
    pub const DEFAULT: Classes = Classes {
        lower: true,
        upper: true,
        digits: true,
        special: false,
    };

    /// Keine Klasse gewählt
    pub fn is_empty(self) -> bool {
        !(self.lower || self.upper || self.digits || self.special)
    }

    /// Ist die Klasse enthalten
    pub fn contains(self, class: Class) -> bool {
        match class {
            Class::Lower => self.lower,
            Class::Upper => self.upper,
            Class::Digits => self.digits,
            Class::Special => self.special,
        }
    }

    /// Klasse setzen oder entfernen
    pub fn set(&mut self, class: Class, on: bool) {
        match class {
            Class::Lower => self.lower = on,
            Class::Upper => self.upper = on,
            Class::Digits => self.digits = on,
            Class::Special => self.special = on,
        }
    }

    /// Enthaltene Klassen in fester Reihenfolge
    pub fn iter(self) -> impl Iterator<Item = Class> {
        Class::ALL.into_iter().filter(move |c| self.contains(*c))
    }
}

/// Einstellungen für die Passwort-Erzeugung
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Anzahl Zeichen ohne Trennzeichen
    pub length: usize,
    /// Trennzeichen zwischen den Viererblöcken; leer = keine Blöcke
    pub separator: String,
    /// Grundvorrat; leer bedeutet `Classes::DEFAULT`. Von jeder enthaltenen
    /// Klasse ist mindestens ein Zeichen garantiert.
    pub base: Classes,
    /// Extras: genau ein Zeichen aus dieser Klasse; nur für Klassen, die
    /// nicht im Grundvorrat sind
    pub one: Classes,
    /// Genau ein Zeichen aus jeder Klasse, die nicht im Grundvorrat ist
    pub strict: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            length: DEFAULT_LENGTH,
            separator: DEFAULT_SEPARATOR.to_string(),
            base: Classes::default(),
            one: Classes::default(),
            strict: false,
        }
    }
}

impl Options {
    /// Tatsächlicher Grundvorrat (leer -> Standard)
    pub fn effective_base(&self) -> Classes {
        if self.base.is_empty() {
            Classes::DEFAULT
        } else {
            self.base
        }
    }

    /// Tatsächliche Extras: explizite plus, bei `strict`, alle fehlenden Klassen
    pub fn effective_one(&self) -> Classes {
        let base = self.effective_base();
        let mut one = self.one;
        if self.strict {
            for class in Class::ALL {
                if !base.contains(class) {
                    one.set(class, true);
                }
            }
        }
        one
    }
}

/// Fehler bei ungültigen Einstellungen
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    InvalidLength(usize),
    SeparatorTooLong(usize),
    /// Extra für eine Klasse, die bereits im Grundvorrat ist
    OneInBase(Class),
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
            Error::OneInBase(class) => write!(
                f,
                "one-{} ist überflüssig: {} ist bereits im Grundvorrat",
                class.name().trim_end_matches('s'),
                class.name()
            ),
        }
    }
}

impl std::error::Error for Error {}

/// Zeichenvorrat einer Klasse; Sonderzeichen ohne die Zeichen des
/// Trennzeichens, damit Blockgrenze und Inhalt unterscheidbar bleiben.
fn alphabet(class: Class, separator: &str) -> Vec<char> {
    match class {
        Class::Lower => LOWER.chars().collect(),
        Class::Upper => UPPER.chars().collect(),
        Class::Digits => DIGITS.chars().collect(),
        Class::Special => SPECIAL
            .chars()
            .filter(|c| !separator.contains(*c))
            .collect(),
    }
}

/// Gibt ein zufälliges Zeichen aus dem angegebenen Alphabet zurück
fn random_char<R: Rng>(rng: &mut R, alphabet: &[char]) -> char {
    alphabet[rng.gen_range(0..alphabet.len())]
}

/// Erzeugt die rohen Passwortzeichen (ohne Trennzeichen): je ein Pflichtzeichen
/// pro Klasse des Grundvorrats und pro Extra, der Rest zufällig aus dem
/// Grundvorrat. Anschließend wird gemischt, damit die Pflichtzeichen keine
/// feste Position haben.
fn generate_chars<R: Rng>(rng: &mut R, opts: &Options) -> Vec<char> {
    let base = opts.effective_base();
    let one = opts.effective_one();

    let mut chars: Vec<char> = Vec::with_capacity(opts.length);
    let mut pool: Vec<char> = Vec::new();
    for class in base.iter() {
        let alpha = alphabet(class, &opts.separator);
        chars.push(random_char(rng, &alpha));
        pool.extend(alpha);
    }
    for class in one.iter() {
        chars.push(random_char(rng, &alphabet(class, &opts.separator)));
    }
    while chars.len() < opts.length {
        chars.push(random_char(rng, &pool));
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
    let base = opts.effective_base();
    if let Some(class) = opts.one.iter().find(|c| base.contains(*c)) {
        return Err(Error::OneInBase(class));
    }
    Ok(())
}

/// Erzeugt ein zufälliges Passwort gemäß `opts`, z.B. `aB3d.Ef7g.H9jk.Lm2n`
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

    fn classes(lower: bool, upper: bool, digits: bool, special: bool) -> Classes {
        Classes {
            lower,
            upper,
            digits,
            special,
        }
    }

    fn raw(base: Classes, one: Classes, strict: bool, length: usize) -> Options {
        Options {
            length,
            separator: String::new(),
            base,
            one,
            strict,
        }
    }

    /// Anzahl Zeichen je Klasse (lower, upper, digits, special)
    fn count(pw: &str) -> (usize, usize, usize, usize) {
        (
            pw.chars().filter(|c| LOWER.contains(*c)).count(),
            pw.chars().filter(|c| UPPER.contains(*c)).count(),
            pw.chars().filter(|c| DIGITS.contains(*c)).count(),
            pw.chars().filter(|c| SPECIAL.contains(*c)).count(),
        )
    }

    #[test]
    fn default_length_has_four_blocks() {
        let pw = generate_password(&Options::default()).unwrap();
        assert_eq!(pw.len(), 19);
        assert_eq!(pw.split('.').count(), 4);
        assert!(pw.split('.').all(|b| b.len() == BLOCK_SIZE));
    }

    #[test]
    fn odd_length_has_short_last_block() {
        let o = Options {
            length: 10,
            ..Options::default()
        };
        let pw = generate_password(&o).unwrap();
        let blocks: Vec<&str> = pw.split('.').collect();
        assert_eq!(blocks.len(), 3);
        assert_eq!(blocks[2].len(), 2);
    }

    #[test]
    fn custom_separator() {
        let o = Options {
            length: 8,
            separator: "--".to_string(),
            ..Options::default()
        };
        let pw = generate_password(&o).unwrap();
        assert_eq!(pw.len(), 10);
        assert_eq!(pw.split("--").count(), 2);
    }

    #[test]
    fn empty_separator_has_no_blocks() {
        let pw =
            generate_password(&raw(Classes::default(), Classes::default(), false, 12)).unwrap();
        assert_eq!(pw.len(), 12);
        assert!(pw.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn default_base_is_lower_upper_digits_each_at_least_once() {
        for _ in 0..300 {
            let pw =
                generate_password(&raw(Classes::default(), Classes::default(), false, 4)).unwrap();
            let (l, u, d, s) = count(&pw);
            assert!(l >= 1 && u >= 1 && d >= 1 && s == 0, "{pw}");
            assert_eq!(l + u + d, 4, "{pw}");
        }
    }

    #[test]
    fn excludes_ambiguous_and_yz() {
        let forbidden = "OoIlyzYZ";
        let o = raw(
            classes(true, true, true, true),
            Classes::default(),
            false,
            64,
        );
        for _ in 0..300 {
            let pw = generate_password(&o).unwrap();
            assert!(
                !pw.chars().any(|c| forbidden.contains(c)),
                "verbotenes Zeichen in {pw}"
            );
        }
    }

    #[test]
    fn single_classes() {
        let cases: [(Classes, &str); 4] = [
            (classes(true, false, false, false), LOWER),
            (classes(false, true, false, false), UPPER),
            (classes(false, false, true, false), DIGITS),
            (classes(false, false, false, true), SPECIAL),
        ];
        for (base, alphabet) in cases {
            let o = raw(base, Classes::default(), false, 16);
            for _ in 0..100 {
                let pw = generate_password(&o).unwrap();
                assert!(pw.chars().all(|c| alphabet.contains(c)), "{pw}");
            }
        }
    }

    #[test]
    fn combined_base_guarantees_each_class() {
        let o = raw(
            classes(true, false, true, false),
            Classes::default(),
            false,
            4,
        );
        for _ in 0..300 {
            let pw = generate_password(&o).unwrap();
            let (l, u, d, s) = count(&pw);
            assert!(l >= 1 && d >= 1, "{pw}");
            assert_eq!((u, s), (0, 0), "{pw}");
        }
    }

    #[test]
    fn digits_use_all_ten() {
        let o = raw(
            classes(false, false, true, false),
            Classes::default(),
            false,
            6,
        );
        let mut seen = std::collections::HashSet::new();
        for _ in 0..300 {
            seen.extend(generate_password(&o).unwrap().chars());
        }
        assert_eq!(seen.len(), 10, "{seen:?}");
    }

    #[test]
    fn one_extras_are_exactly_one() {
        let base = classes(true, false, false, false);
        let cases = [
            (classes(false, true, false, false), (15, 1, 0, 0)),
            (classes(false, false, true, false), (15, 0, 1, 0)),
            (classes(false, false, false, true), (15, 0, 0, 1)),
            (classes(false, true, true, true), (13, 1, 1, 1)),
        ];
        for (one, expected) in cases {
            let o = raw(base, one, false, 16);
            for _ in 0..100 {
                let pw = generate_password(&o).unwrap();
                assert_eq!(count(&pw), expected, "{pw}");
            }
        }
    }

    #[test]
    fn strict_adds_one_of_each_missing_class() {
        // Standard: nur Sonderzeichen fehlt
        let o = raw(Classes::default(), Classes::default(), true, 16);
        for _ in 0..100 {
            let pw = generate_password(&o).unwrap();
            let (l, u, d, s) = count(&pw);
            assert_eq!(s, 1, "{pw}");
            assert!(l >= 1 && u >= 1 && d >= 1, "{pw}");
        }
        // Nur Ziffern: drei Klassen fehlen
        let o = raw(
            classes(false, false, true, false),
            Classes::default(),
            true,
            4,
        );
        for _ in 0..100 {
            let pw = generate_password(&o).unwrap();
            assert_eq!(count(&pw), (1, 1, 1, 1), "{pw}");
        }
        // Alles im Grundvorrat: strict ohne Wirkung
        let o = raw(classes(true, true, true, true), Classes::default(), true, 4);
        for _ in 0..100 {
            let pw = generate_password(&o).unwrap();
            assert_eq!(count(&pw), (1, 1, 1, 1), "{pw}");
        }
    }

    #[test]
    fn special_never_uses_separator_char() {
        let o = Options {
            length: 64,
            separator: "_".to_string(),
            base: classes(false, false, false, true),
            ..Options::default()
        };
        for _ in 0..100 {
            let pw = generate_password(&o).unwrap();
            // 64 Zeichen in 16 Blöcken = 15 Trennzeichen, keines im Inhalt
            assert_eq!(pw.matches('_').count(), 15, "{pw}");
        }
    }

    #[test]
    fn one_in_base_is_rejected() {
        let o = raw(
            classes(true, false, true, false),
            classes(false, false, true, false),
            false,
            16,
        );
        assert_eq!(generate_password(&o), Err(Error::OneInBase(Class::Digits)));
        // auch gegen den impliziten Standard
        let o = raw(
            Classes::default(),
            classes(false, true, false, false),
            false,
            16,
        );
        assert_eq!(generate_password(&o), Err(Error::OneInBase(Class::Upper)));
    }

    #[test]
    fn rejects_invalid_length_and_separator() {
        for n in [0, MIN_LENGTH - 1, MAX_LENGTH + 1] {
            let o = Options {
                length: n,
                ..Options::default()
            };
            assert_eq!(generate_password(&o), Err(Error::InvalidLength(n)));
        }
        let o = Options {
            separator: "-".repeat(MAX_SEPARATOR_LEN + 1),
            ..Options::default()
        };
        assert_eq!(
            generate_password(&o),
            Err(Error::SeparatorTooLong(MAX_SEPARATOR_LEN + 1))
        );
    }
}
