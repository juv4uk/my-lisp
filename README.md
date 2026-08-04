# Signal & Radio Log

<p align="center">
  <img src="docs/hero.svg" alt="Signal & Radio Log" width="100%">
</p>

<p align="center">
  <a href="README.md"><img src="https://img.shields.io/badge/English-2563EB?style=for-the-badge" alt="English"></a>
  <a href="README.uk.md"><img src="https://img.shields.io/badge/Українська-172554?style=for-the-badge" alt="Українська"></a>
  <a href="README.de.md"><img src="https://img.shields.io/badge/Deutsch-172554?style=for-the-badge" alt="Deutsch"></a>
  <a href="#about-the-author"><img src="https://img.shields.io/badge/About_me-172554?style=for-the-badge" alt="About me"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-172554?style=for-the-badge" alt="License"></a>
</p>

<p align="center">
  <strong>Type less. Spend more time on air.</strong>
</p>

<p align="center">
  <a href="https://github.com/juv4uk/my-ide/releases"><img src="https://img.shields.io/github/v/release/juv4uk/my-ide?color=7c3aed&label=release" alt="Latest release"></a>
  <a href="https://github.com/juv4uk/my-ide/actions"><img src="https://github.com/juv4uk/my-ide/actions/workflows/publish-release.yml/badge.svg" alt="Build status"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-f59e0b" alt="MIT License"></a>
</p>

## A radio logbook that stays out of your way

**Signal & Radio Log** is a lightweight offline-first logbook for amateur-radio operators. Enter a callsign, tap the band and mode, check the RST, and save the QSO. No account is required, and your records stay on your device.

The project has a special place for **QRPp** experiments at 500, 100, or even 50 mW. It still records contacts at any power—QRPp is an invitation to experiment, not a restriction.

### What you can do

- log a QSO with large, thumb-friendly controls;
- use portrait or landscape mode on mobile, tablet, and desktop;
- search, edit, and delete contacts;
- import and export ADIF 3.1.7 without losing unknown fields;
- keep Markdown notes with live preview;
- learn tables and Mermaid diagrams through ready-made templates;
- switch between English, Ukrainian, and German with one tap.

## Download

Get the latest build from **[Releases](https://github.com/juv4uk/my-ide/releases)**.

| Platform | Package |
|---|---|
| Windows | `.msi`, `.exe` |
| Linux | `.deb`, `.rpm`, `.AppImage`, `.flatpak` |
| macOS | `.dmg` |
| Android | `.apk` |
| iOS | `.app` for Simulator |
| Raspberry Pi / ARM64 | Linux packages |
| Web | Static build |

> The exact files available depend on successful builds in GitHub Actions.

## What comes next

**QSO Connect** is the planned bridge between an on-air contact and private online communication. Its encrypted, transport-independent foundation is already being built. Internet relay comes first; WebRTC P2P and LoRa may follow later. The journal itself will remain fully useful offline.

## About the author

I am **Waldemar**, a radio amateur and the creator of Signal & Radio Log. I am building the tool I want to use myself: simple enough for a beginner, practical in the field, respectful of personal data, and open to experimentation.

GitHub: **[@juv4uk](https://github.com/juv4uk)**

## Development

```bash
npm install
npm run dev
```

```bash
npm test
npm run check
npm run build
cargo check --manifest-path src-tauri/Cargo.toml
```

Built with [Tauri 2](https://v2.tauri.app/), [SvelteKit](https://svelte.dev/docs/kit), TypeScript, and Rust. Contributions, translations, device testing, and practical amateur-radio experience are welcome. You can start with an **[Issue](https://github.com/juv4uk/my-ide/issues)**.

## License

Signal & Radio Log is open-source software available under the **[MIT License](LICENSE)**.
