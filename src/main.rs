mod password;

use axum::{Router, extract::Query, extract::State, http::StatusCode, routing::get};
use clap::{Parser, Subcommand};
use password::{DEFAULT_LENGTH, MAX_LENGTH, MIN_LENGTH, generate_password};
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "password-generator")]
#[command(version = "0.1.0")]
#[command(about = "Generiert zufällige Passwörter (CLI oder Webserver)")]
struct Cli {
    /// Länge der Passwörter (beim Server per ?length=N überschreibbar)
    #[arg(short, long, default_value_t = DEFAULT_LENGTH, value_parser = parse_length, global = true)]
    length: usize,

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

#[derive(Clone)]
struct AppState {
    default_length: usize,
}

#[derive(Deserialize)]
struct Params {
    length: Option<usize>,
}

async fn password_handler(
    State(state): State<AppState>,
    Query(params): Query<Params>,
) -> Result<String, (StatusCode, String)> {
    let length = params.length.unwrap_or(state.default_length);
    generate_password(length).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))
}

async fn serve(host: &str, port: u16, default_length: usize) {
    let app = Router::new()
        .route("/", get(password_handler))
        .with_state(AppState { default_length });

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Server läuft auf http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Serve { host, port }) => {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(serve(&host, port, cli.length));
        }
        None => {
            for _ in 0..cli.count {
                // Länge ist durch parse_length bereits validiert
                println!("{}", generate_password(cli.length).unwrap());
            }
        }
    }
}
