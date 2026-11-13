#!/usr/bin/env bash
set -e

# ==============================================================================
# TuxBar .deb Package Generator
# Author: Juan José Monzón (@mylnzon58)
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="$SCRIPT_DIR/dist"
PKG_DIR="/tmp/tuxbar_0.2.7_amd64"

echo "=== [1/4] Compilando binario optimizado release ==="
cd "$SCRIPT_DIR"
cargo build --release

echo "=== [2/4] Preparando estructura de directorios del paquete Debian ==="
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/local/bin"
mkdir -p "$PKG_DIR/etc/systemd/system"
mkdir -p "$PKG_DIR/lib/systemd/system-sleep"
mkdir -p "$DIST_DIR"

# 1. Copiar binario ejecutable
install -m 755 "$SCRIPT_DIR/target/release/tuxbar" "$PKG_DIR/usr/local/bin/tuxbar"

# 2. Copiar script de conmutación
cat << 'EOF' > "$PKG_DIR/usr/local/bin/tuxbar-toggle"
#!/usr/bin/env bash
if [ "$EUID" -ne 0 ]; then
    exec sudo "$0" "$@"
fi
if systemctl is-active --quiet tuxbar.service; then
    echo "[-] Desactivando TuxBar (Widgets Dinámicos)..."
    systemctl stop tuxbar.service
    echo "[+] Restaurando tiny-dfr (Modo tradicional)..."
    systemctl unmask tiny-dfr.service 2>/dev/null || true
    systemctl start tiny-dfr.service
    echo "✓ Touch Bar restaurada a tiny-dfr."
else
    echo "[-] Deteniendo y enmascarando tiny-dfr para evitar conflictos udev..."
    systemctl stop tiny-dfr.service 2>/dev/null || true
    systemctl mask tiny-dfr.service 2>/dev/null || true
    echo "[+] Activando TuxBar (Música, Telemetría, Teclas F1-F12 y Galería)..."
    systemctl start tuxbar.service
    echo "✓ TuxBar activo en la Touch Bar."
fi
EOF
chmod 755 "$PKG_DIR/usr/local/bin/tuxbar-toggle"

# 3. Copiar archivo de servicio systemd y hook de suspensión/reposo
install -m 644 "$SCRIPT_DIR/tuxbar.service" "$PKG_DIR/etc/systemd/system/tuxbar.service"

cat << 'EOF' > "$PKG_DIR/lib/systemd/system-sleep/tuxbar-sleep"
#!/bin/sh
# Reinicia TuxBar limpiamente al salir de suspensión para re-enlazar DRM y Touch input
case "$1" in
    post)
        if systemctl is-enabled --quiet tuxbar.service 2>/dev/null; then
            sleep 1
            systemctl restart tuxbar.service 2>/dev/null || true
        fi
        ;;
esac
EOF
chmod 755 "$PKG_DIR/lib/systemd/system-sleep/tuxbar-sleep"

# 4. Control metadata
cat << 'EOF' > "$PKG_DIR/DEBIAN/control"
Package: tuxbar
Version: 0.2.7
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Juan José Monzón <mylnzon58@gmail.com>
Description: Pure-Rust dynamic Touch Bar engine for Apple T2 MacBooks on Linux
 TuxBar transforms the Apple Touch Bar into an interactive widget engine:
 32-band Audiophile equalizer, 12-core CPU telemetry matrix, full-height
 F1-F12 keys with hardware Fn integration, photo filmstrip scrubber,
 Control Deck system shortcuts (brightness, mic, volume, calc, browser, files, spaces),
 and high-DPI typographic clock.
Homepage: https://github.com/mylnzon58/tuxbar
EOF

# 5. postinst script
cat << 'EOF' > "$PKG_DIR/DEBIAN/postinst"
#!/bin/sh
set -e
systemctl daemon-reload
echo "[TuxBar] Deteniendo y enmascarando tiny-dfr para evitar conflictos al iniciar o salir de reposo..."
systemctl stop tiny-dfr.service 2>/dev/null || true
systemctl disable tiny-dfr.service 2>/dev/null || true
systemctl mask tiny-dfr.service 2>/dev/null || true
echo "[TuxBar] Activando TuxBar en la Touch Bar de forma inmediata..."
systemctl enable tuxbar.service 2>/dev/null || true
systemctl restart tuxbar.service 2>/dev/null || true
echo "[TuxBar] ¡TuxBar ha sido instalado y activado en tu Touch Bar!"
echo "[TuxBar] Usa 'sudo tuxbar-toggle' cuando quieras alternar entre widgets y tiny-dfr."
exit 0
EOF
chmod 755 "$PKG_DIR/DEBIAN/postinst"

# 6. prerm script
cat << 'EOF' > "$PKG_DIR/DEBIAN/prerm"
#!/bin/sh
set -e
systemctl stop tuxbar.service 2>/dev/null || true
systemctl unmask tiny-dfr.service 2>/dev/null || true
systemctl enable tiny-dfr.service 2>/dev/null || true
systemctl start tiny-dfr.service 2>/dev/null || true
exit 0
EOF
chmod 755 "$PKG_DIR/DEBIAN/prerm"

echo "=== [3/4] Empaquetando con dpkg-deb ==="
DEB_FILE="$DIST_DIR/tuxbar_0.2.7_amd64.deb"
dpkg-deb --build "$PKG_DIR" "$DEB_FILE"
rm -rf "$PKG_DIR"

echo "=== [4/4] Paquete .deb generado con éxito ==="
ls -lh "$DEB_FILE"
echo "Para instalar: sudo apt install ./dist/tuxbar_0.2.7_amd64.deb (o doble clic)"
