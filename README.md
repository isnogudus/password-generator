# password-generator

Generiert zufällige, gut abtippbare Passwörter — als CLI-Tool oder als kleiner Webserver in Rust. Gegenstück zu `password-rs` (merkbare Passphrasen).

## Überblick

Passwörter werden aus frei kombinierbaren Zeichenklassen zusammengesetzt und in Viererblöcken dargestellt, getrennt durch `.`. Standard sind 16 Zeichen aus Klein-, Großbuchstaben und Ziffern.

```
x8GG.JpJN.LN40.t7qx      Standard (-wud)
ppfw.htkm.jdca.fapn      -w        nur Kleinbuchstaben
669a.h3e3.rjjm.wv7s      -wd       Klein + Ziffern
193886                   -d -l 6 --no-separator   PIN
tR_#.pSAv.0WGC.x?7c      -wudx     alles inklusive Sonderzeichen
hqtc.ahfg.raxn.Xk1j      -w --one-upper --one-digit   Apple-Stil
48q7.+v9M.mX0P.Vct3      --strict  Standard plus genau ein Sonderzeichen
```

Der Punkt ist nur Darstellung: Die Passwortlänge zählt die Zeichen ohne Trennzeichen. Das Trennzeichen ist frei wählbar (`--separator`) oder lässt sich weglassen (`--no-separator`).

## Zeichenklassen

| Schalter        | Klasse          | Zeichen                  |
|-----------------|-----------------|--------------------------|
| `-w, --lower`   | Kleinbuchstaben | `abcdefghijkmnpqrstuvwx` |
| `-u, --upper`   | Großbuchstaben  | `ABCDEFGHJKLMNPQRSTUVWX` |
| `-d, --digits`  | Ziffern         | `0123456789`             |
| `-x, --special` | Sonderzeichen   | `!#$%&*+=?@_`            |

Die Schalter lassen sich kombinieren, auch zusammengezogen wie `-wd` oder `-wudx`. Ohne Angabe gilt `-wud`. **Von jeder gewählten Klasse ist mindestens ein Zeichen enthalten**, die übrigen Zeichen kommen zufällig aus dem Gesamtvorrat der gewählten Klassen.

Bewusst weggelassen:

- **Leicht verwechselbare Buchstaben**: `l`, `o`, `I`, `O`. Weil sie in keinem Vorrat vorkommen, bleiben die Ziffern `0` und `1` unverwechselbar und sind enthalten.
- **`y` / `z` / `Y` / `Z`**: auf QWERTY- und QWERTZ-Tastaturen vertauscht, so lässt sich das Passwort auf deutschen und englischen Tastaturen gleich tippen.
- **Problematische Sonderzeichen**: Anführungszeichen, Backslash und Backtick (Quoting-Fallen) sowie `|` (verwechselbar mit `l`/`I`). Zeichen, die im Trennzeichen vorkommen, werden zusätzlich aus den Sonderzeichen entfernt.

### Extras

Für Passwortabfragen, die eine Klasse zwingend verlangen, ohne dass sie das ganze Passwort prägen soll:

| Option          | Wirkung                                                         |
|-----------------|-----------------------------------------------------------------|
| `--one-lower`   | genau ein Kleinbuchstabe                                        |
| `--one-upper`   | genau ein Großbuchstabe                                         |
| `--one-digit`   | genau eine Ziffer                                               |
| `--one-special` | genau ein Sonderzeichen                                         |
| `--strict`      | genau ein Zeichen aus jeder Klasse, die nicht im Grundvorrat ist |

Extras gelten nur für Klassen außerhalb des Grundvorrats. `-wd --one-digit` ist ein Fehler, weil Ziffern bereits enthalten sind. `--strict` ist bei `-wudx` ohne Wirkung.

### Entropie

