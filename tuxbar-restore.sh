#!/usr/bin/env bash
# TuxBar - pelna instalacja od zera na Fedora Asahi Remix (MacBook Pro M2, Mac14,7)
#
# Co robi: instaluje zaleznosci, pobiera zrodla TuxBar, nakłada poprawki pod
# Apple Silicon (klawiatura Fn + temperatura), buduje i instaluje usluge systemd.
#
# Uzycie (jako zwykly uzytkownik, NIE przez sudo - sudo zapyta samo):
#   chmod +x tuxbar-restore.sh && ./tuxbar-restore.sh
#
# WAZNE: wpisz ponizej hash commita, ktory u Ciebie dzialal
# (na starym systemie: git -C ~/tuxbar rev-parse HEAD).
# Dzieki temu poprawki zawsze pasuja do tej samej wersji kodu.
set -euo pipefail

REPO="https://github.com/MacBunciak/tuxbar"            # Twoj fork (pierwszy wybor)
REPO_FALLBACK="https://github.com/mylnzon58/tuxbar"   # oryginal autora (gdyby fork nie dzialal)
TUXBAR_COMMIT="48fec3d719ee735f2a40a22f756cc8c4f5be6724"          # przypięty commit, na którym działały poprawki
WORKDIR="$HOME/tuxbar"

if [[ $EUID -eq 0 ]]; then
    echo "Nie uruchamiaj jako root. Uruchom zwyklym uzytkownikiem." >&2
    exit 1
fi

echo "==> 1/5 Zaleznosci"
sudo dnf install -y rust cargo git pipewire-devel evtest

echo "==> 2/5 Zrodla"
if [[ ! -d "$WORKDIR/.git" ]]; then
    git clone "$REPO" "$WORKDIR" || {
        echo "Nie udalo sie pobrac forka, probuje oryginal autora..."
        rm -rf "$WORKDIR"
        git clone "$REPO_FALLBACK" "$WORKDIR"
    }
fi
cd "$WORKDIR"
if [[ -n "$TUXBAR_COMMIT" ]]; then
    git checkout "$TUXBAR_COMMIT"
else
    echo "UWAGA: brak przypietego commita, uzywam aktualnego stanu repo."
fi

echo "==> 3/5 Poprawki pod Apple Silicon"
python3 - <<'PY'
import re, sys, pathlib

# --- Poprawka 1: klawiatura Fn (Apple MTP keyboard) ---
kb = pathlib.Path("src/core/keyboard.rs")
s = kb.read_text()
if 'apple mtp keyboard' not in s:
    old = 'if name.contains("apple internal keyboard") {'
    new = 'if name.contains("apple internal keyboard") || name.contains("apple mtp keyboard") {'
    if old not in s:
        sys.exit("BLAD: nie znaleziono linii z nazwa klawiatury w keyboard.rs (kod sie zmienil?)")
    kb.write_text(s.replace(old, new, 1))
    print("  keyboard.rs: poprawiono")
else:
    print("  keyboard.rs: juz poprawione")

# --- Poprawka 2: temperatura (najcieplejszy czujnik macsmc_hwmon) ---
tl = pathlib.Path("src/services/telemetry.rs")
s = tl.read_text()
if 'macsmc_hwmon' not in s:
    anchor = 'fn read_cpu_temp() -> f32 {'
    if anchor not in s:
        sys.exit("BLAD: nie znaleziono read_cpu_temp() w telemetry.rs (kod sie zmienil?)")
    block = '''
        // Priority 0: Apple Silicon (Asahi) - no CPU die sensor exposed,
        // so report the hottest sensor of macsmc_hwmon
        if let Ok(entries) = fs::read_dir("/sys/class/hwmon") {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Ok(name) = fs::read_to_string(p.join("name")) {
                    if name.trim() == "macsmc_hwmon" {
                        let mut max_t = 0.0f32;
                        for i in 1..=16 {
                            if let Ok(s) = fs::read_to_string(p.join(format!("temp{i}_input"))) {
                                if let Ok(v) = s.trim().parse::<f32>() {
                                    let t = if v > 1000.0 { v / 1000.0 } else { v };
                                    if (10.0..=115.0).contains(&t) && t > max_t {
                                        max_t = t;
                                    }
                                }
                            }
                        }
                        if max_t > 0.0 {
                            return max_t;
                        }
                    }
                }
            }
        }
'''
    tl.write_text(s.replace(anchor, anchor + block, 1))
    print("  telemetry.rs: poprawiono")
