# TuxBar 🐧

**The Pure & Dynamic Touch Bar Engine for Linux | MPRIS Music, Hardware Telemetry & Extensible Widgets**

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-100%25_Pure_Safe-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![DRM](https://img.shields.io/badge/DRM-KMS_Zero_C_Deps-emerald.svg?style=flat-square&logo=linux)](https://en.wikipedia.org/wiki/Direct_Rendering_Manager)
[![Ubuntu 24.04](https://img.shields.io/badge/Ubuntu-24.04_LTS_Verified-E95420.svg?style=flat-square&logo=ubuntu)](https://ubuntu.com/)
[![Web / Live Previews](https://img.shields.io/badge/Documentaci%C3%B3n-GitHub_Pages-77216F.svg?style=flat-square)](https://mylnzon58.github.io/tuxbar/)

> **Autor:** [Juan José Monzón (@mylnzon58)](https://github.com/mylnzon58)  
> **Hardware:** Apple MacBook Pro 16,1 (A2141) / Serie Apple T2 / Laptops con Touch Bar DRM (`appletbdrm`)  
> **Tecnología:** Rust 2021 · DRM KMS Dumb Buffers · D-Bus MPRIS · PipeWire PCM Telemetry · Evdev Multi-Touch · Virtual `/dev/uinput`  
> **Sitio Web Oficial & Capturas:** [https://mylnzon58.github.io/tuxbar/](https://mylnzon58.github.io/tuxbar/)

---

## 🇪🇸 Descripción en Español

Hasta hoy, utilizar Linux en una MacBook Pro con Touch Bar significaba conformarse con una hilera estática (`tiny-dfr`): botones fijos de volumen y brillo o teclas F1-F12 dibujadas en una pantalla OLED inerte y desaprovechada.

**TuxBar transforma la Touch Bar en un centro de widgets interactivo, dinámico y en tiempo real con 5 modos diseñados a 60 píxeles de altura:**

### ⚡ 1. Modo Pro Split (Equilibrio y Productividad)
![TuxBar Pro Split Mode](assets/tuxbar_preview_prosplit.png)
*Vista balanceada: reproductor musical MPRIS con título y artista, mini-espectro dinámico, matriz de telemetría de silicio y reloj tipográfico de alta definición.*

### 🎵 2. Modo Audiophile (Ecualizador Reactivo en Vivo con Decibeles Reales)
![TuxBar Audiophile Mode](assets/tuxbar_preview_audiophile.png)
*Ecualizador de 32 bandas impulsado por audio PCM real de PipeWire (`@DEFAULT_AUDIO_SINK@`). Incorpora escala cromática Heatmap de 10 segmentos (Cian -> Verde -> Lima -> Ámbar -> Naranja -> Carmesí), indicador dinámico de decibeles reales (`dBFS`) con coloración por potencia, retención de picos flotantes (*peak-hold*) y flatline instantáneo en silencio. Cero sonido emitido a los altavoces.*

### 📷 3. Modo Gallery (Photo Scrubber de Capturas e Imágenes)
![TuxBar Gallery Photo Scrubber Mode](assets/tuxbar_preview_gallery.png)
*Replica la función icónica de macOS Photos en Linux. Escanea capturas reales en `~/Pictures/Capturas de pantalla`, renderiza miniaturas de 84×54 px con bordes redondeados y permite deslizar con inercia táctil. Al presionar `[ ABRIR ]` o realizar doble toque en una foto, la imagen se abre instantáneamente en el visor de imágenes nativo de tu escritorio GNOME (Eye of GNOME / Loupe).*

### 💻 4. Modo Cyberdeck (Matriz de Silicio Núcleo por Núcleo)
![TuxBar Cyberdeck Mode](assets/tuxbar_preview_cyberdeck.png)
*Matriz de telemetría individual hilo por hilo (hasta 16 hilos de CPU), temperatura del procesador en tiempo real en °C, uso de memoria RAM y monitor de actividad del kernel.*

### ⌨️ 5. Modo F1-F12 Keys (Auto-conmutación por Hardware con Tecla Fn Física)
![TuxBar Fn Keys Mode](assets/tuxbar_preview_fnkeys.png)
*13 teclas estándar completas de 58 px de altura con bordes físicos resaltados de 2 px (estilo keycap mecánico) y tipografía vectorial de alta definición. Esc resaltado en carmesí y F1-F12 en acero reflectante. El listener de hardware monitorea el teclado interno de Apple: **al mantener pulsada la tecla física `Fn`, la barra cambia a las teclas de función F1-F12 en 0 ms y regresa a tus widgets al soltarla**.*

### 🎛️ 6. Modo Control Deck (Centro de Control Iconográfico & Accesos Rápidos)
![TuxBar Control Deck Mode](assets/tuxbar_preview_controldeck.png)
*15 botones táctiles de 114×56 px con diseño iconográfico vectorial cromático y triple despacho (Kernel uinput + Sysfs directo + Puente D-Bus `systemd-run --user`):*
* **ESC:** Salida de emergencia en hardware con distintivo circular.
* **BRILLO PANTALLA - / +:** Regulación suave de retroiluminación con iconos solar y OSD nativo de GNOME.
* **BRILLO TECLADO - / +:** Control táctil de iluminación del teclado físico (`/sys/class/leds/*kbd_backlight*` y `KEY_KBDILLUMDOWN`/`UP`).
* **MIC MUTE:** Silenciamiento de micrófono por hardware y PipeWire (`wpctl`) con icono de cápsula y soporte.
* **VOL - / + / MUTE:** Control acústico de volumen con conos de altavoz, ondas sonoras dinámicas y cruz de silenciamiento nítida.
* **CALCULADORA:** Apertura inmediata de la Calculadora con icono vectorial de pantalla y matriz numérica.
* **WEB:** Apertura del navegador predeterminado con icono de globo terráqueo y meridianos.
* **CARPETAS (FILES):** Acceso directo al Explorador de Archivos (Nautilus) con icono de carpeta y solapa.
* **ESCRITORIOS (DESKS):** Vista general de todos los escritorios y espacios de trabajo (GNOME Activities Overview) con icono de cuadrícula multiventana.
* **MEDIA ⏯ & NEXT ⏭:** Controles multimedia universales con pulsos uinput de 25 ms.

---

## 🌟 Novedades y Mejoras Técnicas en la Versión v0.2.1

La versión **v0.2.1** consolida la madurez de TuxBar como un entorno táctil interactivo de nivel de producción:

* 🖼️ **Puente Desacoplado de Sesión Gráfica (`systemd-run --user`):**
  Lanzamiento instantáneo de imágenes y capturas desde la barra táctil hacia el escritorio activo de GNOME (Wayland y X11). Al presionar `[ ABRIR ]` o dar doble toque a una miniatura, TuxBar delega la invocación al gestor de sesión del usuario con variables completas de entorno (`HOME`, `WAYLAND_DISPLAY`, `DBUS`), priorizando el visor nativo (`/usr/bin/eog` o `loupe`) y eliminando bloqueos de sandbox en daemons de sistema.
* 🌈 **Gradiente Térmico Heatmap en Espectro (Studio VU Meter):**
  El ecualizador de 32 bandas y el mini-espectro integran una escala cromática de 10 segmentos con transición gradual según la potencia del audio:
  - `0 - 1`: Azul Cian Eléctrico *(Sub-graves)*
  - `2 - 3`: Verde Neón *(Medios-bajos)*
  - `4 - 5`: Cyber Lime *(Medios)*
  - `6 - 7`: Ámbar Dorado *(Medios-altos)*
  - `8`: Naranja de Alta Potencia *(Agudos enérgicos)*
  - `9`: Rojo Carmesí *(Picos de techo)*  
  Incorpora puntos flotantes dinámicos de retención de picos (*peak-hold*) y badge dinámico de decibeles con alerta de saturación de potencia.
* ⏭️ **Control Multimedia Universal con Pulso de Retención de 25 ms:**
  Solución definitiva al avance y retroceso de pista (`KEY_NEXTSONG`, `KEY_PREVIOUSSONG`, `KEY_PLAYPAUSE`): TuxBar introduce un pulso de retención de 25 ms en `/dev/uinput` y difusión dual por D-Bus MPRIS. Los cambios de canción responden con 100% de efectividad en Spotify, YouTube (en Chrome/Chromium/Brave), Firefox y reproductores nativos.
* 🏷️ **Cápsula de Metadatos de Galería con Recorte Anticolisión:**
  El título de la foto y el contador (`X/Y`) se agrupan en una tarjeta encapsulada con recorte tipográfico dinámico (`draw_text_clipped`), evitando colisiones o solapamientos con la tira filmstrip de miniaturas.
* 🔒 **Auditoría Estricta de Privacidad y Portabilidad Universal:**
  Eliminación total de rutas personales o nombres de usuario hardcodeados. Toda la sesión, rutas de imágenes y bus D-Bus se descubren en tiempo de ejecución a través de llamadas de bajo nivel a la API POSIX `libc::getpwuid(target_uid)`.

---

## 🚀 Instalación Rápida (Sin Reiniciar)

TuxBar ha sido empaquetado para instalarse y activarse de inmediato en la Touch Bar **sin necesidad de reiniciar tu laptop**:

### Opción 1: Un solo comando en Terminal (Recomendado)
```bash
curl -sSL https://raw.githubusercontent.com/mylnzon58/tuxbar/main/install.sh | sudo bash
```

### Opción 2: Paquete Debian Nativo (`.deb`)
Descarga el paquete precompilado e instálalo con `dpkg` o `apt`:
```bash
# Descargar el paquete v0.2.5
wget https://github.com/mylnzon58/tuxbar/releases/download/v0.2.5/tuxbar_0.2.5_amd64.deb

# Instalar y activar de inmediato
sudo dpkg -i tuxbar_0.2.5_amd64.deb
```

### Opción 3: Compilación desde el Código Fuente
```bash
git clone https://github.com/mylnzon58/tuxbar.git
cd tuxbar
cargo build --release
sudo ./install.sh
```

---

## 🔀 Conmutación en Caliente (`tuxbar-toggle`)

¿Necesitas volver temporalmente a las teclas estáticas de `tiny-dfr` para una sesión de trabajo específica? TuxBar incluye un conmutador sin fricción:

```bash
sudo tuxbar-toggle
```
*Alterna de forma segura entre **TuxBar (Widgets)** y **tiny-dfr (Teclado Tradicional)** en 0.2 segundos sin reiniciar ni apagar el equipo.*

---

## 🎮 Comandos de Control CLI

| Comando | Acción |
| :--- | :--- |
| **`sudo tuxbar-toggle`** | Alterna al instante entre **TuxBar (Widgets)** y **tiny-dfr (Teclado F1-F12)** |
| **`tuxbar status`** | Muestra en terminal la pista activa (MPRIS), decibeles y espectro de audio (PipeWire), uso de CPU por núcleo y RAM |
| **`tuxbar play-pause`** | Envía comando Play/Pause por D-Bus al reproductor activo en el escritorio |
| **`tuxbar next`** | Salta a la siguiente canción en Spotify, Chrome o VLC |
| **`tuxbar previous`** | Regresa a la canción anterior |
| **`tuxbar test`** | Verifica el motor gráfico y la tipografía en memoria sin capturar la pantalla |

---

## 🏛️ Arquitectura Técnica y Estándares de Ingeniería

TuxBar fue diseñado bajo estrictos estándares de ingeniería de software para garantizar **máximo rendimiento, legibilidad y seguridad absoluta del hardware**:

* **Cero Dependencias C (Pure Rust):**
  A diferencia de proyectos tradicionales que dependen de complejas librerías C externas (Cairo, FreeType C, librsvg C), TuxBar compila directamente contra el subsistema del kernel de Linux utilizando crates puros de Rust (`drm`, `evdev`, `fontdue`, `zbus`). Esto elimina incompatibilidades de bibliotecas dinámicas y fugas de memoria.
* **Seguridad de Hardware y Protección del Panel OLED:**
  La comunicación con la pantalla se realiza exclusivamente mediante la API estándar de dumb buffers del kernel (`DRM_IOCTL_MODE_CREATE_DUMB`) en `/dev/dri/card*` administrado por `appletbdrm`. Los voltajes y frecuencias de refresco se mantienen siempre dentro de los parámetros seguros de fábrica.
* **Audio Telemetry Pasivo (PipeWire PCM):**
  `AudioCaptureService` monitorea pasivamente el flujo de audio de salida a 8000 Hz / 16-bit mono desde `@DEFAULT_AUDIO_SINK@`. No emite ni reproduce sonidos a los altavoces, evitando ruidos o sobrecargas en los amplificadores del MacBook Pro.
* **Teclado Virtual de Hardware (`/dev/uinput`) & Temporización Real:**
  Al presionar las teclas táctiles `Esc`, `F1` a `F12` o controles de música (`Next`, `Prev`, `PlayPause`), TuxBar emite eventos de hardware genuinos (`evdev::Key`) al kernel con un pulso de retención de 25 ms entre KeyDown y KeyUp, garantizando compatibilidad absoluta con compositores Wayland, terminales y navegadores (Chrome, Firefox, Spotify).
* **Puente de Sesión Gráfica y Desacoplamiento de Daemon (`systemd-run --user`):**
  Para interactuar con el escritorio activo sin comprometer la seguridad ni sufrir bloqueos de permisos (`ProtectHome`), TuxBar delega la apertura de archivos al gestor de sesión `systemd --user` del usuario activo resuelto mediante `libc::getpwuid`. Esto permite lanzar visores gráficos nativos (`eog`, `loupe`) con acceso total a sockets Wayland y D-Bus.

---

## 🍏 Matriz de Compatibilidad & Hardware Comprobado

### 🔬 Equipo Oficial de Pruebas y Laboratorio (100% Funcional)
* **Laptop:** Apple MacBook Pro 16 pulgadas (2019)
* **Identificador de Modelo:** `MacBookPro16,1` (Modelo A2141)
* **Chip de Seguridad:** Apple T2 SoC
* **Procesador:** Intel Core i7 / i9 (9na Generación)
* **Panel OLED Touch Bar:** 2008 × 60 píxeles @ 60 Hz
* **Sistema Operativo:** Ubuntu 24.04 LTS (Noble Numbat)
* **Kernel de Linux:** `Linux 7.3.0-4-t2-noble` (con parches DRM para Apple)
* **Subsistema de Audio:** PipeWire 1.0+ (captura pasiva en `@DEFAULT_AUDIO_SINK@`)
* **Sesión Gráfica:** Wayland y X11 (GNOME Desktop)

### 💻 Modelos de MacBook Compatibles
Soporta cualquier MacBook con pantalla Touch Bar reconocida por el controlador del kernel `appletbdrm`:
* **MacBook Pro 16" (2019):** `MacBookPro16,1`, `MacBookPro16,4`
* **MacBook Pro 15" (2016 - 2019):** `MacBookPro15,1`, `MacBookPro15,3`, `MacBookPro14,3`, `MacBookPro13,3`
* **MacBook Pro 13" (2016 - 2020):** `MacBookPro16,2`, `MacBookPro16,3`, `MacBookPro15,2`, `MacBookPro14,2`, `MacBookPro13,2`

### 🐧 Distribuciones Linux Soportadas
* **Ubuntu 24.04 LTS:** Comprobado y verificado en hardware de laboratorio.
* **Ubuntu 22.04 LTS:** Compatible mediante paquete nativo `.deb`.
* **Debian 12 / Linux Mint / Pop!_OS:** Compatible mediante paquete nativo `.deb`.
* **Fedora / Arch Linux / Manjaro:** Compatible mediante compilación con `cargo build --release` (requiere kernel con soporte `appletbdrm`).

---

## 🤖 Declaración de Transparencia: Desarrollo con Inteligencia Artificial

> [!NOTE]
> **Aviso de transparencia tecnológica:**  
> Este proyecto ha sido programado y optimizado utilizando herramientas avanzadas de Inteligencia Artificial (no apto para puristas de la programación artesanal). Sin embargo, cada línea de código, cada ioctl de DRM KMS y cada cálculo de frecuencia de PipeWire ha sido sometido a un **riguroso proceso de ingeniería humana y pruebas directas en hardware físico Apple MacBook Pro 16,1 (A2141) bajo Ubuntu 24.04 LTS Kernel 7.3.0-4-t2-noble**.
>
> El código es 100% libre y abierto (GPL-3.0). Si encuentras margen de mejora, casos extremos o deseas proponer nuevos widgets, seré el primero en recibir tus aportes, issues y Pull Requests con total amabilidad y apertura. Creemos en el software libre que resuelve problemas concretos y aporta a la comunidad Linux.

---

## 💙 Mensaje del Autor / Author's Note

Soy **Juan José Monzón** ([@mylnzon58](https://github.com/mylnzon58)), desarrollador de software argentino y padre de una niña y un niño pequeño 🇦🇷👨‍👧‍👦. Intento vivir de lo que amo: el desarrollo de productos digitales, la tecnología y el software que aporta soluciones reales.

Quise crear esta solución de forma abierta para que cualquier persona que instale Linux en su MacBook pueda disfrutar de una Touch Bar viva, útil y moderna.

⭐ **Si este proyecto le devolvió la utilidad a la Touch Bar de tu Mac, apóyame con una estrella en GitHub:**  
👉 **[https://github.com/mylnzon58/tuxbar](https://github.com/mylnzon58/tuxbar)**

☕ **Si deseas apoyar mi trabajo y a mi familia para seguir manteniendo proyectos de código libre:**  
👉 **[Donar un Tecito en tecito.app/juanjoclassic](https://tecito.app/juanjoclassic)**

* ¡Conectemos en [LinkedIn](https://www.linkedin.com/in/juan-m-97b85681/)
* Sígueme en [Instagram (@juanjoclassic)](https://www.instagram.com/juanjoclassic/)

¡Muchas gracias por tu apoyo! / *Thank you for your support!*
