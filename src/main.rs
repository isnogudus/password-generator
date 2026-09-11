mod hash;
mod password;

use axum::{Router, extract::Query, extract::State, http::StatusCode, routing::get};
use clap::{Parser, Subcommand, ValueEnum};
use hash::Algorithm;
use password::{
    Class, Classes, DEFAULT_LENGTH, DEFAULT_SEPARATOR, MAX_LENGTH, MIN_LENGTH, Options,
    generate_password,
};
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "password-generator")]
#[command(version = "0.1.0")]
#[command(about = "Generiert zufällige Passwörter (CLI oder Webserver)")]
struct Cli {
    /// Kleinbuchstaben in den Grundvorrat (abcdefghijkmnpqrstuvwx)
    #[arg(short = 'w', long, global = true, help_heading = BASE_HEADING)]
    lower: bool,

    /// Großbuchstaben in den Grundvorrat (ABCDEFGHJKLMNPQRSTUVWX)
    #[arg(short = 'u', long, global = true, help_heading = BASE_HEADING)]
    upper: bool,

    /// Ziffern in den Grundvorrat (0123456789)
    #[arg(short = 'd', long, global = true, help_heading = BASE_HEADING)]
    digits: bool,

    /// Sonderzeichen in den Grundvorrat (!#$%&*+=?@_)
    #[arg(short = 'x', long, global = true, help_heading = BASE_HEADING)]
    special: bool,

    /// Genau ein Kleinbuchstabe
    #[arg(long, global = true, help_heading = ONE_HEADING)]
    one_lower: bool,

    /// Genau ein Großbuchstabe
    #[arg(long, global = true, help_heading = ONE_HEADING)]
    one_upper: bool,

    /// Genau eine Ziffer
    #[arg(long, global = true, help_heading = ONE_HEADING)]
    one_digit: bool,

    /// Genau ein Sonderzeichen
    #[arg(long, global = true, help_heading = ONE_HEADING)]
    one_special: bool,

    /// Genau ein Zeichen aus jeder Klasse, die nicht im Grundvorrat ist
    #[arg(long, global = true, help_heading = ONE_HEADING)]
    strict: bool,

    /// Länge ohne Trennzeichen (beim Server per ?length=N überschreibbar)
    #[arg(short, long, default_value_t = DEFAULT_LENGTH, value_parser = parse_length, global = true)]
    length: usize,

    /// Trennzeichen zwischen den Viererblöcken (beim Server per ?separator=X überschreibbar)
    #[arg(short, long, default_value = DEFAULT_SEPARATOR, allow_hyphen_values = true, global = true)]
    separator: String,

    /// Passwort am Stück ausgeben (entspricht --separator "")
    #[arg(long, conflicts_with = "separator", global = true)]
    no_separator: bool,

    /// Hash mit ausgeben, durch Tabulator getrennt; --hash allein bedeutet
    /// argon2id, sonst --hash=ALGO (beim Server per ?hash oder ?hash=ALGO)
    #[arg(long, value_enum, num_args = 0..=1, require_equals = true, default_missing_value = "argon2id", global = true)]
    hash: Option<Algorithm>,

    /// Anzahl der auszugebenden Passwörter (nur CLI)
    #[arg(short = 'n', long, default_value_t = 1)]
    count: usize,

    #[command(subcommand)]
    command: Option<Command>,
}

const BASE_HEADING: &str =
    "Grundvorrat (kombinierbar, z.B. -wd; ohne Angabe -wud; jede Klasse mindestens einmal)";
const ONE_HEADING: &str = "Extras (genau ein Zeichen aus einer Klasse außerhalb des Grundvorrats)";