| Aufruf              | Zeichen | Vorrat | Entropie |
|---------------------|---------|--------|----------|
| Standard (`-wud`)   | 16      | 54     | ~92 Bit  |
| `-wudx`             | 16      | 65     | ~96 Bit  |
| `-wd`               | 16      | 32     | ~80 Bit  |
| `-w`                | 16      | 22     | ~71 Bit  |
| `-w`, 20 Zeichen    | 20      | 22     | ~89 Bit  |
| `-d`, PIN           | 6       | 10     | ~20 Bit  |

Die Extras ändern die Entropie kaum. Alles über 70 Bit ist gegen Online-Angriffe wie gegen Offline-Angriffe auf bcrypt oder Argon2 mehr als ausreichend; der Hebel ist die Länge, ein Block mehr bringt 18 bis 23 Bit. Eine PIN ist nur zusammen mit einer Versuchsbegrenzung sicher, wie sie Geräte und Karten mitbringen.

## Hashes

Mit `--hash` wird zu jedem Passwort der passende Hash ausgegeben, getrennt durch einen Tabulator. `--hash` allein bedeutet Argon2id, andere Verfahren mit `--hash=ALGO`.

| `--hash=`      | Format               | Typischer Einsatz                                     |
|----------------|----------------------|-------------------------------------------------------|
| `argon2id`     | `$argon2id$v=19$…`   | Standard; eigene Anwendungen, PHC-String              |
| `bcrypt`       | `$2b$12$…`           | Caddy `basic_auth`, htpasswd, OpenBSD `passwd`, Gitea |
| `sha512-crypt` | `$6$rounds=5000$…`   | `/etc/shadow` unter Linux, `chpasswd -e`              |

Jeder Hash bekommt ein frisches zufälliges Salt. Die Ausgabezeile ist `<passwort>\t<hash>`, so lässt sich mit `cut -f2` der Hash allein herausziehen:

```bash
password-generator --hash=bcrypt | cut -f2
```

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
# Ein Passwort mit 16 Zeichen aus Klein-, Großbuchstaben und Ziffern
password-generator

# Fünf Passwörter mit 20 Zeichen
password-generator -n 5 -l 20

# Anderes Trennzeichen bzw. keines
password-generator -s -
password-generator --no-separator

# Zeichenklassen wählen
password-generator -w                      # nur Kleinbuchstaben
password-generator -wd                     # Kleinbuchstaben und Ziffern
password-generator -wudx                   # alles, Sonderzeichen im ganzen Vorrat
password-generator -d -l 6 --no-separator  # sechsstellige PIN

# Extras für strenge Richtlinien
password-generator -w --one-upper --one-digit
password-generator --strict                # Standard plus genau ein Sonderzeichen

# Passwort und Hash
password-generator --hash                  # argon2id
password-generator --hash=bcrypt
```

### Webserver

```bash
# Mit Default-Einstellungen (127.0.0.1:3000, 16 Zeichen, -wud)
password-generator serve

# Eigenen Host, Port und Vorgaben angeben
password-generator serve --host 0.0.0.0 --port 8080 -wd --length 20
```

Sobald der Server läuft:

```bash
# Im Browser
http://127.0.0.1:3000

# Mit curl
curl http://127.0.0.1:3000

# Optionen pro Request
curl 'http://127.0.0.1:3000/?length=20'
curl 'http://127.0.0.1:3000/?separator=-'
curl 'http://127.0.0.1:3000/?separator='                     # ohne Trennzeichen
curl 'http://127.0.0.1:3000/?upper=0'                        # -wd
curl 'http://127.0.0.1:3000/?special=1'                      # -wudx
curl 'http://127.0.0.1:3000/?lower=0&upper=0&length=6&separator='   # PIN
curl 'http://127.0.0.1:3000/?strict'
curl 'http://127.0.0.1:3000/?one-special=1'
curl 'http://127.0.0.1:3000/?hash'                           # Passwort<TAB>Argon2id-Hash
curl 'http://127.0.0.1:3000/?hash=bcrypt'
```

Die Query-Parameter heißen wie die Langformen der Optionen: `lower`, `upper`, `digits`, `special` schalten Klassen relativ zur Servervorgabe ein oder aus, `one-lower`, `one-upper`, `one-digit`, `one-special`, `strict`, `length`, `separator` und `hash` wie im CLI. Für die Schalter gelten `1`, `true`, `yes`, `on` oder ein leerer Wert (`?strict`) als wahr. Werden alle Klassen abgeschaltet, gilt der Standard `-wud`. Jeder Request generiert ein neues, zufälliges Passwort. Ungültige Werte (Länge außerhalb 4 bis 128, Trennzeichen länger als 8 Zeichen, Extra für eine Klasse im Grundvorrat) beantwortet der Server mit `400 Bad Request`.

### CLI-Optionen

```
Generiert zufällige Passwörter (CLI oder Webserver)

