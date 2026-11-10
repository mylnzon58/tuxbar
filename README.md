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
*Ecualizador de 32 bandas impulsado por audio PCM real de PipeWire (`@DEFAULT_AUDIO_SINK@`). Muestra insignia de decibeles reales (`dBFS`), retención de picos (*peak-hold*) y flatline instantáneo en pausa o silencio. Cero sonido emitido a los altavoces.*

### 📷 3. Modo Gallery (Photo Scrubber de Capturas e Imágenes)
![TuxBar Gallery Photo Scrubber Mode](assets/tuxbar_preview_gallery.png)
*Replica la función icónica de macOS Photos en Linux. Escanea capturas reales en `~/Pictures/Capturas de pantalla`, renderiza miniaturas de 84×54 px con bordes redondeados y permite deslizar con inercia táctil. Al presionar `[ ABRIR ]`, la imagen se lanza inmediatamente en el visor del escritorio GNOME.*

### 💻 4. Modo Cyberdeck (Matriz de Silicio Núcleo por Núcleo)
![TuxBar Cyberdeck Mode](assets/tuxbar_preview_cyberdeck.png)
*Matriz de telemetría individual hilo por hilo (hasta 16 hilos de CPU), temperatura del procesador en tiempo real en °C, uso de memoria RAM y monitor de actividad del kernel.*

### ⌨️ 5. Modo F1-F12 Keys (Auto-conmutación por Hardware con Tecla Fn Física)
![TuxBar Fn Keys Mode](assets/tuxbar_preview_fnkeys.png)
*13 teclas estándar completas de 58 px de altura con tipografía vectorial de alta definición. El listener de hardware monitorea el teclado interno de Apple: **al mantener pulsada la tecla física `Fn`, la barra cambia a las teclas de función F1-F12 en 0 ms y regresa a tus widgets al soltarla**.*

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
# Descargar el paquete v0.2.0
wget https://github.com/mylnzon58/tuxbar/releases/download/v0.2.0/tuxbar_0.2.0_amd64.deb

# Instalar y activar de inmediato
sudo dpkg -i tuxbar_0.2.0_amd64.deb
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
* **Teclado Virtual de Hardware (`/dev/uinput`):**
  Al presionar las teclas táctiles `Esc` o `F1` a `F12`, TuxBar emite eventos de hardware genuinos (`evdev::Key`) al kernel, garantizando compatibilidad con atajos de teclado, terminales, editores y entornos gráficos.

---

## 🍏 Compatibilidad con Hardware Apple T2

TuxBar ha sido diseñado para cualquier usuario de Linux en laptops Apple con chip de seguridad T2:

1. **Coexistencia Pacífica:** La unidad de servicio `tuxbar.service` declara `Conflicts=tiny-dfr.service`. Nunca habrá dos procesos intentando adquirir el dumb buffer DRM simultáneamente.
2. **Empaquetado Estándar:** El paquete `.deb` generado con `package_deb.sh` sigue la jerarquía Filesystem Hierarchy Standard (FHS) de Debian/Ubuntu y se instala limpiamente en cualquier distribución basada en Ubuntu/Debian.
3. **Soporte de Modelos:** Compatible con cualquier laptop Apple con Touch Bar reconocida por el kernel: MacBookPro15,1 / 15,2 / 16,1 / 16,2 / 16,3 / 16,4.

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
