> Этот документ является переводом английского README. Оригинал может опережать перевод — если что-то расходится, считайте верным [английский источник](../../../README.md).

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | **Русский** | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Ваш AI-агент, построенный вокруг вас

*Не шаблонный вариант — а harness, собранный под вашу роль, экспертизу и цели.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

Большинство AI-настроек вручают вам фиксированный набор инструментов и говорят «удачи». BYOH переворачивает это: он интервьюирует вас, узнаёт, чем вы реально занимаетесь, и генерирует персонализованный harness агента — skills, agents, goal pipelines — который сразу ложится на ваш рабочий процесс.

## Для кого это?

- **Разработчики**, которым нужен агент, уже знающий их стек, стиль тестирования и ритм поставки
- **Исследователи**, которым нужны связанные между собой обзор литературы, отслеживание цитат и синтез
- **Автор**, которым нужен соавтор, подстраивающийся под их голос и структуру проекта
- **Бизнес-аналитики**, которым нужны фреймворки принятия решений и пайплайны отчётности, а не «голый» чат

Если вы когда-нибудь думали «жаль, что мой AI реально знает мой контекст» — это и делает BYOH.

## Как это работает за 60 секунд

BYOH создан, чтобы им управлял ваш AI-агент — а не вы, печатающий команды. Установите plugin, а дальше просто разговаривайте. Разговор *и есть* интервью, мастер и сборка.

```
1. Install the plugin      # Claude Code / Codex / agy — auto-installs the binary
2. "Build me a harness"    # ваш агент проводит интервью, собирает, сам заполняет
                           # пробелы и устанавливает — всё в одном диалоге
```

На следующей сессии ваш хост автоматически загружает harness — agents, skills и goal pipelines, настроенные под вас.

## Установите plugin (рекомендуется)

Используете **Claude Code, Codex или agy**? Установите plugin. Он включает MCP-сервер и **автоматически устанавливает бинарник при первой загрузке** — никакого Rust-тулчейна, никакой ручной настройки:

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@byoh
```

**agy (Antigravity):**
```bash
agy plugin install https://github.com/epicsagas/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add epicsagas/BuildYourOwnHarness
codex plugin add byoh@byoh
```

### Используете любой другой MCP-совместимый хост?

BYOH говорит на MCP, поэтому Cursor, Zed, Continue и подобные тоже работают. Один раз установите [бинарник](#установка-бинарного-файла-напрямую), затем укажите хосту на сервер:

```bash
byoh serve   # stdio MCP server
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Примечание:** Этот репозиторий несёт собственный маркетплейс `byoh` (.claude-plugin/marketplace.json) — автономная установка, без хаба.

## Установка бинарного файла напрямую

Нужна только если вы **не** используете plugin (он сам устанавливает бинарник) или хотите использовать BYOH на не-plugin MCP-хосте.

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

### Из исходников

```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verify
```

## Режим, управляемый агентом — основной путь

Когда хост подключён, вы не вводите команды — вы просто разговариваете. Ваш агент напрямую вызывает MCP-инструменты BYOH, и разговор *и есть* интервью, сборка и цикл evolve:

> **Вы:** *Я бэкенд-разработчик на Go, в этом месяце сдаю payments API. Собери мне harness.*
>
> **Агент:** *(сканирует репозиторий через `profile_scan`, задаёт несколько точечных вопросов через `profile_interview`, фиксирует жанр как `developer`)* → `build` синтезирует `HarnessBundle` и классифицирует каждый skill как `matched` / `authored` / `skeleton` → для любого skeleton, который нужен профилю (например, skill верификации для платежей), агент пишет его на месте через `author_skill`, а затем снова запускает `build` для подтверждения → устанавливает agents, skills и secure-ship goal pipeline в Claude Code. Авторский контент сохраняется между пересборками. Готово — на следующей сессии агент уже говорит на вашем стеке.

Тот же поток в предлагаемом порядке инструментов:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → build → (author_skill / author_doc to fill skeletons) → build → install_plugin
```

Доступные инструменты: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `build`, `author_skill`, `author_doc`, `enable_hook`, `list_overrides`, `delete_override`, `render_plugin`, `install_plugin`, `catalog_search`, `catalog_vendor`.

Хотите, чтобы агент провёл вас через всё это? Просто скажите *"build my harness"* — встроенный агент `byoh-guide` оркестрирует весь поток.

## Каталог plugins

Каталог строится из README [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) — поддерживаемого сообществом, ранжированного по звёздам списка топ-100 репозиториев Claude plugins. BYOH поставляется с предсобранным бандлом (обновляется **еженедельно**, каждый понедельник в 03:17 UTC), поэтому `byoh catalog index` отрабатывает за секунды; передайте `--no-bundle`, чтобы парсить вышестоящий список напрямую.

```bash
# One-time index — downloads a prebuilt bundle in seconds
byoh catalog index

# Search offline — no network needed after indexing
byoh catalog search "memory" --genre developer --limit 5