else:
    print("  telemetry.rs: juz poprawione")

# --- Poprawka 3: plynny wizualizer (mniejsze opoznienie parec) ---
ac = pathlib.Path("src/services/audio_capture.rs")
s = ac.read_text()
if "--latency-msec" not in s:
    i = s.find('Command::new("parec")')
    if i < 0:
        sys.exit("BLAD: nie znaleziono wywolania parec w audio_capture.rs (kod sie zmienil?)")
    m = re.compile(r'^([ \t]*)"--rate=8000",[ \t]*\n', re.M).search(s, i)
    if not m:
        sys.exit("BLAD: nie znaleziono argumentu --rate=8000 przy parec (kod sie zmienil?)")
    ins = m.group(1) + '"--latency-msec=30",\n'
    ac.write_text(s[:m.end()] + ins + s[m.end():])
    print("  audio_capture.rs: poprawiono")
else:
    print("  audio_capture.rs: juz poprawione")

# --- Poprawka 4: polskie napisy ---
def replace_all(path, pairs):
    f = pathlib.Path(path)
    t = f.read_text()
    changed = False
    for old, new in pairs:
        if old in t:
            t = t.replace(old, new)
            changed = True
    f.write_text(t)
    print("  %s: %s" % (path, "poprawiono" if changed else "juz poprawione"))

replace_all("src/widgets/now_playing.rs", [("Toca ▶ para reproducir", "Dotknij ▶ aby odtworzyć")])
replace_all("src/widgets/gallery.rs", [('"ABRIR"', '"OTWÓRZ"'), ('"Sin fotos"', '"Brak zdjęć"')])

# --- Poprawka 5: sprzatanie procesow parec (bez zombie) ---
ac = pathlib.Path("src/services/audio_capture.rs")
s = ac.read_text()
if "child.wait()" not in s:
    m = re.search(r'^([ \t]*)let _ = child\.kill\(\);[ \t]*\n', s, re.M)
    if not m:
        sys.exit("BLAD: nie znaleziono child.kill() w audio_capture.rs (kod sie zmienil?)")
    ins = m.group(1) + "let _ = child.wait();\n"
    ac.write_text(s[:m.end()] + ins + s[m.end():])
    print("  audio_capture.rs (wait): poprawiono")
else:
    print("  audio_capture.rs (wait): juz poprawione")

# --- Poprawka 6: polskie katalogi w galerii (Obrazy, Pulpit, Pobrane) ---
gl = pathlib.Path("src/widgets/gallery.rs")
s = gl.read_text()
if "Obrazy" not in s:
    m = re.search(r'^([ \t]*)dirs_to_check\.push\(session\.home\.join\("Pictures/Capturas de pantalla"\)\);[ \t]*\n', s, re.M)
    if not m:
        sys.exit("BLAD: nie znaleziono listy katalogow w gallery.rs (kod sie zmienil?)")
    ind = m.group(1)
    ins = "".join(
        '%sdirs_to_check.push(session.home.join("%s"));\n' % (ind, d)
        for d in ("Obrazy/Zrzuty ekranu", "Obrazy", "Pulpit", "Pobrane")
    )
    gl.write_text(s[:m.end()] + ins + s[m.end():])
    print("  gallery.rs (katalogi): poprawiono")
else:
    print("  gallery.rs (katalogi): juz poprawione")
PY

echo "==> 4/5 Budowanie"
cargo build --release

echo "==> 5/5 Instalacja uslugi (wymaga sudo)"
INSTALLER="$(mktemp)"
cat > "$INSTALLER" <<'INSTALLER_EOF'
# TuxBar installer for Fedora Asahi Remix on Apple Silicon (MacBook Pro M2, Mac14,7)
# Run from the tuxbar source directory after: cargo build --release
#   sudo ./tuxbar-install-asahi.sh
#
# Installs:
#   /usr/local/bin/tuxbar
#   /usr/local/bin/tuxbar-toggle
#   /etc/systemd/system/tuxbar.service
set -euo pipefail

# --- Device paths found on this machine (adjust if they ever change) ---
CARD="/dev/dri/by-path/platform-228200000.display-pipe-card"
INPUT="/dev/input/by-path/platform-23510c000.spi-cs-0-event"

BIN_SRC="${1:-./target/release/tuxbar}"

if [[ $EUID -ne 0 ]]; then
    echo "Uruchom jako root: sudo $0" >&2
    exit 1
fi

