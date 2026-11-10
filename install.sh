#!/usr/bin/env bash
set -e

# ==============================================================================
# TuxBar Installer — The Dynamic Touch Bar Engine for Linux
# Author: Juan José Monzón (@mylnzon58)
# License: GPL-3.0-or-later
# ==============================================================================

GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}==========================================================================${NC}"
echo -e " 🚀 ${GREEN}TuxBar Installer — Dynamic Touch Bar Engine for Linux${NC}"
echo -e " Autor: Juan José Monzón (@mylnzon58)"
echo -e " Arquitectura: Pure-Rust DRM KMS + MPRIS D-Bus + Virtual uinput + Evdev"
echo -e "${CYAN}==========================================================================${NC}"

if [ "$EUID" -ne 0 ]; then
    echo -e "${YELLOW}[!] Privilegios de superusuario requeridos. Solicitando sudo...${NC}"
    exec sudo "$0" "$@"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 1. Verificar o compilar binario optimizado
echo -e "\n${CYAN}[1/4] Verificando binario TuxBar compilado en Rust...${NC}"
if [ ! -f "$SCRIPT_DIR/target/release/tuxbar" ]; then
    # Si se ejecuta mediante curl/pipe o no existe el binario local, intentar descargar release oficial
    if command -v curl >/dev/null 2>&1; then
        echo -e "${YELLOW}[i] Descargando paquete oficial precompilado v0.2.0 desde GitHub...${NC}"
        TMP_DIR=$(mktemp -d)
        if curl -sSL "https://github.com/mylnzon58/tuxbar/releases/download/v0.2.0/tuxbar_0.2.0_amd64.deb" -o "$TMP_DIR/tuxbar.deb" 2>/dev/null && [ -s "$TMP_DIR/tuxbar.deb" ]; then
            dpkg -i "$TMP_DIR/tuxbar.deb"
            rm -rf "$TMP_DIR"
            echo -e "${GREEN}[✓] TuxBar instalado y activado exitosamente mediante paquete oficial Debian.${NC}"
            exit 0
        fi
        rm -rf "$TMP_DIR"
    fi

    # Fallback: compilar con cargo si está disponible
    CARGO_BIN="$(command -v cargo 2>/dev/null || true)"
    if [ -z "$CARGO_BIN" ] && [ -n "$SUDO_USER" ]; then
        USER_HOME=$(getent passwd "$SUDO_USER" | cut -d: -f6)
        if [ -x "$USER_HOME/.cargo/bin/cargo" ]; then
            CARGO_BIN="$USER_HOME/.cargo/bin/cargo"
        fi
    fi
    if [ -n "$CARGO_BIN" ]; then
        echo -e "Compilando binario con $CARGO_BIN..."
        cd "$SCRIPT_DIR" && "$CARGO_BIN" build --release
    else
        echo -e "${RED}[X] Error: target/release/tuxbar no existe y no se encontró cargo.${NC}"
        exit 1
    fi
fi
echo -e "${GREEN}[✓] Binario TuxBar verificado con éxito (${SCRIPT_DIR}/target/release/tuxbar).${NC}"

# 2. Instalar binario en /usr/local/bin (deteniendo instancias previas limpiamente)
echo -e "\n${CYAN}[2/4] Instalando ejecutable en /usr/local/bin...${NC}"
systemctl kill -s SIGKILL tuxbar.service 2>/dev/null || true
pkill -9 -x tuxbar 2>/dev/null || true
systemctl stop tuxbar.service 2>/dev/null || true
install -m 755 "$SCRIPT_DIR/target/release/tuxbar" /usr/local/bin/tuxbar
echo -e "${GREEN}[✓] Ejecutable TuxBar instalado correctamente en /usr/local/bin/tuxbar.${NC}"

# 3. Crear script de conmutación instantánea (tuxbar-toggle)
echo -e "\n${CYAN}[3/4] Instalando conmutador rápido 'tuxbar-toggle'...${NC}"
cat << 'EOF' > /usr/local/bin/tuxbar-toggle
#!/usr/bin/env bash
# TuxBar Mode Switcher (Zero-Conflict Toggle between tiny-dfr and TuxBar)

if [ "$EUID" -ne 0 ]; then
    exec sudo "$0" "$@"
fi

if systemctl is-active --quiet tuxbar.service; then
    echo "[-] Desactivando TuxBar (Widgets Dinámicos)..."
    systemctl stop tuxbar.service
    echo "[+] Restaurando tiny-dfr (Modo tradicional)..."
    systemctl start tiny-dfr.service
    echo "✓ Touch Bar restaurada a tiny-dfr."
else
    echo "[-] Deteniendo tiny-dfr temporalmente..."
    systemctl stop tiny-dfr.service
    echo "[+] Activando TuxBar (Música, Telemetría, Teclas F1-F12 y Galería)..."
    systemctl start tuxbar.service
    echo "✓ TuxBar activo en la Touch Bar."
fi
EOF
chmod 755 /usr/local/bin/tuxbar-toggle
echo -e "${GREEN}[✓] tuxbar-toggle instalado con éxito.${NC}"

# 4. Configurar servicio systemd y activar inmediatamente
echo -e "\n${CYAN}[4/4] Configurando y activando TuxBar en la Touch Bar de inmediato...${NC}"
install -m 644 "$SCRIPT_DIR/tuxbar.service" /etc/systemd/system/tuxbar.service
systemctl daemon-reload
systemctl stop tiny-dfr.service 2>/dev/null || true
systemctl enable tuxbar.service 2>/dev/null || true
systemctl restart tuxbar.service
echo -e "${GREEN}[✓] ¡TuxBar v0.2.0 activado en la Touch Bar!${NC}"

echo -e "\n${GREEN}==========================================================================${NC}"
echo -e " ${GREEN}✓ ¡Instalación de TuxBar completada con éxito!${NC}"
echo -e " Ejecutable instalado en: ${CYAN}/usr/local/bin/tuxbar${NC}"
echo -e " Comando de alternancia:  ${CYAN}tuxbar-toggle${NC} (cambia entre TuxBar y tiny-dfr)"
echo -e " Comandos de control CLI: ${CYAN}tuxbar status${NC} | ${CYAN}tuxbar next${NC} | ${CYAN}tuxbar play-pause${NC}"
echo -e "${CYAN}--------------------------------------------------------------------------${NC}"
echo -e " 💙 ${GREEN}Mensaje del autor / Author's note:${NC}"
echo -e " Soy Juan José Monzón (@mylnzon58), desarrollador argentino y padre de una niña y un niño pequeño."
echo -e " Intento vivir de lo que amo (el desarrollo de software y tecnología), y quise crear"
echo -e " esta solución de forma abierta para dotar a la Touch Bar de widgets vivos en Linux."
echo -e ""
echo -e " ⭐ Si este proyecto le dio vida e interactividad a la Touch Bar de tu Mac, déjame una estrella en GitHub:"
echo -e " 👉 ${CYAN}https://github.com/mylnzon58/tuxbar${NC}"
echo -e " ¡Muchas gracias por tu apoyo! / Thank you for your support!"
echo -e "${CYAN}==========================================================================${NC}"
