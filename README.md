# ⚡ Signal & Radio IDE

<p align="center">
  <img src="docs/demo.png" alt="Signal & Radio IDE Screenshot" width="800">
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
| 🐧 **Linux** | `.deb`, `.AppImage`, `.rpm` | 🟢 **Automated in Releases** |
| 🍏 **macOS** | `.dmg` *(Universal: M-Series & Intel)* | 🟢 **Automated in Releases** |
| 🤖 **Android** | Project source ready | 🟡 **Builds locally / NDK ready** |
| 📱 **iOS** | Xcode project structure | ⚪ **Ready to build via Xcode** |

---

## 🇬🇧 English

### Overview
**Signal & Radio IDE** is a high-performance, lightweight, cross-platform Integrated Development Environment designed for custom DSL parsing, radio signal path visualization, and amateur radio ADIF log management.

Built with safety, speed, and modern UX in mind using **Tauri v2, Rust, SvelteKit, CodeMirror 6, and Mermaid.js**.

### Key Features
* **⚡ Rust-Powered Core:** Blazing fast parsing for custom `#graph` and `#adif` domain-specific languages.
* **📝 Advanced Code Editor:** Modern editing experience using CodeMirror 6 with dynamic syntax highlighting and line management.
* **📊 Real-Time Visual Schematics:** Automatic rendering of RF path diagrams and topology graphs via embedded Mermaid.js.
* **📻 ADIF Log Management:** Interactive visual rendering and structuring of amateur radio contact logs.
* **📂 Native File System Access:** Secure, direct file operations on your local machine.
* **🌐 True Cross-Platform:** Automated multiplatform releases for Desktop (Windows, macOS, Linux).

---

## 🇺🇦 Українська

### Опис
**Signal & Radio IDE** — це потужне, легке та кросплатформенне середовище розробки (IDE) для парсингу кастомних DSL, візуалізації структурних схем радіотрактів та зручної роботи з ADIF-логами аматорського радіозв'язку.

Створено на базі **Tauri v2, Rust, SvelteKit, CodeMirror 6 та Mermaid.js**.

### Основні можливості
* **⚡ Високопродуктивне ядро на Rust:** Миттєвий та безпечний розбір синтаксичних блоків `#graph` та `#adif`.
* **📝 Сучасний редактор коду:** Підсвітка синтаксису, керування рядками та висока швидкість роботи завдяки CodeMirror 6.
* **📊 Візуалізація радіотрактів у реальному часі:** Автоматичний рендеринг структурних схем та графів зв'язків через Mermaid.js.
* **📻 Робота з ADIF-логами:** Візуальне відображення журналів радіозв'язку у вигляді інтерактивних карток.
* **📂 Нативна робота з файлами:** Пряме читання та збереження локальних файлів на диску пристрою.
* **🌐 Повна кросплатформенність:** Автоматичні релізи для робочого столу (Windows, macOS Universal, Linux).

---

## 🇩🇪 Deutsch

### Übersicht
**Signal & Radio IDE** ist eine hochleistungsfähige, leichtgewichtige und plattformübergreifende Entwicklungsumgebung (IDE), die speziell für das Parsen benutzerdefinierter DSLs, die Visualisierung von Funksignalpfaden und das Verwalten von Amateurfunk-ADIF-Logs entwickelt wurde.

### Hauptmerkmale
* **⚡ Rust-Kernel:** Extrem schnelles Parsen von domänenspezifischen Sprachen (`#graph`, `#adif`).
* **📝 Moderner Code-Editor:** Integrierter CodeMirror 6 Editor mit dynamischem Syntax-Highlighting.
* **📊 Echtzeit-Schaltpläne:** Automatische Generierung von Signalflussdiagrammen mittels Mermaid.js.
* **📻 ADIF-Log-Verwaltung:** Interaktive visuelle Aufbereitung von Funkkontakt-Protokollen.

---

## 🛠️ Tech Stack / Стек технологій

| Layer | Technology |
|---|---|
| **Core & Native API** | [Rust](https://www.rust-lang.org/) + [Tauri v2](https://v2.tauri.app/) |
| **Frontend Framework** | [SvelteKit](https://kit.svelte.dev/) + [TypeScript](https://www.typescriptlang.org/) |
| **Code Editor** | [CodeMirror 6](https://codemirror.net/) |
| **Diagram Engine** | [Mermaid.js](https://mermaid.js.org/) |

---

## 📦 Downloads / Збірки

Pre-compiled binaries for supported platforms are automatically built via CI/CD and available under **[Releases](https://github.com/juv4uk/my-ide/releases)**:

* 🪟 **Windows:** `.exe`, `.msi`
* 🐧 **Linux:** `.deb`, `.AppImage`, `.rpm`
* 🍎 **macOS:** `.dmg` *(Universal: Apple Silicon & Intel)*

---

## 📄 License

This project is open-source software licensed under the **[MIT License](LICENSE)**.

Copyright (c) 2026 **Volodymyr Sydiy**