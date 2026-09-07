# Build und Betrieb unter OpenBSD

Rust führt OpenBSD als Tier-3-Target: Es gibt keine vorgebaute Standardbibliothek
und kein `rustup target add`. Ein Cross-Build von macOS oder Linux ist deshalb
unpraktisch. Der einfache Weg ist, direkt auf der OpenBSD-Maschine zu bauen.

## 1. Rust installieren

OpenBSD liefert Rust als Paket, `rustup` wird nicht benötigt:

```sh
doas pkg_add rust
rustc --version
```

Das Projekt nutzt Edition 2024 und braucht daher **Rust 1.85 oder neuer**.
Das ist ab OpenBSD 7.7 der Fall. Auf älteren Releases zuerst das System
aktualisieren (`sysupgrade`), ein neueres Rust-Paket gibt es dort nicht.

Git ist praktisch, aber optional (`doas pkg_add git`).

## 2. Quellen holen und bauen

```sh
git clone <repo-url> password-generator
cd password-generator
cargo build --release --locked
```

`--locked` baut exakt die Versionen aus `Cargo.lock`. Das Binary liegt danach in
`target/release/password-generator`.

Kurzer Test:

```sh
./target/release/password-generator
./target/release/password-generator -n 3 -l 16
```

### Ohne Internet auf dem Zielsystem

Wenn die OpenBSD-Maschine keine Crates aus dem Netz laden darf, die
Abhängigkeiten auf dem Entwicklungsrechner einpacken:

```sh
cargo vendor
tar czf password-generator-src.tgz Cargo.toml Cargo.lock src vendor .cargo
```

Vor dem Packen in `.cargo/config.toml` eintragen (der Befehl `cargo vendor`
gibt den Block am Ende aus):

```toml
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
```

Auf OpenBSD dann entpacken und wie oben mit `cargo build --release --locked`
bauen, es wird nichts mehr heruntergeladen.

## 3. Installieren

```sh
doas install -o root -g bin -m 755 target/release/password-generator /usr/local/bin/
```

Damit ist die CLI-Variante für alle Benutzer verfügbar.

## 4. Als Dienst betreiben (rc.d)

Eigener Benutzer ohne Shell und ohne Home:

```sh
doas useradd -g =uid -d /var/empty -s /sbin/nologin _pwgen
```

rc.d-Skript `/etc/rc.d/password_generator` anlegen:

```sh
#!/bin/ksh

daemon="/usr/local/bin/password-generator"
daemon_flags="serve --host 127.0.0.1 --port 3000"
daemon_user="_pwgen"

. /etc/rc.d/rc.subr

rc_bg=YES
rc_reload=NO

rc_cmd $1
```

`rc_bg=YES` ist nötig, weil das Programm im Vordergrund läuft und sich nicht
selbst daemonisiert. Skript ausführbar machen, aktivieren und starten:

```sh
doas chmod 755 /etc/rc.d/password_generator
doas rcctl enable password_generator
doas rcctl start password_generator
curl http://127.0.0.1:3000/
```

Flags lassen sich später ohne Bearbeiten des Skripts ändern, z.B. andere
Standardlänge, anderes Trennzeichen oder Strict-Modus:

```sh
doas rcctl set password_generator flags serve --host 127.0.0.1 --port 3000 --length 16 --separator - --strict
doas rcctl restart password_generator
```

Logs des Dienstes landen nicht in einer Datei, die Startmeldung geht nach
stdout. Bei Problemen den Dienst zum Debuggen einmal direkt starten:

```sh
doas -u _pwgen /usr/local/bin/password-generator serve --port 3000
```

## 5. Hinter Caddy

Caddy kann von Haus aus kein Programm pro Request starten, der Dienst läuft
deshalb dauerhaft und Caddy leitet nur weiter:

```sh
doas pkg_add caddy
```

`/etc/caddy/Caddyfile`:

```
pw.example.org {
    reverse_proxy 127.0.0.1:3000
}
```

```sh
doas rcctl enable caddy
doas rcctl start caddy
```

Der Dienst selbst bleibt auf `127.0.0.1` gebunden und ist nur über Caddy
erreichbar. Alternativ geht auch das mitgelieferte `relayd`/`httpd`, Caddy ist
aber wegen der automatischen TLS-Zertifikate der bequemere Weg.

## 6. Aktualisieren

```sh
cd password-generator
git pull
cargo build --release --locked
doas install -o root -g bin -m 755 target/release/password-generator /usr/local/bin/
doas rcctl restart password_generator
```

## Hinweise

- Der Zufall kommt über `getrandom` aus `getentropy(2)`, also aus dem
  Kernel-CSPRNG von OpenBSD.
- `pledge(2)` und `unveil(2)` nutzt das Programm bislang nicht.
- Diese Anleitung wurde nicht auf einer OpenBSD-Maschine durchgespielt. Die
  Schritte entsprechen dem üblichen Vorgehen für Rust-Programme und rc.d-Dienste
  unter OpenBSD 7.7.
