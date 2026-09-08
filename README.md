# password-generator

Generiert zufällige, gut abtippbare Passwörter — als CLI-Tool oder als kleiner Webserver in Rust. Gegenstück zu `password-rs` (merkbare Passphrasen).

## Überblick

Passwörter bestehen aus Groß-/Kleinbuchstaben und Ziffern, dargestellt in Viererblöcken, getrennt durch `.`.

**Beispiele:**
- `rCJS.2RCX.tAMG` (Standard: 12 Zeichen)
- `Vr3c.Jt6h.UAK7.6u3J` (16 Zeichen)

Der Punkt ist nur Darstellung: Die Passwortlänge zählt die Zeichen ohne Trennzeichen. Das Trennzeichen ist frei wählbar (`--separator`) oder lässt sich ganz weglassen (`--no-separator`).

**Strict-Modus** (`--strict`): Für Passwortabfragen, die zwingend Sonderzeichen verlangen. Erweitert den Zeichenvorrat um `!#$%&*+=?@_` und garantiert mindestens eines davon.

- `@T7S.gwcW.Tt8x`
- `cu5fwL#+92jg&!Bx` (strict, ohne Trennzeichen, 16 Zeichen)

**Kleinbuchstaben-Modus** (`--lowercase`): Nur Kleinbuchstaben, dafür 16 Zeichen. Angelehnt an Apples Schlüsselbund-Passwörter: ohne Shift-Taste tippbar, auf jeder Tastatur gleich, und mit 16 Zeichen genauso stark wie 12 gemischte. Auf Wunsch kommt genau ein Großbuchstabe, eine Ziffer und/oder ein Sonderzeichen dazu, damit strenge Richtlinien zufrieden sind.

- `gpdq.hkmr.kdnd.xpdg`
- `wmd$.hbdg.grji.gGm7` (`--lowercase --upper --digit --special`)

**Alnum-Modus** (`--alnum`): Kleinbuchstaben und Ziffern 0 bis 9, Standardlänge 16. Mindestens ein Buchstabe und eine Ziffer sind garantiert. Auch hier lassen sich genau ein Großbuchstabe und/oder ein Sonderzeichen zuschalten. Mit 16 Zeichen aus 32 sind das rund 80 Bit.

- `k7hq.3w9m.d2xp.6nvr`
- `F5t0.+7v7.qk3m.8hbs` (`--alnum --upper --special`)

**Hashes** (`--hash`): Auf Wunsch wird zu jedem Passwort gleich der passende Hash ausgegeben, getrennt durch einen Tabulator. Unterstützt werden bcrypt, SHA-512-crypt und Argon2id.

```
B5Vk.Xaqc.6Wjg	$2b$12$bjOdbhv8LUmzFwsGEC4R6.VCkRTtMkqLyqWJqWBdvKnPc9XZarlyu
```

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

