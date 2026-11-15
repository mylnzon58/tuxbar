#!/usr/bin/env bash
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
