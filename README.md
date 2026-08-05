# Signal & Radio Log

<p align="center">
  <img src="docs/hero.svg" alt="Signal & Radio Log" width="100%">
</p>

<p align="center">
  <a href="#english"><img src="https://img.shields.io/badge/English-2563EB?style=for-the-badge" alt="English"></a>
  <a href="#українська"><img src="https://img.shields.io/badge/Українська-172554?style=for-the-badge" alt="Українська"></a>
  <a href="#deutsch"><img src="https://img.shields.io/badge/Deutsch-172554?style=for-the-badge" alt="Deutsch"></a>
  <a href="#help-wanted"><img src="https://img.shields.io/badge/Help_wanted-F59E0B?style=for-the-badge" alt="Help wanted"></a>
  <a href="#about-the-author"><img src="https://img.shields.io/badge/About_me-172554?style=for-the-badge" alt="About me"></a>
  <a href="#license"><img src="https://img.shields.io/badge/License-172554?style=for-the-badge" alt="License"></a>
</p>

<p align="center">
  <strong>Type less. Spend more time on air.</strong>
</p>

<p align="center">
  <a href="https://github.com/juv4uk/my-ide/releases"><img src="https://img.shields.io/github/v/release/juv4uk/my-ide?color=7c3aed&label=release" alt="Latest release"></a>
  <a href="https://github.com/juv4uk/my-ide/actions"><img src="https://github.com/juv4uk/my-ide/actions/workflows/publish-release.yml/badge.svg" alt="Build status"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-f59e0b" alt="MIT License"></a>
</p>

<a id="english"></a>

## English

### A radio logbook that stays out of your way

**Signal & Radio Log** is a lightweight offline-first logbook for amateur-radio operators. Enter a callsign, tap the band and mode, check the RST, and save the QSO. No account is required, and your records stay on your device.

The project has a special place for **QRPp** experiments at 500, 100, or even 50 mW. It still records contacts at any power—QRPp is an invitation to experiment, not a restriction.

### What you can do

- log a QSO with large, thumb-friendly controls;
- use portrait or landscape mode on mobile, tablet, and desktop;
- search, edit, and delete contacts;
- import and export ADIF 3.1.7 without losing unknown fields;
- keep Markdown notes with live preview;
- learn tables and Mermaid diagrams through ready-made templates;
- build on the safe, local [Radio Rules](docs/radio-rules.md) foundation for QSO tags and notes;
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

### This project is looking for experienced friends

Signal & Radio Log is a serious open-source project, but it is still maintained by one creator—not a company or a large team. It has reached the point where thoughtful review and mentorship can make a real difference.

The Apple builds especially need someone willing to take them under their wing. GitHub Actions currently produces a universal macOS package and an iOS Simulator app, but proper distribution still needs:

- macOS code signing and notarization;
- iOS provisioning and builds for real devices;
- testing on actual Macs, iPhones, and iPads;
- patient review of the Tauri/Swift/Xcode release process.

Help with Rust, Svelte, accessibility, security, radio workflows, documentation, and release engineering is equally welcome. You do not have to take over the project. Reviewing one workflow, explaining one Apple requirement, or mentoring one architectural decision is already valuable.

