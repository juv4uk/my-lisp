# ⚡ Signal & Radio IDE

<p align="center">
  <img src="docs/hero.svg" alt="Signal & Radio IDE Banner" width="100%">
</p>

<p align="center">
  <a href="https://github.com/juv4uk/my-ide/releases"><img src="https://img.shields.io/github/v/release/juv4uk/my-ide?color=blue&label=Latest%20Release" alt="Release"></a>
  <a href="https://github.com/juv4uk/my-ide/releases"><img src="https://img.shields.io/github/downloads/juv4uk/my-ide/total?color=brightgreen&label=Downloads" alt="Downloads"></a>
  <a href="https://v2.tauri.app/"><img src="https://img.shields.io/badge/Tauri-v2-blue.svg" alt="Tauri v2"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-1.75+-orange.svg" alt="Rust"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-yellow.svg" alt="License: MIT"></a>
</p>

---

## 🌍 Languages / Мови / Sprachen
* [English](#-english)
* [Українська](#-українська)
* [Deutsch](#-deutsch)

---

## 📊 Platform Build Status / Стан збірки платформ

| Platform / Платформа | Target / Формат | Status / Статус |
|---|---|---|
| 🪟 **Windows** | `.exe`, `.msi` | 🟢 **Automated in Releases** |
| 🐧 **Linux x86_64** | `.deb`, `.AppImage`, `.rpm` | 🟢 **Automated in Releases** |
| 🍏 **macOS** | `.dmg` *(Universal: M-Series & Intel)* | 🟢 **Automated in Releases** |
| 🤖 **Android** | `.apk` *(AArch64)* | 🟢 **Automated in Releases** |
| 📱 **iOS** | `.app` *(Simulator, Xcode)* | 🟢 **Automated in Releases** |
| 🥧 **ARM Linux** | `.deb`, `.AppImage`, `.rpm` *(AArch64, Raspberry Pi)* | 🟢 **Automated in Releases** |
| 📦 **Flatpak** | `.flatpak` | 🟡 **Automated in Releases** |
| 🌐 **Web / WASM** | `.zip` *(static hosting)* | 🟢 **Automated in Releases** |

---

## 🇬🇧 English

### Overview
**Signal & Radio IDE** is a high-performance, lightweight, cross-platform Integrated Development Environment designed for custom DSL parsing, radio signal path visualization, RF circuit schematic rendering, and amateur radio ADIF log management.

Built with safety, speed, and modern UX in mind using **Tauri v2, Rust, SvelteKit, CodeMirror 6, and Mermaid.js**. Runs natively on Windows, Linux, macOS, Android, iOS, and Raspberry Pi (ARM64).

### Key Features
* **⚡ Rust-Powered Core:** Blazing fast parsing for custom `#graph` and `#adif` domain-specific languages.
* **📝 Advanced Code Editor:** Modern editing experience using CodeMirror 6 with dynamic syntax highlighting and line management.
* **📊 Real-Time Visual Schematics:** Automatic rendering of RF path diagrams, circuit topology graphs, and signal flowcharts via embedded Mermaid.js.
* **📻 ADIF Log Management:** Interactive visual rendering and structuring of amateur radio contact logs (QSOs).
* **📂 Native File System Access:** Secure, direct file operations on your local machine.
* **🌐 True Cross-Platform:** Automated multiplatform releases for Desktop (Windows, macOS, Linux), Android APK, iOS Simulator, ARM64 Linux (Raspberry Pi), Flatpak, and static Web builds via GitHub Actions CI/CD.
* **📱 Mobile & Embedded:** Native Android builds with SDK/NDK provisioning. ARM64 Linux packages for embedded and SBC devices.

---

## 🇺🇦 Українська

### Опис
**Signal & Radio IDE** — це потужне, легке та кросплатформенне середовище розробки (IDE) для парсингу кастомних DSL, візуалізації структурних схем радіотрактів, рендерингу електричних схем та зручної роботи з ADIF-логами аматорського радіозв'язку.

Створено на базі **Tauri v2, Rust, SvelteKit, CodeMirror 6 та Mermaid.js**. Працює нативно на Windows, Linux, macOS, Android, iOS та Raspberry Pi (ARM64).

### Основні можливості
* **⚡ Високопродуктивне ядро на Rust:** Миттєвий та безпечний розбір синтаксичних блоків `#graph` та `#adif`.
* **📝 Сучасний редактор коду:** Підсвітка синтаксису, керування рядками та висока швидкість роботи завдяки CodeMirror 6.
* **📊 Візуалізація радіотрактів у реальному часі:** Автоматичний рендеринг структурних схем, графів зв'язків та сигнальних діаграм через Mermaid.js.
* **📻 Робота з ADIF-логами:** Візуальне відображення журналів радіозв'язку у вигляді інтерактивних карток (QSO).
* **📂 Нативна робота з файлами:** Пряме читання та збереження локальних файлів на диску пристрою.
* **🌐 Повна кросплатформенність:** Автоматичні релізи для робочого столу (Windows, macOS Universal, Linux), Android APK, iOS Simulator, ARM64 Linux (Raspberry Pi), Flatpak та статичної Web-версії через GitHub Actions CI/CD.
* **📱 Мобільні та вбудовані системи:** Нативні збірки Android з SDK/NDK. Пакети ARM64 Linux для вбудованих пристроїв та одноплатникових комп'ютерів.

---

## 🇩🇪 Deutsch

### Übersicht
**Signal & Radio IDE** ist eine hochleistungsfähige, leichtgewichtige und plattformübergreifende Entwicklungsumgebung (IDE), die speziell für das Parsen benutzerdefinierter DSLs, die Visualisierung von Funksignalpfaden, das Rendern von HF-Schaltplänen und das Verwalten von Amateurfunk-ADIF-Logs entwickelt wurde.

Erstellt mit **Tauri v2, Rust, SvelteKit, CodeMirror 6 und Mermaid.js**. Läuft nativ auf Windows, Linux, macOS, Android, iOS und Raspberry Pi (ARM64).

### Hauptmerkmale
* **⚡ Rust-Kernel:** Extrem schnelles Parsen von domänenspezifischen Sprachen (`#graph`, `#adif`).
* **📝 Moderner Code-Editor:** Integrierter CodeMirror 6 Editor mit dynamischem Syntax-Highlighting.
* **📊 Echtzeit-Schaltpläne:** Automatische Generierung von Signalflussdiagrammen und HF-Topologie-Graphen mittels Mermaid.js.
* **📻 ADIF-Log-Verwaltung:** Interaktive visuelle Aufbereitung von Funkkontakt-Protokollen (QSOs).
* **📂 Nativer Dateisystemzugriff:** Sichere, direkte Dateioperationen auf dem lokalen Rechner.
* **🌐 Echte Plattformübergreifung:** Automatisierte Multiplattform-Releases für Desktop (Windows, macOS, Linux), Android APK, iOS Simulator, ARM64 Linux (Raspberry Pi), Flatpak und statische Web-Builds via GitHub Actions CI/CD.
* **📱 Mobil & Embedded:** Native Android-Builds mit SDK/NDK. ARM64 Linux-Pakete für eingebettete Systeme und Single-Board-Computer.

---

## 🛠️ Tech Stack / Стек технологій

| Layer | Technology |
|---|---|
| **Core & Native API** | [Rust](https://www.rust-lang.org/) + [Tauri v2](https://v2.tauri.app/) |
| **Frontend Framework** | [SvelteKit](https://kit.svelte.dev/) + [TypeScript](https://www.typescriptlang.org/) |
| **Code Editor** | [CodeMirror 6](https://codemirror.net/) |
| **Diagram Engine** | [Mermaid.js](https://mermaid.js.org/) |
| **CI/CD** | GitHub Actions |
| **Package Formats** | `.deb`, `.rpm`, `.AppImage`, `.flatpak`, `.apk`, `.dmg`, `.msi`, `.exe` |

---

## 📦 Downloads / Збірки

Pre-compiled binaries for supported platforms are automatically built via CI/CD and available under **[Releases](https://github.com/juv4uk/my-ide/releases)**:

* 🪟 **Windows:** `.exe`, `.msi`
* 🐧 **Linux x86_64:** `.deb`, `.AppImage`, `.rpm`
* 🍎 **macOS:** `.dmg` *(Universal: Apple Silicon & Intel)*
* 🤖 **Android:** `.apk` *(AArch64)*
* 📱 **iOS:** `.app` *(Simulator)*
* 🥧 **ARM Linux:** `.deb`, `.AppImage`, `.rpm` *(Raspberry Pi, ARM64 SBCs)*
* 📦 **Flatpak:** `.flatpak`
* 🌐 **Web:** `.zip` *(static files for hosting)*

---

## 🏷️ GitHub Topics / Теги

`tauri-v2` `rust` `sveltekit` `typescript` `codemirror` `mermaid-js` `dsl-parser` `adif` `ham-radio` `amateur-radio` `rf-engineering` `signal-visualization` `cross-platform` `desktop-app` `android` `ios` `arm64` `raspberry-pi` `flatpak` `webassembly`

> 💡 **Tip:** Click the gear icon next to "About" on the repo main page to add these topics and improve discoverability.

---

## 📄 License

This project is open-source software licensed under the **[MIT License](LICENSE)**.

Copyright (c) 2026 **Waldemar Sydiy M**
