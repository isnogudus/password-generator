mod hash;
mod password;

use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
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
#[command(about = "Generates random, easy-to-type passwords (CLI or web server)")]
struct Cli {
    /// Lowercase letters in the base set (abcdefghijkmnpqrstuvwx)
    #[arg(short = 'w', long, global = true, help_heading = BASE_HEADING)]
    lower: bool,

    /// Uppercase letters in the base set (ABCDEFGHJKLMNPQRSTUVWX)
    #[arg(short = 'u', long, global = true, help_heading = BASE_HEADING)]
    upper: bool,

    /// Digits in the base set (0123456789)
    #[arg(short = 'd', long, global = true, help_heading = BASE_HEADING)]
    digits: bool,

    /// Special characters in the base set (!#$%&*+=?@_)
    #[arg(short = 'x', long, global = true, help_heading = BASE_HEADING)]
    special: bool,

    /// Exactly one lowercase letter
    #[arg(long, global = true, help_heading = ONE_HEADING)]
    one_lower: bool,

    /// Exactly one uppercase letter
    #[arg(long, global = true, help_heading = ONE_HEADING)]
    one_upper: bool,

    /// Exactly one digit
    #[arg(long, global = true, help_heading = ONE_HEADING)]
    one_digit: bool,

    /// Exactly one special character
    #[arg(long, global = true, help_heading = ONE_HEADING)]
    one_special: bool,

    /// Exactly one character of every class not in the base set
    #[arg(long, global = true, help_heading = ONE_HEADING)]
    strict: bool,

    /// Length without separators (server: ?length=N)
    #[arg(short, long, default_value_t = DEFAULT_LENGTH, value_parser = parse_length, global = true)]
    length: usize,

    /// Separator between blocks of four (server: ?separator=X)
    #[arg(short, long, default_value = DEFAULT_SEPARATOR, allow_hyphen_values = true, global = true)]
    separator: String,

    /// No blocks, print the password as one piece (same as --separator "")
    #[arg(long, conflicts_with = "separator", global = true)]
    no_separator: bool,

    /// Also print a hash, tab-separated; --hash alone means argon2id,
    /// otherwise --hash=ALGO (server: ?hash or ?hash=ALGO)
    #[arg(long, value_enum, num_args = 0..=1, require_equals = true, default_missing_value = "argon2id", global = true)]
    hash: Option<Algorithm>,

    /// Number of passwords to print (CLI only)
    #[arg(short = 'n', long, default_value_t = 1)]
    count: usize,

    #[command(subcommand)]
    command: Option<Command>,
}

const BASE_HEADING: &str =
    "Base set (combinable, e.g. -wd; default -wud; every chosen class appears at least once)";
const ONE_HEADING: &str = "Extras (exactly one character of a class outside the base set)";

#[derive(Subcommand)]
enum Command {
    /// Start the web server, one password per request
    Serve {
        /// Host address to listen on
        #[arg(short = 'H', long, default_value = "127.0.0.1")]
        host: String,

        /// Port to listen on
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
    },
}

/// Prüft das CLI-Argument --length auf den erlaubten Bereich
fn parse_length(s: &str) -> Result<usize, String> {
    let n: usize = s.parse().map_err(|_| format!("'{s}' is not a number"))?;
    if (MIN_LENGTH..=MAX_LENGTH).contains(&n) {
        Ok(n)
    } else {
        Err(format!("allowed range is {MIN_LENGTH} to {MAX_LENGTH}"))
    }
}

/// Interpretiert Query-Werte wie `1`, `true`, `yes` oder leer (`?strict`) als wahr
fn parse_flag(value: &str) -> Result<bool, String> {
    match value.to_ascii_lowercase().as_str() {
        "" | "1" | "true" | "yes" | "on" => Ok(true),
        "0" | "false" | "no" | "off" => Ok(false),
        other => Err(format!("invalid boolean value '{other}'")),
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

/// Die Web-Oberfläche, mit den Servervorgaben als JSON eingebettet
const INDEX_HTML: &str = include_str!("../static/index.html");

fn index_page(state: &AppState) -> String {
    let d = &state.defaults;
    let base = d.effective_base();
    let classes = |c: Classes| {
        serde_json::json!({
            "lower": c.lower, "upper": c.upper, "digits": c.digits, "special": c.special
        })
    };
    let defaults = serde_json::json!({
        "length": d.length,
        "separator": d.separator,
        "base": classes(base),
        "one": classes(d.one),
        "strict": d.strict,
        "hash": state.hash.map(|h| h.to_possible_value().map(|v| v.get_name().to_string())),
    });
    INDEX_HTML.replace("__DEFAULTS__", &defaults.to_string())
}

/// Browser (Accept: text/html) bekommen die Oberfläche, alle anderen den reinen Text
fn wants_html(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|accept| accept.contains("text/html"))
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

async fn root_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    query: Result<Query<Params>, axum::extract::rejection::QueryRejection>,
) -> Response {
    if wants_html(&headers) {
        return Html(index_page(&state)).into_response();
    }
    let params = match query {
        Ok(Query(p)) => p,
        Err(e) => return (StatusCode::BAD_REQUEST, e.body_text()).into_response(),
    };
    match password_handler(state, params).await {
        Ok(text) => text.into_response(),
        Err(rejection) => rejection.into_response(),
    }
}

async fn password_handler(state: AppState, params: Params) -> Result<String, Rejection> {
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
        .route("/", get(root_handler))
        .with_state(AppState { defaults, hash });

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    println!("Listening on http://{}", addr);

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
