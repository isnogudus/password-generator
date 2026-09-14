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

- **Leicht verwechselbare Buchstaben**: `l`, `o`, `I`, `O`. Weil sie in keinem Vorrat vorkommen, bleiben die Ziffern `0` und `1` unverwechselbar und sind enthalten. Die Buchstaben fehlen auch dann, wenn nur Kleinbuchstaben gewählt sind: Der Empfänger kennt die Erzeugungsregel nicht und soll nie rätseln müssen, und der Gewinn wären nur zwei Bit bei 16 Zeichen.
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

Der Server liefert auf `/` zwei Dinge, je nach `Accept`-Header: Browser bekommen eine kleine Oberfläche, `curl`, `wget` und Healthchecks den reinen Text.

**Web-Oberfläche**: Zeichenklassen, Extras, Länge, Trennzeichen und Hash-Verfahren lassen sich anklicken, jede Änderung erzeugt sofort ein neues Passwort. "Copy password" legt es in die Zwischenablage, bei gewähltem Hash gibt es "Copy hash" dazu. Die Startvorgaben der Oberfläche sind die Optionen, mit denen der Server gestartet wurde. Die Seite ist in das Binary eingebettet, es werden keine externen Ressourcen geladen. Der Kopieren-Knopf nutzt die Clipboard-API, die Browser nur über HTTPS oder auf `localhost` freigeben; bei reinem HTTP im LAN greift ein Fallback, der auch ohne Secure Context funktioniert.

```bash
# Mit Default-Einstellungen (127.0.0.1:3000, 16 Zeichen, -wud)
password-generator serve

# Eigenen Host, Port und Vorgaben angeben
password-generator serve --host 0.0.0.0 --port 8080 -wd --length 20

# Als root: nach dem Binden chroot und Rechte abgeben
password-generator serve --chroot /var/empty --user _pwgen
```

**Sandbox**: Mit `--chroot DIR` wechselt der Server nach dem Binden des Ports per `chroot(2)` in das Verzeichnis, mit `--user NAME` gibt er anschließend die Rechte an diesen Benutzer ab (Gruppen, gid, uid). Beides braucht root und ist für den Betrieb als Systemdienst gedacht, siehe [docs/OPENBSD.md](docs/OPENBSD.md). Das Binary braucht nach dem Start keine Dateien mehr, `/var/empty` reicht daher als Wurzel. Im Docker-Image läuft der Prozess bereits als eigener Benutzer, dort sind die Optionen nicht nötig. Unter OpenBSD blendet der Server zusätzlich immer per `unveil(2)` das Dateisystem aus und ruft `pledge("stdio inet")` auf, beides ohne root; `--chroot` ist dort optional.

Sobald der Server läuft:

```bash
# Im Browser: die Oberfläche
http://127.0.0.1:3000

# Mit curl: reiner Text
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

Die Hilfe des Programms ist englisch, das README deutsch.

```
Generates random, easy-to-type passwords (CLI or web server)

Usage: password-generator [OPTIONS] [COMMAND]

Commands:
  serve  Start the web server, one password per request
  help   Print this message or the help of the given subcommand(s)

Options:
  -l, --length <LENGTH>        Length without separators (server: ?length=N) [default: 16]
  -s, --separator <SEPARATOR>  Separator between blocks of four (server: ?separator=X) [default: .]
      --no-separator           No blocks, print the password as one piece (same as --separator "")
      --hash[=<HASH>]          Also print a hash, tab-separated; --hash alone means argon2id, otherwise --hash=ALGO (server: ?hash or ?hash=ALGO) [possible values: bcrypt, sha512-crypt, argon2id]
  -n, --count <COUNT>          Number of passwords to print (CLI only) [default: 1]
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version