# Add a plugin to your harness
# license, keywords, and genre are auto-detected from the cloned repo
byoh catalog vendor obra/superpowers --genre developer
```

LLM-агент (через MCP-инструменты `catalog_search` / `catalog_vendor`) может выполнить весь этот поток автономно — *"add a memory plugin to my harness"* — либо вы можете управлять им прямо из CLI.

Несколько сопутствующих инструментов попадают в результаты поиска как **справочные материалы** (не зависимости): собственные инструменты исполнительного слоя BYOH — [alcove](https://github.com/epicsagas/alcove) (док-сервер), [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (автоматизация хранилища), [epic-harness](https://github.com/epicsagas/epic-harness) (рантайм hook/skill) — всплывают контекстно (запрос «doc server» / «search backend» находит alcove), чтобы агент мог порекомендовать их там, где это уместно. Добавляйте их только если они вам действительно нужны; бандлы так или иначе поставляются без зависимостей.

## Продвинутые пользователи: CLI (опционально)

Каждый поток выше доступен и из терминала. CLI — **вспомогательный**: удобен для скриптов, CI или когда вам не хочется общаться с чатом, — но управляемый агентом путь является основным.

### Ваш первый harness — из CLI

```bash
byoh profile init me --paths ./src ./docs   # auto-scans your project
byoh profile confirm me --genre developer   # lock in your genre (+ optional --goal)

byoh render me --target claude              # synthesize (compile + preset injection + static gate) and write the HarnessBundle; or: codex | agy | all (default: all)
byoh install me --scope local               # render to dist/, activate into this project's .claude/ only
byoh install me --scope global              # ...or ~/.claude + ~/.codex + ~/.gemini (was --host)
byoh install me --scope publish             # ...or add LICENSE + .gitignore and print git instructions
```

Само интервью управляется агентом (MCP-инструмент `profile_interview`) — разговор и есть интервью, поэтому интерактивного CLI-интервью нет. Статический гейт сборки всегда выполняется, поэтому бандл структурно валиден до отгрузки. Улучшение после установки — это разговорная ретроспектива в последующих сессиях, а не вызов инструмента.

## Как это работает «под капотом»

Синтез-движок BYOH сопоставляет теги вашего профиля с реестром skills, выстраивает их в пайплайн с учётом зависимостей и выдаёт `HarnessBundle` — git-ready артефакт, который рендерится в нативный формат любого поддерживаемого хоста.

- **Модель безопасности из 4 колец** — spec жизненного цикла (Ring 0) и встроенные pipeline-skills (Ring 1) вплоть до сообществных / недоверенных skills (Ring 3), каждое кольцо со всё более строгой валидацией; vendored skills фиксируются sha256 и проверяются при чтении + встраивании
- **Фундамент безопасности из 3 гейтов** — каждая сборка проходит static gate, который подтверждает наличие гейтов Critic (качество), Seesaw (регрессия) и Stagnation (плато); обход невозможен
- **Ориентированные на цель пайплайны** — декларирование 30-дневной цели (запуск продукта, исследовательский отчёт, безопасная поставка…) автоматически накладывает подходящую skill-лестницу

Архитектура: гексагональная — `domain / ports / adapters / application / compiler / evolve / templates / deploy / catalog / mcp / i18n / security / cli`. Полное руководство см. в `AGENTS.md`.

## Полный справочник по CLI

CLI намеренно небольшой: точки входа для машины (`serve`, `catalog index` в CI, `vendor` для мейнтейнеров) плюс скриптуемое зеркало основного потока сборки. Интервью и эволюция — только через MCP (управляются агентом).

```bash
# Profile
byoh profile init <slug> [--paths ...]      # non-destructive project scan
byoh profile confirm <slug> --genre <g> [--goal <text>]  # confirm and lock profile
byoh profile show <slug>                    # print the profile YAML

# Build (static gate always runs; render synthesizes: compile + preset injection)
byoh render <slug> [--target <host>]        # claude | codex | agy | all (default: all); writes the HarnessBundle
byoh install <slug> [--target <host>] [--scope local|global|publish] [--host] [--force]  # dist/ tree; --scope decides where it goes (local=this project, global=HOME, publish=+LICENSE/.gitignore+git steps). --host is legacy for --scope global.

# Community skills (maintainer/build-time; sha256-pinned and verified at read + embed time)
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Catalog
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<query>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]

# Diagnostics / server
byoh doctor                                 # check execution-layer tools
byoh serve                                  # stdio MCP server (agent-led mode)
```

Профили и кэш каталога по умолчанию лежат в `~/.byoh` (переопределяется через `BYOH_HOME`).

## Сборка и разработка

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unit + e2e
cvp                               # parallel: check → clippy → test → fmt → build
```

Фича `mcp` (stdio MCP-сервер) включена по умолчанию. BYOH не поставляет встроенной базы знаний — для retrieval укажите вашему сгенерированному harness на док-сервер вроде [alcove](https://github.com/epicsagas/alcove).

## Благодарности

BYOH стоит на плечах нескольких инициатив сообщества:

- **Каталог plugins** — берётся из [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins), ранжированного по звёздам списке топ-100 репозиториев Claude plugins от сообщества. Без него каталога бы не существовало.
- **Сопутствующие инструменты** — спроектированы для совместной работы с [alcove](https://github.com/epicsagas/alcove) (док-сервер / RAG), [Episteme](https://github.com/epicsagas/Episteme) (граф знаний) и [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (автоматизация хранилища).
- **Стек open-source** — построен на [clap](https://docs.rs/clap), [serde](https://serde.rs), [ureq](https://docs.rs/ureq) и экосистеме Rust.

Записи каталога и vendored сообществные skills сохраняют собственные лицензии (определяются автоматически при vendor). Сам BYOH распространяется под Apache-2.0.

## Лицензия

Apache-2.0.
