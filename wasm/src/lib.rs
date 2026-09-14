//! WebAssembly-Bindings für `password_generator::password`, gebaut mit
//! wasm-pack. Die Funktionen nehmen ein JavaScript-Objekt mit den Feldern
//! von [`Options`] (`length`, `separator`, `base`, `one`, `strict`; `base`
//! und `one` mit `lower`, `upper`, `digits`, `special`); fehlende Felder
//! nehmen die Standardwerte an. Ungültige Einstellungen werfen einen
//! JavaScript-`Error` mit derselben Meldung wie das CLI.

use password_generator::Options;
use wasm_bindgen::prelude::*;

/// Liest die Optionen aus einem JavaScript-Objekt; `undefined` = Standard
fn options(opts: JsValue) -> Result<Options, JsError> {
    if opts.is_undefined() || opts.is_null() {
        return Ok(Options::default());
    }
    serde_wasm_bindgen::from_value(opts).map_err(|e| JsError::new(&format!("invalid options: {e}")))
}

/// Erzeugt ein zufälliges Passwort, z.B. `aB3d.Ef7g.H9jk.Lm2n`
#[wasm_bindgen(js_name = generatePassword)]
pub fn generate_password(opts: JsValue) -> Result<String, JsError> {
    password_generator::generate_password(&options(opts)?).map_err(|e| JsError::new(&e.to_string()))
}

/// Prüft die Optionen; wirft bei ungültigen Einstellungen
#[wasm_bindgen]
pub fn validate(opts: JsValue) -> Result<(), JsError> {
    password_generator::validate(&options(opts)?).map_err(|e| JsError::new(&e.to_string()))
}

/// Grobe Entropie in Bit, gerundet
#[wasm_bindgen(js_name = entropyBits)]
pub fn entropy_bits(opts: JsValue) -> Result<u32, JsError> {
    Ok(password_generator::entropy_bits(&options(opts)?))
}
