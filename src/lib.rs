//! Zufällige, gut abtippbare Passwörter aus kombinierbaren Zeichenklassen,
//! dargestellt in Viererblöcken, plus Hash-Verfahren für gängige Passwortdateien.
//!
//! ```
//! use password_generator::password::{Classes, Options, generate_password};
//!
//! // Standard: 16 Zeichen aus Klein-, Großbuchstaben und Ziffern, z.B. `x8GG.JpJN.LN40.t7qx`
//! let pw = generate_password(&Options::default()).unwrap();
//! assert_eq!(pw.len(), 19);
//!
//! // Nur Ziffern, ohne Trennzeichen: eine PIN
//! let pin = generate_password(&Options {
//!     length: 6,
//!     separator: String::new(),
//!     base: Classes { digits: true, ..Classes::default() },
//!     ..Options::default()
//! })
//! .unwrap();
//! assert!(pin.chars().all(|c| c.is_ascii_digit()));
//! ```
//!
//! Features: `hash` bringt das Modul [`hash`] mit argon2id, bcrypt und
//! sha512-crypt; `bin` (Standard, schließt `hash` ein) zieht die
//! Abhängigkeiten von CLI und Webserver hinein. Als Bibliothek reicht
//! `default-features = false`, gegebenenfalls plus `hash`. Für den Browser
//! gibt es die Bindings im Crate `wasm/`.

#[cfg(feature = "hash")]
pub mod hash;
pub mod password;

#[cfg(feature = "hash")]
pub use hash::Algorithm;
pub use password::{Class, Classes, Error, Options, entropy_bits, generate_password, validate};