if [[ ! -x "$BIN_SRC" ]]; then
    echo "Nie znaleziono binarki: $BIN_SRC" >&2
    echo "Uruchom skrypt z katalogu ~/tuxbar po 'cargo build --release'," >&2
    echo "albo podaj sciezke: sudo $0 /sciezka/do/tuxbar" >&2
    exit 1
fi

echo "==> Instaluje binarke"
install -Dm755 "$BIN_SRC" /usr/local/bin/tuxbar
command -v restorecon >/dev/null 2>&1 && restorecon -v /usr/local/bin/tuxbar || true

echo "==> Tworze /etc/systemd/system/tuxbar.service"
cat > /etc/systemd/system/tuxbar.service <<EOF
[Unit]
Description=TuxBar - dynamic widget engine for Apple Touch Bar (Apple Silicon)
# Only one program can own the Touch Bar display at a time. tiny-dfr is kept
# from starting by 'tuxbar-toggle on' (it masks tiny-dfr), so no Conflicts= here:
# Conflicts= made tiny-dfr's udev start / restart kill TuxBar at boot.
After=systemd-udevd.service

[Service]
Type=simple
# Wait (max 30 s) for the Touch Bar display and touch devices to appear
ExecStartPre=/bin/sh -c 'for i in \$(seq 1 30); do [ -e "${CARD}" ] && [ -e "${INPUT}" ] && exit 0; sleep 1; done; echo "Touch Bar devices not found" >&2; exit 1'
ExecStart=/usr/local/bin/tuxbar run --card ${CARD} --input ${INPUT}
Restart=on-failure
RestartSec=2

[Install]
WantedBy=multi-user.target
EOF

echo "==> Tworze /usr/local/bin/tuxbar-toggle"
cat > /usr/local/bin/tuxbar-toggle <<'EOF'
#!/usr/bin/env bash
# tuxbar-toggle         - switch between TuxBar and tiny-dfr (stock Touch Bar)
# tuxbar-toggle on      - TuxBar now AND at every boot (tiny-dfr is masked)
# tuxbar-toggle off     - tiny-dfr now AND at every boot (TuxBar disabled)
# tuxbar-toggle status  - show what is active
# (boot-on / boot-off are kept as aliases of on / off)
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
    exec sudo "$0" "$@"
fi

use_tuxbar() {
    systemctl stop tiny-dfr.service || true
    # Masking stops udev from starting tiny-dfr on the Touch Bar at boot
    systemctl mask tiny-dfr.service
    systemctl enable tuxbar.service
    systemctl restart tuxbar.service
    echo "Aktywny: TuxBar (takze po restarcie)"
}

use_tinydfr() {
    systemctl disable --now tuxbar.service || true
    systemctl unmask tiny-dfr.service
    systemctl start tiny-dfr.service
    echo "Aktywny: tiny-dfr (takze po restarcie)"
}

case "${1:-toggle}" in
    on|boot-on)   use_tuxbar ;;
    off|boot-off) use_tinydfr ;;
    status)
        echo "tuxbar:   $(systemctl is-active tuxbar.service || true) / $(systemctl is-enabled tuxbar.service 2>&1 || true)"
        echo "tiny-dfr: $(systemctl is-active tiny-dfr.service || true) / $(systemctl is-enabled tiny-dfr.service 2>&1 || true)"
        ;;
    toggle)
        if systemctl is-active --quiet tuxbar.service; then
            use_tinydfr
        else
            use_tuxbar
        fi
        ;;
    *)
        echo "Uzycie: tuxbar-toggle [on|off|status]" >&2
        exit 1
        ;;
esac
EOF
chmod 755 /usr/local/bin/tuxbar-toggle

echo "==> Przeladowuje systemd"
systemctl daemon-reload

cat <<MSG

Gotowe.

  sudo tuxbar-toggle on         TuxBar teraz i po kazdym restarcie
  sudo tuxbar-toggle off        wroc do tiny-dfr (teraz i po restarcie)
  sudo tuxbar-toggle            przelacz miedzy nimi
  tuxbar-toggle status          co jest aktywne
  journalctl -u tuxbar -f       logi na zywo

Aby wlaczyc teraz:  sudo tuxbar-toggle on
MSG
INSTALLER_EOF
sudo bash "$INSTALLER" "$WORKDIR/target/release/tuxbar"
rm -f "$INSTALLER"

echo
echo "Gotowe. Wlacz teraz:  sudo tuxbar-toggle on"
echo "Autostart:            sudo tuxbar-toggle boot-on"