#[derive(Subcommand)]
enum Command {
    /// Webserver starten, der pro Request ein Passwort liefert
    Serve {
        /// Host-Adresse, auf der der Server lauscht
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,

        /// Port, auf dem der Server lauscht
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

/// Prüft das CLI-Argument --length auf den erlaubten Bereich
fn parse_length(s: &str) -> Result<usize, String> {
    let n: usize = s.parse().map_err(|_| format!("'{s}' ist keine Zahl"))?;
    if (MIN_LENGTH..=MAX_LENGTH).contains(&n) {
        Ok(n)
    } else {
        Err(format!("erlaubt sind {MIN_LENGTH} bis {MAX_LENGTH}"))
    }
}

/// Interpretiert Query-Werte wie `1`, `true`, `yes` oder leer (`?strict`) als wahr
fn parse_flag(value: &str) -> Result<bool, String> {
    match value.to_ascii_lowercase().as_str() {
        "" | "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        other => Err(format!("Ungültiger Wahrheitswert '{other}'")),
    }
}

/// Erzeugt ein Passwort und hängt bei Bedarf den Hash an: `<passwort>\t<hash>`
fn render(opts: &Options, hash: Option<Algorithm>) -> Result<String, String> {
    let password = generate_password(opts).map_err(|e| e.to_string())?;
    match hash {
        Some(algo) => Ok(format!("{password}\t{}", algo.hash(&password)?)),
        None => Ok(password),
    }
}

#[derive(Clone)]
struct AppState {
    defaults: Options,
    hash: Option<Algorithm>,
}

#[derive(Deserialize)]
struct Params {
    length: Option<usize>,
    separator: Option<String>,
    lower: Option<String>,
    upper: Option<String>,
    digits: Option<String>,
    special: Option<String>,
    #[serde(rename = "one-lower")]
    one_lower: Option<String>,
    #[serde(rename = "one-upper")]
    one_upper: Option<String>,
    #[serde(rename = "one-digit")]
    one_digit: Option<String>,
    #[serde(rename = "one-special")]
    one_special: Option<String>,
    strict: Option<String>,
    hash: Option<String>,
}

type Rejection = (StatusCode, String);

/// Wendet die Klassen-Schalter eines Requests auf eine Vorgabe an
fn apply_classes(
    mut classes: Classes,
    params: [(Class, Option<String>); 4],
) -> Result<Classes, Rejection> {
    for (class, value) in params {
        if let Some(v) = value {
            classes.set(
                class,
                parse_flag(&v).map_err(|e| (StatusCode::BAD_REQUEST, e))?,
            );
        }
    }
    Ok(classes)
}

async fn password_handler(
    State(state): State<AppState>,
    Query(params): Query<Params>,
) -> Result<String, Rejection> {
    let bad = |msg: String| (StatusCode::BAD_REQUEST, msg);
    let d = &state.defaults;

    // Auf dem effektiven Vorrat aufsetzen, damit ?special=1 den Standard
    // ergänzt statt ihn zu ersetzen.
    let base = apply_classes(
        d.effective_base(),
        [
            (Class::Lower, params.lower),
            (Class::Upper, params.upper),
            (Class::Digits, params.digits),
            (Class::Special, params.special),
        ],
    )?;
    // Extras aus der Servervorgabe, die durch den Request im Grundvorrat
    // gelandet sind, fallen stillschweigend weg; explizit angeforderte
    // Extras im Grundvorrat weist validate() ab.
    let effective_base = if base.is_empty() {
        Classes::DEFAULT
    } else {
        base
    };
    let mut server_one = d.one;
    for class in Class::ALL {
        if effective_base.contains(class) {
            server_one.set(class, false);
        }
    }
    let one = apply_classes(
        server_one,
        [
            (Class::Lower, params.one_lower),
            (Class::Upper, params.one_upper),
            (Class::Digits, params.one_digit),
            (Class::Special, params.one_special),
        ],
    )?;
    let strict = match params.strict {
        Some(v) => parse_flag(&v).map_err(bad)?,
        None => d.strict,
    };
    let hash = match params.hash.as_deref() {
        None => state.hash,
        Some("") => Some(Algorithm::Argon2id),
        Some(name) => Some(Algorithm::from_str(name, true).map_err(bad)?),
    };

    let opts = Options {
        length: params.length.unwrap_or(d.length),
        separator: params.separator.unwrap_or_else(|| d.separator.clone()),
        base,
        one,
        strict,
    };
    // Hashen ist absichtlich langsam; nicht den Async-Worker blockieren.
    tokio::task::spawn_blocking(move || render(&opts, hash))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(bad)
}

async fn serve(host: &str, port: u16, defaults: Options, hash: Option<Algorithm>) {
    let app = Router::new()
        .route("/", get(password_handler))
        .with_state(AppState { defaults, hash });

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server läuft auf http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

fn main() {
    let cli = Cli::parse();

    let opts = Options {
        length: cli.length,
        separator: if cli.no_separator {
            String::new()
        } else {
            cli.separator
        },
        base: Classes {
            lower: cli.lower,
            upper: cli.upper,
            digits: cli.digits,
            special: cli.special,
        },
        one: Classes {
            lower: cli.one_lower,
            upper: cli.one_upper,
            digits: cli.one_digit,
            special: cli.one_special,
        },
        strict: cli.strict,
    };
    if let Err(e) = password::validate(&opts) {
        eprintln!("error: {e}");
        std::process::exit(2);
    }

    match cli.command {
        Some(Command::Serve { host, port }) => {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(serve(&host, port, opts, cli.hash));
        }
        None => {
            for _ in 0..cli.count {
                match render(&opts, cli.hash) {
                    Ok(line) => println!("{line}"),
                    Err(e) => {
                        eprintln!("error: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}
