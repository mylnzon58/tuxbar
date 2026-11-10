# TuxBar 🐧

**The Pure & Dynamic Touch Bar Engine for Linux | MPRIS Music, Hardware Telemetry & Extensible Widgets**

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-100%25_Pure_Safe-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![DRM](https://img.shields.io/badge/DRM-KMS_Zero_C_Deps-emerald.svg?style=flat-square&logo=linux)](https://en.wikipedia.org/wiki/Direct_Rendering_Manager)
[![Ubuntu 24.04](https://img.shields.io/badge/Ubuntu-24.04_LTS_Verified-E95420.svg?style=flat-square&logo=ubuntu)](https://ubuntu.com/)

> **Autor:** [Juan José Monzón (@mylnzon58)](https://github.com/mylnzon58)  
> **Hardware:** Apple MacBook Pro 16,1 (A2141) / Serie Apple T2 / Dispositivos con Touch Bar DRM  
> **Tecnología:** Rust 2021 · DRM KMS Dumb Buffers · D-Bus MPRIS · Telemetría `/proc` · Fontdue Typography

---

## 🇪🇸 Descripción en Español

Hasta hoy, usar Linux en una MacBook Pro con Touch Bar significaba conformarse con un teclado estático (`tiny-dfr`): botones fijos de volumen, brillo o teclas F1-F12 dibujadas en una pantalla OLED apagada e inerte.

**TuxBar transforma la Touch Bar en un centro de widgets interactivo, dinámico y vivo en tiempo real con 3 modos conmutables al tacto:**

![TuxBar Audiophile Mode](assets/tuxbar_audiophile_preview.png)
*Modo Audiophile: Reproductor MPRIS con Ecualizador de Espectro de 32 bandas dinámico y picos flotantes.*

![TuxBar Cyberdeck Mode](assets/tuxbar_cyberdeck_preview.png)
*Modo Cyberdeck: Matriz de carga CPU hilo por hilo (12 núcleos), RAM, temperatura y pulso de kernel.*

![TuxBar Gallery Photo Scrubber Mode](assets/tuxbar_gallery_preview.png)
*Modo Gallery: Visor y Scrubber de imágenes y capturas táctil (como en macOS Photos).*

![TuxBar Pro Split Mode](assets/tuxbar_prosplit_preview.png)
*Modo Pro Split: Vista balanceada con música, mini-espectro, telemetría y reloj tipográfico.*

1. 📷 **Modo Gallery (Photo Scrubber de Imágenes & Capturas):** Replica la función icónica de macOS Photos en Linux. Renderiza una tira de película (*filmstrip*) horizontal de fotos y capturas reales con bordes redondeados; permite deslizar el dedo sobre la Touch Bar para explorar miniaturas a 30 FPS y tocar `[ OPEN ]` para ver la imagen a resolución completa en el monitor principal.
2. 🎵 **Modo Audiophile (Ecualizador de Frecuencias en Vivo):** Integra un ecualizador dinámico de 32 bandas con degradado tricolor neón (Cyan base -> Magenta medio -> Rosa picos) y retención de picos (*peak-hold*), reproductor MPRIS (Spotify, Chrome, Firefox, VLC) con desplazamiento *marquee* y controles táctiles directos (`◀◀`, `▶/❚❚`, `▶▶`).
3. ⚡ **Modo Cyberdeck (Matriz de Silicio en Tiempo Real):** Muestra el nivel de carga individual de cada uno de los núcleos lógicos del procesador (hasta 16 hilos) mediante medidores verticales estilo LED, temperatura del silicio en °C, uso de RAM y monitor de latido del kernel.
4. ⏱️ **Modo Pro Split (Productividad Equilibrada):** Combina el reproductor musical con medidor de espectro compacto, telemetría esencial de hardware y reloj tipográfico de alta definición.
5. ✨ **Respuesta Táctil Visual (Luminous Touch Glow):** Al posar el dedo sobre la pantalla OLED, un halo de luz neón sigue las coordenadas exactas de la pulsación sobre el cristal.
6. 🔄 **Selector de Modos al Tacto:** Botón interactivo `[ MODE ]` directo en la Touch Bar que conmuta en caliente entre los 4 modos.
7. 🔀 **Conmutación Instantánea (`tuxbar-toggle`):** Alterna al instante entre TuxBar y el teclado tradicional (`tiny-dfr`) sin reiniciar la laptop.

---

## 🏛️ Arquitectura de Software y Principios de Ingeniería

TuxBar fue diseñado desde cero siguiendo estrictos estándares de ingeniería de software para garantizar **máximo rendimiento, legibilidad y seguridad absoluta del hardware**:

* **Principios SOLID:**
  * **S (Single Responsibility):** Módulos desacoplados; `drm.rs` gestiona buffers KMS, `input.rs` captura coordenadas táctiles, `mpris.rs` comunica con D-Bus y `telemetry.rs` lee `/proc`.
  * **O (Open/Closed):** Sistema extensible de widgets mediante el trait `Widget`. Se pueden agregar nuevos componentes visuales sin modificar el ciclo de renderizado central.
  * **L (Liskov Substitution):** Cualquier widget que implemente `Widget` encaja de manera transparente en la cuadrícula de diseño.
  * **I (Interface Segregation):** Separación limpia entre la lógica de dibujado (`render`) y la gestión de pulsaciones (`handle_touch`).
  * **D (Dependency Inversion):** El orquestador de diseño depende de la abstracción `Widget`, no de implementaciones concretas.
* **Cero Dependencias C (Pure Rust):**
  A diferencia de proyectos tradicionales que dependen de complejas librerías C externas (Cairo, FreeType C, librsvg C), TuxBar compila directamente contra el subsistema del kernel de Linux utilizando crates puros de Rust (`drm`, `evdev`, `fontdue`, `zbus`). Esto elimina problemas de compatibilidad de librerías dinámicas y fugas de memoria.
* **Seguridad de Hardware y OWASP:**
  * **Protección del Panel OLED:** La comunicación con la pantalla se realiza exclusivamente a través de la API estándar de dumb buffers del kernel de Linux (`DRM_IOCTL_MODE_CREATE_DUMB`). El controlador del kernel (`appletbdrm`) garantiza que los voltajes y frecuencias de refresco se mantengan siempre dentro de las especificaciones seguras de fábrica.
  * **Sin Inyección de Comandos:** No se ejecutan comandos de shell (`sh -c`). Toda la comunicación con reproductores y el sistema se realiza mediante llamadas directas a APIs del kernel y D-Bus tipado.
  * **Límites de Memoria Estrictos:** Verificación continua de límites en cada píxel del buffer de 2008x60 para evitar desbordamientos de búfer.

---

## 🚀 Instalación Rápida (1 Paso)

```bash
# Clonar el repositorio
git clone https://github.com/mylnzon58/tuxbar.git
cd tuxbar

# Instalar y compilar con 1 comando
sudo ./install.sh
```

El script de instalación compilará el binario optimizado, instalará el servicio `tuxbar.service` y configurará el comando de conmutación `tuxbar-toggle`.

---

## 🎮 Comandos y Uso

| Comando | Acción |
| :--- | :--- |
| **`sudo tuxbar-toggle`** | Alterna al instante entre **TuxBar (Widgets)** y **tiny-dfr (Teclado F1-F12)** |
| **`tuxbar status`** | Muestra en terminal la pista musical actual y la telemetría del sistema |
| **`tuxbar play-pause`** | Envía comando de reproducir/pausar al reproductor activo |
| **`tuxbar next`** | Salta a la siguiente canción |
| **`tuxbar previous`** | Vuelve a la canción anterior |
| **`tuxbar test`** | Verifica el motor gráfico y la tipografía en memoria sin capturar la pantalla |

---

## 🇬🇧 English Description

Until today, using Linux on a MacBook Pro with a Touch Bar meant settling for a static keyboard experience (`tiny-dfr`): fixed volume and brightness buttons or F1-F12 keys rendered on an otherwise dormant OLED screen.

**TuxBar turns the Touch Bar into an interactive, real-time dynamic widget engine:**

* 🎵 **Live MPRIS Media Display:** Automatically binds to Spotify, YouTube, Chrome, Firefox, or VLC. Shows player pill, track title and artist with smooth marquee scrolling, touch playback controls (`Prev`, `Play/Pause`, `Next`), and progress bar.
* ⚡ **Real-Time Hardware Telemetry:** Displays CPU usage %, live temperature in °C with color status (green <70°C, yellow <85°C, red >85°C), and RAM utilization directly from `/proc` with zero CPU overhead.
* 🕒 **High-DPI Typography Clock:** Crisp time and date rendered using a pure-Rust vector rasterizer.
* 🔄 **Instant Toggle (`tuxbar-toggle`):** Hot-switch between dynamic widgets and the classic function keyboard without rebooting.

---

## 💙 Mensaje del autor / Author's note

Soy **Juan José Monzón** ([@mylnzon58](https://github.com/mylnzon58)), desarrollador argentino y padre de una niña y un niño pequeño. Intento vivir de lo que amo (el desarrollo de software y tecnología), y quise crear esta solución de forma abierta para dotar a la Touch Bar de widgets vivos en Linux.

⭐ **Si este proyecto le dio vida e interactividad a la Touch Bar de tu Mac, déjame una estrella en GitHub:**  
👉 **[https://github.com/mylnzon58/tuxbar](https://github.com/mylnzon58/tuxbar)**

¡Muchas gracias por tu apoyo! / *Thank you for your support!*

