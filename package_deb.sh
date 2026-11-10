#!/usr/bin/env bash
set -e

# ==============================================================================
# TuxBar .deb Package Generator
# Author: Juan José Monzón (@mylnzon58)
# ==============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DIST_DIR="$SCRIPT_DIR/dist"
PKG_DIR="/tmp/tuxbar_0.1.0_amd64"

echo "=== [1/4] Compilando binario optimizado release ==="
cd "$SCRIPT_DIR"
cargo build --release

echo "=== [2/4] Preparando estructura de directorios del paquete Debian ==="
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/local/bin"
mkdir -p "$PKG_DIR/etc/systemd/system"
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
    echo "[+] Restaurando tiny-dfr (Teclado F1-F12 estático)..."
    systemctl start tiny-dfr.service
    echo "✓ Touch Bar restaurada a modo tradicional (tiny-dfr)."
else
    echo "[-] Deteniendo tiny-dfr temporalmente..."
    systemctl stop tiny-dfr.service
    echo "[+] Activando TuxBar (Música en vivo, Galería y Widgets)..."
    systemctl start tuxbar.service
    echo "✓ TuxBar activo en la Touch Bar."
fi
EOF
chmod 755 "$PKG_DIR/usr/local/bin/tuxbar-toggle"

# 3. Copiar archivo de servicio systemd
install -m 644 "$SCRIPT_DIR/tuxbar.service" "$PKG_DIR/etc/systemd/system/tuxbar.service"

# 4. Control metadata
cat << 'EOF' > "$PKG_DIR/DEBIAN/control"
Package: tuxbar
Version: 0.1.0
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Juan José Monzón <mylnzon58@gmail.com>
Description: Pure-Rust dynamic Touch Bar engine for Apple T2 MacBooks on Linux
 TuxBar transforms the Apple Touch Bar into a live dynamic widget center:
 32-band Audiophile spectrum equalizer, 12-core CPU matrix & hardware telemetry,
 touch photo scrubber gallery, high-DPI typographic clock, and luminous touch glow.
Homepage: https://github.com/mylnzon58/tuxbar
EOF

# 5. postinst script
cat << 'EOF' > "$PKG_DIR/DEBIAN/postinst"
#!/bin/sh
set -e
systemctl daemon-reload
echo "[TuxBar] Instalación completada con éxito."
echo "[TuxBar] Ejecute 'sudo tuxbar-toggle' para activar los widgets dinámicos en la Touch Bar."
exit 0
EOF
chmod 755 "$PKG_DIR/DEBIAN/postinst"

# 6. prerm script
cat << 'EOF' > "$PKG_DIR/DEBIAN/prerm"
#!/bin/sh
set -e
systemctl stop tuxbar.service 2>/dev/null || true
systemctl start tiny-dfr.service 2>/dev/null || true
exit 0
EOF
chmod 755 "$PKG_DIR/DEBIAN/prerm"

echo "=== [3/4] Empaquetando con dpkg-deb ==="
DEB_FILE="$DIST_DIR/tuxbar_0.1.0_amd64.deb"
dpkg-deb --build "$PKG_DIR" "$DEB_FILE"
rm -rf "$PKG_DIR"

echo "=== [4/4] Paquete .deb generado con éxito ==="
ls -lh "$DEB_FILE"
echo "Para instalar: sudo apt install ./dist/tuxbar_0.1.0_amd64.deb (o doble clic)"
