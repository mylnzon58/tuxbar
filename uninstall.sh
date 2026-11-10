#!/usr/bin/env bash
set -e

# ==============================================================================
# TuxBar Uninstaller — Clean Restoration Script
# ==============================================================================

if [ "$EUID" -ne 0 ]; then
    echo "Privilegios de superusuario requeridos. Solicitando sudo..."
    exec sudo "$0" "$@"
fi

echo "=== Desinstalando TuxBar y restaurando tiny-dfr ==="

systemctl stop tuxbar.service 2>/dev/null || true
systemctl disable tuxbar.service 2>/dev/null || true

rm -f /etc/systemd/system/tuxbar.service
rm -f /usr/local/bin/tuxbar
rm -f /usr/local/bin/tuxbar-toggle

systemctl daemon-reload

echo "[+] Restaurando servicio predeterminado tiny-dfr..."
systemctl start tiny-dfr.service 2>/dev/null || true

echo "=== Desinstalación completada con éxito. Touch Bar restaurada a tiny-dfr ==="