Usage: password-generator [OPTIONS] [COMMAND]

Commands:
  serve  Webserver starten, der pro Request ein Passwort liefert
  help   Print this message or the help of the given subcommand(s)

Options:
  -l, --length <LENGTH>        Länge ohne Trennzeichen (beim Server per ?length=N überschreibbar) [default: 16]
  -s, --separator <SEPARATOR>  Trennzeichen zwischen den Viererblöcken (beim Server per ?separator=X überschreibbar) [default: .]
      --no-separator           Passwort am Stück ausgeben (entspricht --separator "")
      --hash[=<HASH>]          Hash mit ausgeben, durch Tabulator getrennt; --hash allein bedeutet argon2id, sonst --hash=ALGO (beim Server per ?hash oder ?hash=ALGO) [possible values: bcrypt, sha512-crypt, argon2id]
  -n, --count <COUNT>          Anzahl der auszugebenden Passwörter (nur CLI) [default: 1]
  -h, --help                   Print help
  -V, --version                Print version

Grundvorrat (kombinierbar, z.B. -wd; ohne Angabe -wud; jede Klasse mindestens einmal):
  -w, --lower    Kleinbuchstaben in den Grundvorrat (abcdefghijkmnpqrstuvwx)
  -u, --upper    Großbuchstaben in den Grundvorrat (ABCDEFGHJKLMNPQRSTUVWX)
  -d, --digits   Ziffern in den Grundvorrat (0123456789)
  -x, --special  Sonderzeichen in den Grundvorrat (!#$%&*+=?@_)

Extras (genau ein Zeichen aus einer Klasse außerhalb des Grundvorrats):
      --one-lower    Genau ein Kleinbuchstabe
      --one-upper    Genau ein Großbuchstabe
      --one-digit    Genau eine Ziffer
      --one-special  Genau ein Sonderzeichen
      --strict       Genau ein Zeichen aus jeder Klasse, die nicht im Grundvorrat ist
```

```
Usage: password-generator serve [OPTIONS]

Options:
  -H, --host <HOST>  Host-Adresse, auf der der Server lauscht [default: 127.0.0.1]
  -p, --port <PORT>  Port, auf dem der Server lauscht [default: 3000]

Alle Optionen von oben gelten auch hier und setzen die Vorgaben des Servers.
```

## OpenBSD

Build, Installation und rc.d-Dienst: siehe [docs/OPENBSD.md](docs/OPENBSD.md).

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
│   ├── password.rs    # Zeichenklassen und Passwort-Generierung
│   └── hash.rs        # Hash-Verfahren (argon2id, bcrypt, sha512-crypt)
├── docs/
│   └── OPENBSD.md     # Build und Betrieb unter OpenBSD
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
- Hashes sind absichtlich langsam (bcrypt Cost 12 etwa 250 ms, Argon2id mit 19 MiB Speicher). Im Server laufen sie in einem eigenen Blocking-Thread. Den `serve`-Modus mit `?hash` nicht ungeschützt ins Internet stellen

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
- **argon2**, **bcrypt**, **sha-crypt** - Hash-Verfahren

## Lizenz

[Ihre Lizenz hier einfügen]

## Autor

[Ihr Name hier einfügen]