If you are an experienced developer, radio amateur, teacher, or maintainer and this small independent project speaks to you, please **[open an Issue and say hello](https://github.com/juv4uk/my-ide/issues/new)**. We would be glad to build it with you.

---

<a id="українська"></a>

## Українська

### Радіожурнал, який не заважає працювати в ефірі

**Signal & Radio Log** — легкий офлайн-журнал для радіоаматора. Введіть позивний, торкніться діапазону й режиму, перевірте RST та збережіть QSO. Обліковий запис не потрібен, а записи залишаються на вашому пристрої.

Особливе місце у проєкті займають **QRPp**-експерименти на 500, 100 або навіть 50 мВт. Водночас журнал зберігає зв’язки на будь-якій потужності: QRPp тут є запрошенням до експериментів, а не обмеженням.

### Що вже можна робити

- записувати QSO великими кнопками, зручними для одного пальця;
- працювати в портретній або альбомній орієнтації, на планшеті й комп’ютері;
- шукати, редагувати та видаляти зв’язки;
- імпортувати й експортувати ADIF 3.1.7 без втрати невідомих полів;
- вести Markdown-нотатки з живим переглядом;
- вивчати таблиці й Mermaid за готовими шаблонами;
- використовувати основу безпечних локальних [Radio Rules](docs/radio-rules.md) для тегів і нотаток QSO;
- одним торканням перемикати англійську, українську та німецьку.

Остання версія доступна на сторінці **[Releases](https://github.com/juv4uk/my-ide/releases)**.

### Що буде далі

**QSO Connect** має стати мостом між знайомством в ефірі та приватним спілкуванням через інтернет. Основа зашифрованого протоколу вже закладається. Спочатку планується Internet Relay, згодом — WebRTC P2P і, можливо, LoRa. Сам журнал і надалі повноцінно працюватиме офлайн.

### Проєкт шукає досвідчених друзів

Signal & Radio Log — уже серйозний open-source проєкт, але його розвиває одна людина, а не компанія чи велика команда. Ми дійшли до моменту, коли уважний review і добре наставництво можуть справді змінити майбутнє проєкту.

Особливо потрібна людина, готова взяти під опіку Apple-напрям. GitHub Actions уже створює універсальну збірку macOS та застосунок для iOS Simulator, але для нормального розповсюдження ще потрібні:

- підпис коду й notarization для macOS;
- provisioning і збірки iOS для реальних пристроїв;
- перевірка на справжніх Mac, iPhone та iPad;
- терплячий review процесу релізу Tauri/Swift/Xcode.

Так само цінною буде допомога з Rust, Svelte, доступністю, безпекою, радіоаматорськими сценаріями, документацією та release engineering. Не обов’язково брати на себе весь проєкт. Перевірити один workflow, пояснити одну вимогу Apple або допомогти з одним архітектурним рішенням — це вже багато.

Якщо ви досвідчений розробник, радіоаматор, викладач або мейнтейнер і вам близький цей маленький незалежний проєкт — **[відкрийте Issue й просто привітайтеся](https://github.com/juv4uk/my-ide/issues/new)**. Ми будемо раді будувати його разом.

---

<a id="deutsch"></a>

## Deutsch

### Ein Funklogbuch, das beim Funken nicht im Weg steht

**Signal & Radio Log** ist ein leichtes Offline-Logbuch für Funkamateure. Rufzeichen eingeben, Band und Betriebsart antippen, RST prüfen und das QSO speichern. Ein Konto ist nicht nötig, und die Einträge bleiben auf dem eigenen Gerät.

Einen besonderen Platz haben **QRPp**-Experimente mit 500, 100 oder sogar 50 mW. Verbindungen mit jeder anderen Leistung werden ebenfalls gespeichert: QRPp ist eine Einladung zum Experimentieren, keine Einschränkung.

### Was bereits möglich ist

- QSO-Eingabe mit großen, daumenfreundlichen Schaltflächen;
- Hoch- und Querformat auf Mobilgeräten sowie Layouts für Tablet und Desktop;
- Verbindungen suchen, bearbeiten und löschen;
- ADIF 3.1.7 importieren und exportieren, ohne unbekannte Felder zu verlieren;
- Markdown-Notizen mit Live-Vorschau schreiben;
- Tabellen und Mermaid-Diagramme mit fertigen Vorlagen kennenlernen;
- die sichere lokale [Radio-Rules](docs/radio-rules.md)-Grundlage für QSO-Tags und Notizen nutzen;
- mit einmaligem Antippen zwischen Englisch, Ukrainisch und Deutsch wechseln.

Die aktuelle Version steht unter **[Releases](https://github.com/juv4uk/my-ide/releases)** bereit.

### Wie es weitergeht

**QSO Connect** soll eine Brücke zwischen dem Funkkontakt und privater Online-Kommunikation bilden. Die verschlüsselte Protokollbasis wird bereits vorbereitet. Zuerst ist ein Internet-Relay geplant; später können WebRTC P2P und möglicherweise LoRa folgen. Das Logbuch selbst bleibt vollständig offline nutzbar.

### Das Projekt sucht erfahrene Freunde

Signal & Radio Log ist bereits ein ernsthaftes Open-Source-Projekt, wird aber von einer einzelnen Person entwickelt und nicht von einem Unternehmen oder einem großen Team. Es ist an einem Punkt angekommen, an dem sorgfältiges Review und gute Begleitung wirklich etwas bewirken können.

Besonders gesucht wird jemand, der den Apple-Bereich unter seine Fittiche nimmt. GitHub Actions erstellt bereits ein universelles macOS-Paket und eine App für den iOS Simulator. Für eine reguläre Veröffentlichung fehlen jedoch noch:

- Code-Signierung und Notarisierung für macOS;
- iOS-Provisioning und Builds für echte Geräte;
- Tests auf realen Macs, iPhones und iPads;
- geduldiges Review des Tauri/Swift/Xcode-Release-Prozesses.

Auch Hilfe bei Rust, Svelte, Barrierefreiheit, Sicherheit, Amateurfunk-Abläufen, Dokumentation und Release Engineering ist sehr willkommen. Niemand muss gleich das ganze Projekt übernehmen. Ein Workflow-Review, die Erklärung einer Apple-Anforderung oder Begleitung bei einer Architekturentscheidung ist bereits wertvoll.

Wenn Sie erfahrener Entwickler, Funkamateur, Lehrer oder Maintainer sind und dieses kleine unabhängige Projekt Sie anspricht, **[öffnen Sie bitte ein Issue und sagen Sie einfach Hallo](https://github.com/juv4uk/my-ide/issues/new)**. Wir würden uns freuen, es gemeinsam mit Ihnen weiterzuentwickeln.

---

<a id="help-wanted"></a>

## Help wanted · Потрібна допомога · Hilfe gesucht

The project is ready for contributors and mentors, especially around signed Apple releases and testing on real devices. Small, patient contributions are welcome—there is no expectation to take on everything.

Проєкт відкритий для помічників і наставників, особливо щодо підписаних Apple-релізів та перевірки на реальних пристроях. Навіть невелика спокійна допомога важлива — ніхто не очікує, що одна людина візьме на себе все.

Das Projekt freut sich über Mitwirkende und Mentoren, besonders bei signierten Apple-Releases und Tests auf echten Geräten. Auch kleine, geduldige Beiträge helfen—niemand soll alles allein übernehmen.

**[Introduce yourself in a GitHub Issue · Напишіть про себе в Issue · Stellen Sie sich in einem Issue vor](https://github.com/juv4uk/my-ide/issues/new)**

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

<a id="license"></a>

## License · Ліцензія · Lizenz

Signal & Radio Log is open-source software available under the **[MIT License](LICENSE)**.

Signal & Radio Log — програмне забезпечення з відкритим кодом за умовами **[ліцензії MIT](LICENSE)**.

Signal & Radio Log ist Open-Source-Software unter der **[MIT-Lizenz](LICENSE)**.
