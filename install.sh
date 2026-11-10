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
echo -e " Arquitectura: Pure-Rust DRM KMS + MPRIS D-Bus + Hardware Telemetry"
echo -e "${CYAN}==========================================================================${NC}"

if [ "$EUID" -ne 0 ]; then
    echo -e "${YELLOW}[!] Privilegios de superusuario requeridos. Solicitando sudo...${NC}"
    exec sudo "$0" "$@"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# 1. Compilar binario optimizado si no existe
echo -e "\n${CYAN}[1/4] Verificando compilación del binario en Rust...${NC}"
if [ ! -f "$SCRIPT_DIR/target/release/tuxbar" ]; then
    echo -e "Compilando binario de alto rendimiento con cargo release..."
    cd "$SCRIPT_DIR" && cargo build --release
fi

echo -e "${GREEN}[✓] Binario TuxBar verificado con éxito.${NC}"

# 2. Instalar binario en /usr/local/bin
echo -e "\n${CYAN}[2/4] Instalando ejecutable en el sistema...${NC}"
install -m 755 "$SCRIPT_DIR/target/release/tuxbar" /usr/local/bin/tuxbar

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
    echo "[+] Restaurando tiny-dfr (Teclado F1-F12 estático)..."
    systemctl start tiny-dfr.service
    echo "✓ Touch Bar restaurada a modo tradicional (tiny-dfr)."
else
    echo "[-] Deteniendo tiny-dfr temporalmente..."
    systemctl stop tiny-dfr.service
    echo "[+] Activando TuxBar (Música en vivo, Telemetría y Widgets)..."
    systemctl start tuxbar.service
    echo "✓ TuxBar activo en la Touch Bar."
fi
EOF
chmod 755 /usr/local/bin/tuxbar-toggle

# 4. Configurar servicio systemd
echo -e "\n${CYAN}[4/4] Configurando servicio del sistema tuxbar.service...${NC}"
install -m 644 "$SCRIPT_DIR/tuxbar.service" /etc/systemd/system/tuxbar.service
systemctl daemon-reload

echo -e "\n${GREEN}==========================================================================${NC}"
echo -e " ${GREEN}✓ ¡Instalación de TuxBar completada con éxito!${NC}"
echo -e " Ejecutable instalado en: ${CYAN}/usr/local/bin/tuxbar${NC}"
echo -e " Comando de alternancia:  ${CYAN}tuxbar-toggle${NC} (cambia entre TuxBar y teclado estándar)"
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