Base set (combinable, e.g. -wd; default -wud; every chosen class appears at least once):
  -w, --lower    Lowercase letters in the base set (abcdefghijkmnpqrstuvwx)
  -u, --upper    Uppercase letters in the base set (ABCDEFGHJKLMNPQRSTUVWX)
  -d, --digits   Digits in the base set (0123456789)
  -x, --special  Special characters in the base set (!#$%&*+=?@_)

Extras (exactly one character of a class outside the base set):
      --one-lower    Exactly one lowercase letter
      --one-upper    Exactly one uppercase letter
      --one-digit    Exactly one digit
      --one-special  Exactly one special character
      --strict       Exactly one character of every class not in the base set
```

```
Start the web server, one password per request

Usage: password-generator serve [OPTIONS]

Options:
  -H, --host <HOST>            Host address to listen on [default: 127.0.0.1]
  -p, --port <PORT>            Port to listen on [default: 3000]
      --chroot <DIR>           chroot(2) into this directory after binding the socket, e.g. /var/empty (needs root; on OpenBSD unveil already hides the filesystem, so this is optional there)
      --user <NAME>            Drop privileges to this user after the chroot (needs root)
  -l, --length <LENGTH>        Length without separators (server: ?length=N) [default: 16]
  -s, --separator <SEPARATOR>  Separator between blocks of four (server: ?separator=X) [default: .]
      --no-separator           No blocks, print the password as one piece (same as --separator "")
      --hash[=<HASH>]          Also print a hash, tab-separated; --hash alone means argon2id, otherwise --hash=ALGO (server: ?hash or ?hash=ALGO) [possible values: bcrypt, sha512-crypt, argon2id]
  -h, --help                   Print help (see more with '--help')

Base set (combinable, e.g. -wd; default -wud; every chosen class appears at least once):
  -w, --lower    Lowercase letters in the base set (abcdefghijkmnpqrstuvwx)
  -u, --upper    Uppercase letters in the base set (ABCDEFGHJKLMNPQRSTUVWX)
  -d, --digits   Digits in the base set (0123456789)
  -x, --special  Special characters in the base set (!#$%&*+=?@_)

Extras (exactly one character of a class outside the base set):
      --one-lower    Exactly one lowercase letter
      --one-upper    Exactly one uppercase letter
      --one-digit    Exactly one digit
      --one-special  Exactly one special character
      --strict       Exactly one character of every class not in the base set
```

### Als Bibliothek

Die Generierung steckt in einer eigenen Bibliothek, ohne CLI und Webserver. Für den Browser wird dieselbe Bibliothek nach WebAssembly übersetzt, etwa für Anwendungen wie [weft](https://github.com/isnogudus/weft), die Passwörter clientseitig erzeugen. Es gibt nur eine Implementierung.

**Rust.** Die Features `hash` (Hash-Verfahren) und `bin` (CLI und Webserver, schließt `hash` ein) sind abschaltbar; als Bibliothek reicht `default-features = false`:

```toml
[dependencies]
password-generator = { git = "https://github.com/isnogudus/password-generator", default-features = false, features = ["hash"] }
```

```rust
use password_generator::{Classes, Options, generate_password};

let pw = generate_password(&Options::default())?;           // x8GG.JpJN.LN40.t7qx
let pin = generate_password(&Options {
    length: 6,
    separator: String::new(),
    base: Classes { digits: true, ..Classes::default() },
    ..Options::default()
})?;                                                         // 193886
```

`Options` hat die Felder `length`, `separator`, `base`, `one` und `strict`, entsprechend den CLI-Schaltern, und ist mit serde deserialisierbar; fehlende Felder nehmen die Standardwerte an. `validate` prüft die Einstellungen vorab, `generate_password` liefert bei ungültigen Einstellungen einen `Error`, `entropy_bits` schätzt die Entropie. Das Hash-Modul (`Algorithm::{Bcrypt, Sha512Crypt, Argon2id}`) gehört zum Feature `hash`.

**Browser (WebAssembly).** Das Crate [`wasm/`](wasm/) enthält die Bindings; [wasm-pack](https://rustwasm.github.io/wasm-pack/) baut daraus ein npm-Paket mit JavaScript-Glue und Typdefinitionen. Die Hash-Verfahren sind nicht enthalten, das Wasm ist etwa 80 KB groß.

```bash
cd wasm && wasm-pack build --release --target web    # ergibt wasm/pkg/
```

Das Verzeichnis `pkg/` wird ins Projekt kopiert oder per `npm install ../password-generator/wasm/pkg` eingebunden. Bei Vite bindet man das Wasm als URL ein und initialisiert einmal, am besten faul beim ersten Aufruf:

```js
import init, { generatePassword, validate, entropyBits } from './pkg/password_generator_wasm.js'
import wasmUrl from './pkg/password_generator_wasm_bg.wasm?url'

await init({ module_or_path: wasmUrl })

generatePassword()                                              // 'x8GG.JpJN.LN40.t7qx'
generatePassword({ length: 6, separator: '', base: { digits: true } })   // '193886'
generatePassword({ base: { lower: true }, one: { upper: true, digits: true } })   // Apple-Stil
generatePassword({ strict: true })                              // Standard plus genau ein Sonderzeichen
entropyBits({ length: 20 })                                     // 115
```

Die Optionen sind ein Objekt mit denselben Feldern wie in Rust: `length`, `separator`, `base`, `one`, `strict`; `base` und `one` haben die Schlüssel `lower`, `upper`, `digits`, `special`. Ungültige Einstellungen werfen einen `Error` mit derselben Meldung wie das CLI.

## OpenBSD

Build, Installation und rc.d-Dienst: siehe [docs/OPENBSD.md](docs/OPENBSD.md).

## Hinter einem Reverse Proxy betreiben

Der Server lauscht standardmäßig nur auf `127.0.0.1:3000`, ein Reverse Proxy übernimmt TLS und die öffentliche Adresse. TLS ist nicht nur Kosmetik: Browser geben die Clipboard-API nur über HTTPS oder auf `localhost` frei, die Kopieren-Knöpfe der Oberfläche brauchen sie.

### Caddy

Caddy kann von Haus aus kein Programm pro Request starten (kein CGI im Kern; dafür wäre ein Custom-Build mit dem Plugin `caddy-cgi` nötig). Der übliche Weg ist daher der `serve`-Modus hinter `reverse_proxy`, Zertifikate holt Caddy selbst:

```
pw.example.org {
    reverse_proxy 127.0.0.1:3000
}
```

### nginx

Ein vollständiger Server-Block liegt in [examples/nginx.conf](examples/nginx.conf): HTTP-Weiterleitung auf HTTPS, TLS, Proxy auf `127.0.0.1:3000`, ein längeres `proxy_read_timeout` für `?hash` und optional `auth_basic`. Der Kern:

```
location / {
    proxy_pass         http://127.0.0.1:3000;
    proxy_set_header   Host              $host;
    proxy_set_header   X-Forwarded-For   $proxy_add_x_forwarded_for;
    proxy_set_header   X-Forwarded-Proto $scheme;
    proxy_read_timeout 30s;
}
```

Der `Accept`-Header entscheidet zwischen Oberfläche und reinem Text; nginx reicht ihn unverändert durch. Für `auth_basic` lässt sich die Passwortdatei mit dem Generator selbst füllen, nginx prüft sie über `crypt(3)`, das bcrypt auf OpenBSD und auf Linux mit libxcrypt versteht:

```bash
printf 'admin:%s\n' "$(password-generator --hash=bcrypt | cut -f2)" > /etc/nginx/htpasswd
```

Unter OpenBSD: siehe [docs/OPENBSD.md](docs/OPENBSD.md), dort mit `acme-client` und dem chrooteten nginx-Paket.

### httpd und relayd (OpenBSD)

Mit Bordmitteln von OpenBSD: `httpd` bedient Port 80 mit ACME-Challenge und Weiterleitung, `relayd` terminiert TLS und leitet auf `127.0.0.1:3000` weiter, mit Health-Check auf `/`. Beispiele in [examples/httpd.conf](examples/httpd.conf) und [examples/relayd.conf](examples/relayd.conf), Schritt für Schritt in [docs/OPENBSD.md](docs/OPENBSD.md).

## Projektstruktur

```
password-generator/
├── src/
│   ├── lib.rs         # Bibliothek: exportiert password und hash
│   ├── password.rs    # Zeichenklassen und Passwort-Generierung
│   ├── hash.rs        # Hash-Verfahren (Feature "hash")
│   └── main.rs        # CLI, Subcommand "serve", Webserver (Feature "bin")
├── wasm/              # Eigenständiges Crate: WebAssembly-Bindings für den Browser
│   ├── Cargo.toml
│   ├── src/lib.rs
│   └── test.mjs       # Prüft das gebaute Paket unter Node
├── static/
│   └── index.html     # Web-Oberfläche (wird ins Binary eingebettet)
├── docs/
│   └── OPENBSD.md     # Build und Betrieb unter OpenBSD
├── examples/
│   ├── nginx.conf     # Server-Block für nginx als Reverse Proxy
│   ├── httpd.conf     # OpenBSD httpd: Port 80, ACME-Challenge, Redirect
│   └── relayd.conf    # OpenBSD relayd: TLS und Proxy auf den Generator
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
cargo test                          # Tests ausführen (Bibliothek und Binary)
cargo build --no-default-features   # nur die Bibliothek, ohne Hash- und Server-Abhängigkeiten
cargo fmt                           # Code formatieren
cargo clippy                        # Linter ausführen

cd wasm
wasm-pack build --release --target web            # Wasm-Paket bauen (ergibt pkg/)
cargo clippy --target wasm32-unknown-unknown      # Linter für die Bindings
node --test                                       # gebautes Paket unter Node prüfen
```

Das Wasm-Crate hat einen eigenen Lockfile und Build-Cache, weil es nicht zum Workspace des Hauptcrates gehört; so bleibt der Docker-Build des Servers unberührt.

## Abhängigkeiten

Bibliothek:

- **rand** - Zufallszahlengenerator
- **serde** - Deserialisierung der Optionen

Feature `hash`:

- **argon2**, **bcrypt**, **sha-crypt** - Hash-Verfahren

Feature `bin` (CLI und Webserver):

- **axum** - Moderner Web-Framework
- **tokio** - Async Runtime
- **clap** - CLI Argument Parser
- **serde_json** - Vorgaben für die Web-Oberfläche
- **nix** - chroot, Rechteabgabe, pledge/unveil

Wasm-Crate: **wasm-bindgen**, **serde-wasm-bindgen** und **getrandom** mit JavaScript-Backend.

## Lizenz

[Ihre Lizenz hier einfügen]

## Autor

[Ihr Name hier einfügen]
