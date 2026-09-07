use clap::ValueEnum;
use serde::Deserialize;

/// Unterstützte Hash-Verfahren
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Algorithm {
    /// bcrypt (`$2b$`), z.B. für Caddy basic_auth, htpasswd, OpenBSD passwd
    Bcrypt,
    /// SHA-512-crypt (`$6$`), z.B. für /etc/shadow unter Linux
    Sha512Crypt,
    /// Argon2id (PHC-String), modernes Verfahren für eigene Anwendungen
    Argon2id,
}

impl Algorithm {
    /// Erzeugt einen salted Hash des Passworts im jeweiligen Standardformat
    pub fn hash(self, password: &str) -> Result<String, String> {
        match self {
            Algorithm::Bcrypt => {
                bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| e.to_string())
            }
            Algorithm::Sha512Crypt => {
                use sha_crypt::{PasswordHasher, ShaCrypt};
                ShaCrypt::default()
                    .hash_password(password.as_bytes())
                    .map(|h| h.to_string())
                    .map_err(|e| e.to_string())
            }
            Algorithm::Argon2id => {
                use argon2::{Argon2, PasswordHasher};
                Argon2::default()
                    .hash_password(password.as_bytes())
                    .map(|h| h.to_string())
                    .map_err(|e| e.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PW: &str = "gEv9.5Xed.P73G";

    #[test]
    fn bcrypt_roundtrip() {
        let h = Algorithm::Bcrypt.hash(PW).unwrap();
        assert!(h.starts_with("$2b$12$"), "{h}");
        assert!(bcrypt::verify(PW, &h).unwrap());
        assert!(!bcrypt::verify("falsch", &h).unwrap());
    }

    #[test]
    fn sha512_crypt_roundtrip() {
        use sha_crypt::{PasswordHash, PasswordVerifier, ShaCrypt};
        let h = Algorithm::Sha512Crypt.hash(PW).unwrap();
        assert!(h.starts_with("$6$"), "{h}");
        let parsed = PasswordHash::new(&h).unwrap();
        assert!(
            ShaCrypt::default()
                .verify_password(PW.as_bytes(), &parsed)
                .is_ok()
        );
        assert!(
            ShaCrypt::default()
                .verify_password(b"falsch", &parsed)
                .is_err()
        );
    }

    #[test]
    fn argon2id_roundtrip() {
        use argon2::{Argon2, PasswordHash, PasswordVerifier};
        let h = Algorithm::Argon2id.hash(PW).unwrap();
        assert!(h.starts_with("$argon2id$v=19$"), "{h}");
        let parsed = PasswordHash::new(&h).unwrap();
        assert!(
            Argon2::default()
                .verify_password(PW.as_bytes(), &parsed)
                .is_ok()
        );
        assert!(
            Argon2::default()
                .verify_password(b"falsch", &parsed)
                .is_err()
        );
    }

    #[test]
    fn salted_hashes_differ() {
        assert_ne!(
            Algorithm::Bcrypt.hash(PW).unwrap(),
            Algorithm::Bcrypt.hash(PW).unwrap()
        );
    }
}
