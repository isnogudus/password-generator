mod hash;
mod password;

use axum::{Router, extract::Query, extract::State, http::StatusCode, routing::get};
use clap::{Parser, Subcommand};
use hash::Algorithm;
use password::{
    DEFAULT_LENGTH, DEFAULT_SEPARATOR, MAX_LENGTH, MIN_LENGTH, Options, generate_password,
};
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "password-generator")]
#[command(version = "0.1.0")]
#[command(about = "Generiert zufällige Passwörter (CLI oder Webserver)")]
struct Cli {
    /// Länge der Passwörter (beim Server per ?length=N überschreibbar)
    #[arg(short, long, default_value_t = DEFAULT_LENGTH, value_parser = parse_length, global = true)]
    length: usize,

    /// Trennzeichen zwischen den Viererblöcken (beim Server per ?separator=X überschreibbar)
    #[arg(short, long, default_value = DEFAULT_SEPARATOR, allow_hyphen_values = true, global = true)]
    separator: String,

    /// Keine Blöcke, Passwort am Stück ausgeben (entspricht --separator "")
    #[arg(long, conflicts_with = "separator", global = true)]
    no_separator: bool,

    /// Strict-Modus: Sonderzeichen hinzufügen und mindestens eines garantieren,
    /// für strenge Passwortrichtlinien (beim Server per ?strict=1 überschreibbar)
    #[arg(short = 'x', long, global = true)]
    strict: bool,

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
    hash: Option<Algorithm>,
}

#[derive(Deserialize)]
struct Params {
    length: Option<usize>,
    separator: Option<String>,
    strict: Option<String>,
    hash: Option<Algorithm>,
}

async fn password_handler(
    State(state): State<AppState>,
    Query(params): Query<Params>,
) -> Result<String, (StatusCode, String)> {
    let bad = |msg: String| (StatusCode::BAD_REQUEST, msg);

    let strict = match params.strict {
        Some(v) => parse_flag(&v).map_err(bad)?,
        None => state.defaults.strict,
    };
    let opts = Options {
        length: params.length.unwrap_or(state.defaults.length),
        separator: params.separator.unwrap_or(state.defaults.separator),
        strict,
    };
    // Hashen ist absichtlich langsam; nicht den Async-Worker blockieren.
    let hash = params.hash.or(state.hash);
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
