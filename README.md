# password-generator

Generiert zufällige, gut abtippbare Passwörter — als CLI-Tool oder als kleiner Webserver in Rust. Gegenstück zu `password-rs` (merkbare Passphrasen).

## Überblick

Passwörter bestehen aus Groß-/Kleinbuchstaben und Ziffern, dargestellt in Viererblöcken, getrennt durch `.`.

**Beispiele:**
- `rCJS.2RCX.tAMG` (Standard: 12 Zeichen)
- `Vr3c.Jt6h.UAK7.6u3J` (16 Zeichen)

Der Punkt ist nur Darstellung: Die Passwortlänge zählt die Zeichen ohne Trennzeichen.

## Zeichenvorrat

Bewusst weggelassen:

- **Leicht verwechselbare Zeichen**: `0` / `O` / `o` und `1` / `l` / `I`
- **`y` / `z` / `Y` / `Z`**: auf QWERTY- und QWERTZ-Tastaturen vertauscht, so lässt sich das Passwort auf deutschen und englischen Tastaturen gleich tippen
- Keine Sonderzeichen (tastaturunabhängig)

Verwendet werden also:

```
abcdefghijkmnpqrstuvwx
ABCDEFGHJKLMNPQRSTUVWX
23456789
```

Jedes Passwort enthält garantiert mindestens einen Klein-, einen Großbuchstaben und eine Ziffer. Die Position der Pflichtzeichen wird zufällig gemischt.

## Installation

### Voraussetzungen

- Rust 1.85 oder höher (Edition 2024)
- Cargo

### Build

```bash
# Debug-Build
cargo build

# Optimierter Release-Build (empfohlen)
cargo build --release
```

Das Binary befindet sich dann in:
- Debug: `target/debug/password-generator`
- Release: `target/release/password-generator`

## Verwendung

### CLI

```bash
# Ein Passwort mit 12 Zeichen
password-generator

# Fünf Passwörter mit 16 Zeichen
password-generator -n 5 -l 16
```

### Webserver

```bash
# Mit Default-Einstellungen (127.0.0.1:3000, 12 Zeichen)
password-generator serve

# Eigenen Host, Port und Standardlänge angeben
password-generator serve --host 0.0.0.0 --port 8080 --length 16
```

Sobald der Server läuft:

```bash
# Im Browser
http://127.0.0.1:3000

# Mit curl
curl http://127.0.0.1:3000

# Andere Länge pro Request
curl 'http://127.0.0.1:3000/?length=20'
```

Jeder Request generiert ein neues, zufälliges Passwort. Ungültige Längen (erlaubt: 4 bis 128) beantwortet der Server mit `400 Bad Request`.

### CLI-Optionen

```
Generiert zufällige Passwörter (CLI oder Webserver)

Usage: password-generator [OPTIONS] [COMMAND]

Commands:
  serve  Webserver starten, der pro Request ein Passwort liefert
  help   Print this message or the help of the given subcommand(s)

Options:
  -l, --length <LENGTH>  Länge der Passwörter (beim Server per ?length=N überschreibbar) [default: 12]
  -n, --count <COUNT>    Anzahl der auszugebenden Passwörter (nur CLI) [default: 1]
  -h, --help             Print help
  -V, --version          Print version
```

```
Usage: password-generator serve [OPTIONS]

Options:
  -H, --host <HOST>      Host-Adresse, auf der der Server lauscht [default: 127.0.0.1]
  -l, --length <LENGTH>  Länge der Passwörter (beim Server per ?length=N überschreibbar) [default: 12]
  -p, --port <PORT>      Port, auf dem der Server lauscht [default: 3000]
```

## Hinter Caddy betreiben

Caddy kann von Haus aus kein Programm pro Request starten (kein CGI im Kern; dafür wäre ein Custom-Build mit dem Plugin `caddy-cgi` nötig). Der übliche Weg ist daher der `serve`-Modus hinter `reverse_proxy`:

```
pw.example.org {
    reverse_proxy 127.0.0.1:3000
}
```

## Projektstruktur

```
password-generator/
├── src/
│   ├── main.rs        # CLI, Subcommand "serve", Webserver
│   └── password.rs    # Zeichenvorrat und Passwort-Generierung
├── Cargo.toml
├── Dockerfile
├── compose.yml.example
└── README.md
```

## Technische Details

### Performance

Die Release-Build-Konfiguration optimiert für:
- **Größe**: `opt-level = "z"` minimiert die Binary-Größe
- **Link-Time Optimization**: Verbessert die Performance
- **Stripped Binary**: Entfernt Debug-Symbole

### Sicherheit

- Verwendet `rand::thread_rng()` (ChaCha, aus dem Betriebssystem geseedet) für kryptographisch sichere Zufallszahlen
- Keine persistenten Daten oder Logs
- Jeder Request ist unabhängig

## Entwicklung

```bash
cargo test     # Tests ausführen
cargo fmt      # Code formatieren
cargo clippy   # Linter ausführen
```

## Abhängigkeiten

- **axum** - Moderner Web-Framework
- **tokio** - Async Runtime
- **clap** - CLI Argument Parser
- **rand** - Zufallszahlengenerator
- **serde** - Deserialisierung der Query-Parameter

## Lizenz

[Ihre Lizenz hier einfügen]

## Autor

[Ihr Name hier einfügen]
