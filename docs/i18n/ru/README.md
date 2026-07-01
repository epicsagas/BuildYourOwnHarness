> Этот документ является переводом английского README. Оригинал может опережать перевод — если что-то расходится, считайте верным [английский источник](../../../README.md).

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | **Русский** | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Ваш ИИ-агент, созданный под вас

*Не универсальный шаблон — харнес, скомпилированный под вашу роль, экспертизу и цели.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

Большинство ИИ-настроек выдают фиксированный набор инструментов и говорят «справляйтесь как хотите». BYOH меняет подход: он проводит интервью, узнаёт, чем вы реально занимаетесь, и генерирует персонализованный агентный харнес — навыки, память, пайплайны, — который сразу встраивается в ваш рабочий процесс.

## Кому это нужно?

- **Разработчики**, которым нужен агент, уже знающий их стек, стиль тестирования и ритм поставки
- **Исследователи**, которым нужны обзоры литературы, отслеживание источников и синтез в едином пайплайне
- **Авторы контента**, которым нужен соавтор, соответствующий их голосу и структуре проекта
- **Бизнес-аналитики**, которым нужны фреймворки принятия решений и пайплайны отчётности, а не «голый» чат

Если вы когда-нибудь думали «хотел бы я, чтобы мой ИИ реально знал мой контекст» — именно для этого и создан BYOH.

## Как это работает — обзор за 60 секунд

BYOH задуман так, чтобы им управлял ваш ИИ-агент, а не вы — набирая команды. Установите плагин и просто разговаривайте. Беседа *и есть* интервью, мастер настройки и сборка.

```
1. Install the plugin      # Claude Code / Codex / agy — автоустановка бинарника
2. "Build me a harness"    # ваш агент сканирует репо и компилирует результат
```

На следующей сессии ваш хост автоматически загружает харнес — агентов, навыки, память и пайплайны, настроенные под вас.

## Установите плагин (рекомендуется)

Используете **Claude Code, Codex или agy**? Установите плагин. Он объединяет MCP-сервер и **автоматически ставит бинарник при первой загрузке** — никакого Rust-тулчейна, никакой ручной настройки:

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy (Antigravity):**
```bash
agy plugin install /path/to/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add /path/to/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

### Используете любой другой MCP-совместимый хост?

BYOH говорит на MCP, поэтому Cursor, Zed, Continue и компания тоже работают. Один раз установите [бинарный файл](#installation), затем укажите вашему хосту на сервер:

```bash
byoh serve   # stdio MCP-сервер
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Замечание:** Репозиторий пока приватный. Используйте указанные выше пути. После публикации он появится в общем маркетплейсе `epicsagas/plugins`.

## Агентный режим — основной сценарий

Когда хост подключён, вы не вводите команды — вы просто разговариваете. Ваш агент напрямую вызывает MCP-инструменты BYOH, и беседа *и есть* интервью, сборка и цикл эволюции:

> **Вы:** *Я бэкенд-разработчик на Go, в этом месяце сдаю платежный API. Собери мне харнес.*
>
> **Агент:** *(сканирует репо через `profile_scan`, задаёт несколько точных вопросов через `profile_interview`, фиксирует жанр `developer`)* → компилирует `HarnessBundle` → устанавливает агентов, навыки, память и пайплайн secure-ship в Claude Code. Готово — на следующей сессии ваш агент уже говорит на вашем стеке.

Тот же поток, в рекомендуемом порядке вызова инструментов:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → compile → compile_dry_run → render_plugin → install_plugin
           → (опционально) registry_clone_skill → (позже) evolve_cycle
```

Доступные инструменты: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `genre_list`, `compile`, `compile_dry_run`, `render_plugin`, `install_plugin`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`.

Хотите, чтобы агент провёл вас по потоку? Просто скажите *"build my harness"* — встроенный агент `byoh-guide` оркестрирует весь процесс.

## Каталог плагинов

Каталог строится из README репозитория [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) — общественный, упорядоченный по звёздам список топ-100 репозиториев плагинов Claude. BYOH поставляет предсобранный bundle (пересобирается **еженедельно**, каждый понедельник 03:17 UTC), поэтому `byoh catalog index` выполняется за секунды; передайте `--no-bundle`, чтобы парсить исходный список напрямую.

```bash
# Одноразовая индексация — загружает готовый bundle за секунды
byoh catalog index

# Офлайн-поиск — сеть после индексации не нужна
byoh catalog search "memory" --genre developer --limit 5

# Добавить плагин в харнес
# лицензия, ключевые слова и жанр определяются автоматически из клонированного репо
byoh catalog vendor obra/superpowers --genre developer
```