Im Strict-Modus kommen diese Sonderzeichen hinzu (auf QWERTY und QWERTZ vorhanden, ohne Quoting-Fallen wie `'"\` und ohne das mit `l`/`I` verwechselbare `|`):

```
!#$%&*+=?@_
```

Im Alnum-Modus werden alle zehn Ziffern verwendet: `l` und `o` fehlen bei den Kleinbuchstaben ohnehin, und `I` und `O` kommen im gesamten Vorrat nicht vor, daher sind 0 und 1 dort nicht verwechselbar.

Jedes Passwort enthält garantiert mindestens einen Klein-, einen Großbuchstaben und eine Ziffer, im Strict-Modus zusätzlich ein Sonderzeichen. Im Kleinbuchstaben-Modus sind alle Zeichen klein, bis auf genau einen Großbuchstaben (`--upper`), eine Ziffer (`--digit`) bzw. ein Sonderzeichen (`--special`), sofern zugeschaltet; `--strict` schaltet dort alle drei zu. Im Alnum-Modus gibt es `--upper` und `--special`, `--strict` schaltet beide zu. Die Position der Pflichtzeichen wird zufällig gemischt. Zeichen, die im Trennzeichen vorkommen, werden aus den Sonderzeichen entfernt, damit Blockgrenze und Inhalt unterscheidbar bleiben.

### Entropie

| Modus                                   | Zeichen | Vorrat | Entropie |
|-----------------------------------------|---------|--------|----------|
| Standard                                | 12      | 52     | ~68 Bit  |
| Strict                                  | 12      | 63     | ~72 Bit  |
| Kleinbuchstaben                         | 16      | 22     | ~71 Bit  |
| Kleinbuchstaben, 20 Zeichen             | 20      | 22     | ~89 Bit  |
| Alnum (Kleinbuchstaben + Ziffern)       | 16      | 32     | ~80 Bit  |

Die Extras im Kleinbuchstaben-Modus ändern die Entropie kaum. Alles über 70 Bit ist gegen Online-Angriffe wie gegen Offline-Angriffe auf bcrypt oder Argon2 mehr als ausreichend; der Hebel ist die Länge, ein Block mehr bringt 18 Bit.

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

# Anderes Trennzeichen bzw. keines
password-generator -s -
password-generator --no-separator

# Strict-Modus mit Sonderzeichen
password-generator -x

# Kleinbuchstaben-Modus (16 Zeichen), wahlweise mit je einem Extra
password-generator -w
password-generator -w --upper --digit --special
password-generator -w -x                    # dasselbe wie die Zeile darüber

# Alnum-Modus (16 Zeichen aus Kleinbuchstaben und Ziffern), wahlweise mit Großbuchstabe und Sonderzeichen
password-generator -a
password-generator -a --upper --special

# Passwort und Hash (bcrypt, sha512-crypt oder argon2id)
password-generator --hash bcrypt
```

### Hash-Verfahren

| `--hash`       | Format               | Typischer Einsatz                                    |
|----------------|----------------------|------------------------------------------------------|
| `bcrypt`       | `$2b$12$…`           | Caddy `basic_auth`, htpasswd, OpenBSD `passwd`, Gitea |
| `sha512-crypt` | `$6$rounds=5000$…`   | `/etc/shadow` unter Linux, `chpasswd -e`             |
| `argon2id`     | `$argon2id$v=19$…`   | Eigene Anwendungen, PHC-String                        |

Jeder Hash bekommt ein frisches zufälliges Salt. Die Ausgabezeile ist `<passwort>\t<hash>`, so lässt sich mit `cut -f2` der Hash allein herausziehen:

```bash
password-generator --hash bcrypt | cut -f2
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

# Optionen pro Request
curl 'http://127.0.0.1:3000/?length=20'
curl 'http://127.0.0.1:3000/?separator=-'
curl 'http://127.0.0.1:3000/?separator='          # ohne Trennzeichen
curl 'http://127.0.0.1:3000/?strict=1'
curl 'http://127.0.0.1:3000/?lowercase=1&digit=1'
curl 'http://127.0.0.1:3000/?alnum=1&upper=1'
curl 'http://127.0.0.1:3000/?hash=bcrypt'         # Passwort<TAB>Hash
```

Die Query-Parameter `length`, `separator`, `strict`, `lowercase`, `alnum`, `upper`, `digit`, `special` und `hash` überschreiben die beim Start gesetzten Standardwerte. Für die Schalter gelten `1`, `true`, `yes`, `on` oder ein leerer Wert (`?strict`) als wahr. Jeder Request generiert ein neues, zufälliges Passwort. Ungültige Werte (Länge außerhalb 4 bis 128, Trennzeichen länger als 8 Zeichen) beantwortet der Server mit `400 Bad Request`.

### CLI-Optionen

```
Generiert zufällige Passwörter (CLI oder Webserver)

Usage: password-generator [OPTIONS] [COMMAND]

Commands:
  serve  Webserver starten, der pro Request ein Passwort liefert
  help   Print this message or the help of the given subcommand(s)

Options:
  -l, --length <LENGTH>        Länge der Passwörter, Standard 12, 16 mit --lowercase oder --alnum (beim Server per ?length=N überschreibbar)
  -s, --separator <SEPARATOR>  Trennzeichen zwischen den Viererblöcken (beim Server per ?separator=X überschreibbar) [default: .]
      --no-separator           Keine Blöcke, Passwort am Stück ausgeben (entspricht --separator "")
  -x, --strict                 Strict-Modus für strenge Passwortrichtlinien: Sonderzeichen hinzufügen und mindestens eines garantieren; mit --lowercase wie --upper --digit --special, mit --alnum wie --upper --special (Server: ?strict=1)
  -w, --lowercase              Kleinbuchstaben-Modus: nur Kleinbuchstaben, Standardlänge 16 (beim Server per ?lowercase=1 überschreibbar)
  -a, --alnum                  Alnum-Modus: Kleinbuchstaben und Ziffern 0-9, Standardlänge 16 (beim Server per ?alnum=1 überschreibbar)
      --upper                  Genau ein Großbuchstabe, Rest aus dem Grundvorrat (mit --lowercase oder --alnum; Server: ?upper=1)
      --digit                  Genau eine Ziffer, Rest klein (nur mit --lowercase; Server: ?digit=1)
      --special                Genau ein Sonderzeichen, Rest aus dem Grundvorrat (mit --lowercase oder --alnum; Server: ?special=1)
      --hash <HASH>            Zusätzlich einen Hash des Passworts ausgeben, durch Tabulator getrennt (beim Server per ?hash=ALGO überschreibbar) [possible values: bcrypt, sha512-crypt, argon2id]
  -n, --count <COUNT>          Anzahl der auszugebenden Passwörter (nur CLI) [default: 1]
  -h, --help                   Print help
  -V, --version                Print version
```

```
Usage: password-generator serve [OPTIONS]

Options:
  -H, --host <HOST>            Host-Adresse, auf der der Server lauscht [default: 127.0.0.1]
  -p, --port <PORT>            Port, auf dem der Server lauscht [default: 3000]
  -l, --length, -s, --separator, --no-separator, -x, --strict,
  -w, --lowercase, -a, --alnum, --upper, --digit, --special, --hash
                               wie oben, setzen die Standardwerte des Servers
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
│   ├── password.rs    # Zeichenvorrat und Passwort-Generierung
│   └── hash.rs        # Hash-Verfahren (bcrypt, sha512-crypt, argon2id)
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
- Hashes sind absichtlich langsam (bcrypt Cost 12 etwa 250 ms, Argon2id mit 19 MiB Speicher). Im Server laufen sie in einem eigenen Blocking-Thread. Den `serve`-Modus mit `?hash=` nicht ungeschützt ins Internet stellen

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
- **bcrypt**, **sha-crypt**, **argon2** - Hash-Verfahren

## Lizenz

[Ihre Lizenz hier einfügen]

## Autor

[Ihr Name hier einfügen]
