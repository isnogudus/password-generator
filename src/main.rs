mod hash;
mod password;

use axum::{Router, extract::Query, extract::State, http::StatusCode, routing::get};
use clap::{Parser, Subcommand};
use hash::Algorithm;
use password::{DEFAULT_SEPARATOR, MAX_LENGTH, MIN_LENGTH, Options, generate_password};
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "password-generator")]
#[command(version = "0.1.0")]
#[command(about = "Generiert zufällige Passwörter (CLI oder Webserver)")]
struct Cli {
    /// Länge der Passwörter, Standard 12 bzw. 16 mit --lowercase
    /// (beim Server per ?length=N überschreibbar)
    #[arg(short, long, value_parser = parse_length, global = true)]
    length: Option<usize>,

    /// Trennzeichen zwischen den Viererblöcken (beim Server per ?separator=X überschreibbar)
    #[arg(short, long, default_value = DEFAULT_SEPARATOR, allow_hyphen_values = true, global = true)]
    separator: String,

    /// Keine Blöcke, Passwort am Stück ausgeben (entspricht --separator "")
    #[arg(long, conflicts_with = "separator", global = true)]
    no_separator: bool,

    /// Strict-Modus für strenge Passwortrichtlinien: Sonderzeichen hinzufügen
    /// und mindestens eines garantieren; mit --lowercase wie --upper --digit
    /// --special (beim Server per ?strict=1 überschreibbar)
    #[arg(short = 'x', long, global = true)]
    strict: bool,

    /// Kleinbuchstaben-Modus: nur Kleinbuchstaben, Standardlänge 16
    /// (beim Server per ?lowercase=1 überschreibbar)
    #[arg(short = 'w', long, global = true)]
    lowercase: bool,

    /// Genau ein Großbuchstabe, Rest klein (nur mit --lowercase; Server: ?upper=1)
    #[arg(long, requires = "lowercase", global = true)]
    upper: bool,

    /// Genau eine Ziffer, Rest klein (nur mit --lowercase; Server: ?digit=1)
    #[arg(long, requires = "lowercase", global = true)]
    digit: bool,

    /// Genau ein Sonderzeichen, Rest klein (nur mit --lowercase; Server: ?special=1)
    #[arg(long, requires = "lowercase", global = true)]
    special: bool,

    /// Zusätzlich einen Hash des Passworts ausgeben, durch Tabulator getrennt
    /// (beim Server per ?hash=ALGO überschreibbar)
    #[arg(long, value_enum, global = true)]
    hash: Option<Algorithm>,

    /// Anzahl der auszugebenden Passwörter (nur CLI)
    #[arg(short = 'n', long, default_value_t = 1)]
    count: usize,

    #[command(subcommand)]
    command: Option<Command>,
}

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
    /// Per CLI gesetzte Länge; None = modusabhängiger Standard
    length: Option<usize>,
    hash: Option<Algorithm>,
}

#[derive(Deserialize)]
struct Params {
    length: Option<usize>,
    separator: Option<String>,
    strict: Option<String>,
    lowercase: Option<String>,
    upper: Option<String>,
    digit: Option<String>,
    special: Option<String>,
    hash: Option<Algorithm>,
}

async fn password_handler(
    State(state): State<AppState>,
    Query(params): Query<Params>,
) -> Result<String, (StatusCode, String)> {
    let bad = |msg: String| (StatusCode::BAD_REQUEST, msg);
    let flag = |value: Option<String>, default: bool| -> Result<bool, (StatusCode, String)> {
        match value {
            Some(v) => parse_flag(&v).map_err(bad),
            None => Ok(default),
        }
    };

    let d = &state.defaults;
    let lowercase = flag(params.lowercase, d.lowercase)?;
    // Ohne Kleinbuchstaben-Modus gelten die Extras aus der Servervorgabe nicht;
    // explizit im Request gesetzte Extras werden von validate() abgewiesen.
    let extra = |server_default: bool| lowercase && server_default;
    let opts = Options {
        length: params
            .length
            .or(state.length)
            .unwrap_or_else(|| Options::default_length(lowercase)),
        separator: params.separator.unwrap_or_else(|| d.separator.clone()),
        strict: flag(params.strict, d.strict)?,
        lowercase,
        upper: flag(params.upper, extra(d.upper))?,
        digit: flag(params.digit, extra(d.digit))?,
        special: flag(params.special, extra(d.special))?,
    };
    // Hashen ist absichtlich langsam; nicht den Async-Worker blockieren.
    let hash = params.hash.or(state.hash);
    tokio::task::spawn_blocking(move || render(&opts, hash))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .map_err(bad)
}

async fn serve(
    host: &str,
    port: u16,
    defaults: Options,
    length: Option<usize>,
    hash: Option<Algorithm>,
) {
    let app = Router::new()
        .route("/", get(password_handler))
        .with_state(AppState {
            defaults,
            length,
            hash,
        });

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server läuft auf http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

fn main() {
    let cli = Cli::parse();

    let opts = Options {
        length: cli
            .length
            .unwrap_or_else(|| Options::default_length(cli.lowercase)),
        separator: if cli.no_separator {
            String::new()
        } else {
            cli.separator
        },
        strict: cli.strict,
        lowercase: cli.lowercase,
        upper: cli.upper,
        digit: cli.digit,
        special: cli.special,
    };
    if let Err(e) = password::validate(&opts) {
        eprintln!("error: {e}");
        std::process::exit(2);
    }

    match cli.command {
        Some(Command::Serve { host, port }) => {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(serve(&host, port, opts, cli.length, cli.hash));
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