ИИ-агент (через MCP-инструменты `catalog_search` / `catalog_vendor`) может выполнить весь этот поток самостоятельно — *"добавь плагин памяти в мой харнес"* — либо вы можете управлять им напрямую через CLI.

## Продвинутые пользователи: CLI (опционально)

Каждый из описанных выше потоков доступен и из терминала. CLI **вспомогательный** — удобен для скриптов, CI или когда не хочется вести диалог, — но агентный путь является основным.

### Ваш первый харнес — из CLI

```bash
byoh profile init me --paths ./src ./docs   # автосканирование проекта
byoh profile interview me                   # ~5-минутная беседа
byoh profile confirm me --genre developer   # зафиксировать жанр

byoh compile me --no-dry-run                # проверяет + записывает HarnessBundle (dry-run по умолчанию)
byoh render me --target claude              # или: codex | agy | all (по умолчанию: all)
byoh install me                             # рендерит в dist/, затем --host активирует

byoh run me                                 # запустить с активным харнесом
byoh evolve me                              # улучшить харнес на основе обратной связи
```

BYOH спрашивает о вашей роли, уровне экспертизы, инструментах и 30-дневной цели. Интервью адаптируется — исследователь получает иные вопросы, чем разработчик. `evolve` запускает цикл из трёх шлюзов (Critic / Seesaw / Stagnation), который невозможно обойти — поэтому эволюция безопасна и поддаётся аудиту.

## Как это работает под капотом

Движок синтеза BYOH сопоставляет теги вашего профиля с реестром навыков, выстраивает их в пайплайн с разрешёнными зависимостями и выдаёт `HarnessBundle` — артефакт, готовый к коммиту в git, который преобразуется в нативный формат любого поддерживаемого хоста.

- **4 кольца безопасности** — от встроенных навыков (Ring 1) до навыков сообщества/ненадёжных (Ring 4), каждое с нарастающей валидацией
- **3 шлюза эволюции** — каждый цикл `evolve` проходит шлюзы Critic (качество), Seesaw (регрессия) и Stagnation (плато); обход невозможен
- **Цель-ориентированные пайплайны** — объявление 30-дневной цели (запуск продукта, исследовательский отчёт, безопасная поставка…) автоматически накладывает соответствующую систему навыков поверх пайплайна

Архитектура: гексагональная — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Полное руководство — в `AGENTS.md`.

## Полный справочник CLI

```bash
# Профиль
byoh profile init <slug> [--paths ...]      # неразрушающее сканирование проекта
byoh profile interview <slug>               # интервью с проводником
byoh profile confirm <slug> --genre <g>     # подтвердить и зафиксировать профиль

# Сборка
byoh compile <slug> [--no-dry-run]          # dry-run по умолчанию; --no-dry-run чтобы записать bundle
byoh render <slug> [--target <host>]        # claude | codex | agy | all (по умолчанию: all)
byoh install <slug> [--target <host>] [--host] [--force]  # полиглотное дерево в dist/; --host активирует по хостам

# Запуск и эволюция
byoh run <slug>
byoh evolve <slug>

# Навыки сообщества
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Каталог
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<query>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## Installation

Нужен только если вы **не** используете плагин (он автоматически ставит бинарник) или хотите подключить BYOH к MCP-хосту без плагина.

### Бинарник (Rust-тулчейн не требуется)

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**Из исходников:**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # проверить установку
```

## Сборка и разработка

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # юнит + e2e
cvp                               # параллельно: check → clippy → test → fmt → build
```

Фича `mcp` (stdio MCP-сервер) включена по умолчанию. BYOH не включает встроенную базу знаний — для ретривала укажите сгенерированному харнесу на док-сервер вроде [alcove](https://github.com/epicsagas/alcove).

## Благодарности

BYOH стоит на плечах нескольких общественных проектов:

- **Каталог плагинов** — берётся из [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins), общественный упорядоченный по звёздам список топ-100 репозиториев плагинов Claude. Без него каталога бы не существовало.
- **Сопутствующие инструменты** — спроектирован для взаимодействия с [alcove](https://github.com/epicsagas/alcove) (сервер документов / RAG), [Episteme](https://github.com/epicsagas/Episteme) (граф знаний) и [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (автоматизация хранилищ).
- **Стек с открытым исходным кодом** — построен на [clap](https://docs.rs/clap), [serde](https://serde.rs), [ureq](https://docs.rs/ureq) и экосистеме Rust.

Записи каталога и внедрённые общественные навыки сохраняют собственные лицензии (определяются автоматически при внедрении). Сам BYOH распространяется под Apache-2.0.

## Лицензия

Apache-2.0.
