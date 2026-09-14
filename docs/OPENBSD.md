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

Unter OpenBSD sichert sich der Server nach dem Binden des Ports selbst ab:
`unveil("/", "")` blendet das gesamte Dateisystem aus, danach erlaubt
`pledge("stdio inet")` nur noch Socket-Betrieb, Threads, Speicher und
`getentropy(2)`; jeder andere Systemaufruf beendet den Prozess. Beides
braucht kein root. Das Binary braucht ohnehin keine Datei mehr: Seite und
Zeichenvorräte sind eingebettet. Der Dienst kann daher direkt als `_pwgen`
laufen. rc.d-Skript `/etc/rc.d/password_generator` anlegen:

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

`rc_bg=YES` ist nötig, weil das Programm im Vordergrund läuft und sich
nicht selbst daemonisiert.

Wer zusätzlich einen chroot möchte, lässt `daemon_user` weg und übergibt
stattdessen `--chroot /var/empty --user _pwgen`: Der Dienst startet dann als
root, bindet den Port, wechselt die Wurzel, gibt die Rechte ab und ruft erst
danach unveil und pledge auf. Auf anderen Systemen ohne unveil ist das der
Weg, das Dateisystem zu verstecken. Skript ausführbar machen, aktivieren und
starten:

```sh
doas chmod 755 /etc/rc.d/password_generator
doas rcctl enable password_generator
doas rcctl start password_generator
curl http://127.0.0.1:3000/
```

Flags lassen sich später ohne Bearbeiten des Skripts ändern, z.B. andere
Standardlänge, anderes Trennzeichen oder Strict-Modus:

```sh
doas rcctl set password_generator flags serve --host 127.0.0.1 --port 3000 --length 20 --separator - --strict
doas rcctl restart password_generator
```

Logs des Dienstes landen nicht in einer Datei, die Startmeldung geht nach
stdout. Bei Problemen den Dienst zum Debuggen einmal direkt starten:

```sh
doas -u _pwgen /usr/local/bin/password-generator serve --port 3000
```

Die Startmeldung zeigt unveil und die pledge-Zusagen, bei der chroot-Variante
zusätzlich Verzeichnis, Benutzer, uid und gid.

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
erreichbar. Caddy ist wegen der automatischen TLS-Zertifikate der bequemste
Weg; wer nginx bevorzugt, findet den Aufbau im nächsten Abschnitt.

## 5a. Hinter nginx

nginx aus den Paketen läuft unter OpenBSD als Benutzer `www` und chrootet
sich beim Start nach `/var/www`. Zertifikate und Konfiguration liest es vor
dem chroot, deshalb bleiben die üblichen Pfade unter `/etc` gültig.

```sh
doas pkg_add nginx
```

**Zertifikat mit acme-client** (Bordmittel). In `/etc/acme-client.conf`:

```
authority letsencrypt {
    api url "https://acme-v02.api.letsencrypt.org/directory"
    account key "/etc/acme/letsencrypt-privkey.pem"
}

domain pw.example.org {
    domain key "/etc/ssl/private/pw.example.org.key"
    domain full chain certificate "/etc/ssl/pw.example.org.fullchain.pem"
    sign with letsencrypt
}
```

Die Challenge beantwortet nginx aus `/var/www/acme`, dafür im HTTP-Block auf
Port 80 vor der Weiterleitung:

```
location /.well-known/acme-challenge/ {
    root /var/www/acme;
    rewrite ^/.well-known/acme-challenge/(.*)$ /$1 break;
}
```

Erstes Zertifikat holen, dann die Erneuerung per cron (`crontab -e` als
root):

```sh
doas acme-client -v pw.example.org
```

```
~	*	*	*	*	acme-client pw.example.org && rcctl reload nginx
```

**Server-Block**: Den Inhalt von `examples/nginx.conf` aus dem Repository in
`/etc/nginx/nginx.conf` innerhalb des `http { … }`-Blocks einfügen (oder als
eigene Datei ablegen und per `include` einbinden), Hostname und die beiden
Zertifikatspfade anpassen. Der Block nutzt bereits die acme-client-Pfade von
oben. Der wichtige Teil ist der Proxy auf den Generator:

```
location / {
    proxy_pass         http://127.0.0.1:3000;
    proxy_set_header   Host              $host;
    proxy_set_header   X-Forwarded-For   $proxy_add_x_forwarded_for;
    proxy_set_header   X-Forwarded-Proto $scheme;
    proxy_read_timeout 30s;
}
```

Der Proxy auf `127.0.0.1:3000` funktioniert aus dem nginx-chroot heraus,
weil Netzwerkzugriffe davon nicht betroffen sind.

**Optional Zugriffsschutz**: nginx prüft `auth_basic_user_file` über
`crypt(3)`, das unter OpenBSD bcrypt versteht. Die Datei muss im chroot
liegen, also unter `/var/www`:

```sh
printf 'admin:%s\n' "$(password-generator --hash=bcrypt | tee /dev/tty | cut -f2)" | doas tee /var/www/htpasswd
```

Im Server-Block dann `auth_basic_user_file /htpasswd;` (Pfad relativ zum
chroot) und die beiden `auth_basic`-Zeilen einkommentieren.

**Prüfen und starten**:

```sh
doas nginx -t
doas rcctl enable nginx
doas rcctl start nginx
curl -sI https://pw.example.org/ | head -1
```

## 6. Hashes für OpenBSD-Dienste

OpenBSD verwendet für `passwd` und `htpasswd` bcrypt. Der Generator liefert
das passende Format direkt mit:

```sh
password-generator --hash=bcrypt
```

Die Ausgabe ist `<passwort>	<hash>` (Tabulator). Beispiele:

```sh
# Passwort für einen Systembenutzer setzen (Hash aus der zweiten Spalte)
doas usermod -p "$(password-generator --hash=bcrypt | tee /dev/tty | cut -f2)" benutzer

# Eintrag für Caddy basic_auth
password-generator --hash=bcrypt
```

Für Caddy den Hash in die Caddyfile übernehmen:

```
pw.example.org {
    basic_auth {
        admin $2b$12$...
    }
    reverse_proxy 127.0.0.1:3000
}
```

## 7. Aktualisieren

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
- `unveil(2)` und `pledge(2)` (`stdio inet`) sind fest eingebaut und
  brauchen kein root; `--chroot` ist unter OpenBSD optional.
- Diese Anleitung wurde nicht auf einer OpenBSD-Maschine durchgespielt. Die
  Schritte entsprechen dem üblichen Vorgehen für Rust-Programme und rc.d-Dienste
  unter OpenBSD 7.7.
