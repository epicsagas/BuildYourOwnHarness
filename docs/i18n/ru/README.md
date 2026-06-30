<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | **Русский** | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### Ваш ИИ-агент, созданный под вас

*Не универсальный шаблон — харнес, скомпилированный под вашу роль, экспертизу и цели.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

Большинство ИИ-инструментов дают фиксированный набор функций и говорят «разбирайтесь сами». BYOH делает наоборот: краткое интервью узнаёт, как вы работаете на самом деле, и генерирует персонализированный агентный харнес — навыки, память, пайплайны — который сразу вписывается в ваш рабочий процесс.

## Для кого это?

- **Разработчики** — агент, который уже знает ваш стек, стиль тестирования и цикл поставки
- **Исследователи** — обзор литературы, отслеживание источников и синтез в одном пайплайне
- **Авторы контента** — партнёр по написанию, который соответствует вашему стилю и структуре проекта
- **Бизнес-аналитики** — фреймворки принятия решений и пайплайны отчётности, а не просто чат

Если вы когда-нибудь думали «хотел бы я, чтобы мой ИИ реально знал мой контекст» — именно это делает BYOH.

## Как это работает за 60 секунд

BYOH рассчитан на управление вашим ИИ-агентом. Установите его, подключите хост по MCP и просто разговаривайте — беседа *и есть* интервью, мастер настройки и сборка.

```
1. Install byoh              # однострочная установка (см. ниже)
2. Connect your host via MCP # byoh serve — любой MCP-совместимый агент
3. "Build me a harness"      # ваш агент сканирует репо и компилирует результат
```

На следующей сессии ваш хост автоматически загружает харнес — агентов, навыки, память и пайплайны, настроенные под вас.

**Предпочитаете терминал?** Тот же поток из CLI:
```
byoh profile init me        # сканирует ваш проект — не разрушает, только читает
byoh profile interview me   # короткий разговор о вашей роли и целях
byoh compile me             # генерирует ваш персональный харнес
byoh install me             # разворачивает в Claude / Codex / agy
```

**Уже знаете, что нужно?** Загляните в каталог сообщества:
```bash
byoh catalog index                                 # скачать список топ-100 плагинов (секунды)
byoh catalog search "code review"                  # найти нужный плагин
byoh catalog vendor anthropics/claude-code-review  # добавить в харнес
```

## Установка

### Бинарник (рекомендуется — Rust не нужен)

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

### Подключите ваш ИИ-хост

BYOH говорит на MCP, поэтому им может управлять любой MCP-совместимый агент. Установите бинарник выше, запустите сервер — и ваш хост вызывает любой инструмент BYOH напрямую:

```bash
byoh serve   # stdio MCP-сервер
```

Для **других агентов** (Cursor, Zed, Continue, …) добавьте `byoh` в MCP-конфиг вашего хоста:
```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

Используете **Claude Code, Codex или agy**? Тогда установите плагин — он объединяет MCP-сервер и автоматически ставит бинарник при первой загрузке (Rust не нужен):

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

> **Замечание:** Репозиторий пока приватный. Используйте пути выше. После публикации он появится в общем маркетплейсе `epicsagas/plugins`.

## Агентный режим

Когда хост подключён, вы не вводите команды — вы просто разговариваете. Ваш агент напрямую вызывает 14 инструментов BYOH, и беседа *и есть* интервью, сборка и цикл эволюции:

Доступные инструменты: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `evolve_cycle`, `genre_list`, `registry_clone_skill`, `catalog_search`, `catalog_vendor` и другие.

## Ваш первый харнес — из CLI

Те же шаги, управляемые из терминала:

### Шаг 1 — Профиль
```bash
byoh profile init me --paths ./src ./docs   # автосканирование проекта
byoh profile interview me                   # ~5 минут разговора
byoh profile confirm me --genre developer   # зафиксировать жанр
```

BYOH спрашивает о вашей роли, уровне экспертизы, инструментах и 30-дневной цели. Интервью адаптируется — исследователь получит другие вопросы, чем разработчик.

### Шаг 2 — Компиляция и установка
```bash
byoh compile me          # сгенерировать HarnessBundle (валидация + шлюзы)
byoh render me --target claude   # или: codex | agy | all
byoh install me          # безопасная установка в dist/
```

### Шаг 3 — Запуск и эволюция
```bash
byoh run me              # запустить с активным харнесом
byoh evolve me           # улучшить харнес на основе обратной связи
```

`evolve` запускает цикл из трёх шлюзов (Critic / Seesaw / Stagnation), который нельзя обойти — эволюция безопасна и аудируема.

## Каталог плагинов

Каталог предоставляет кураторский список топ-100 Claude-плагинов (отсортированных по звёздам, обновляемых ежедневно) — находите и добавляйте навыки сообщества, не выходя из терминала.

```bash
# Одноразовая индексация — загружает готовый bundle за секунды
byoh catalog index

# Офлайн-поиск — сеть после индексации не нужна
byoh catalog search "memory" --genre developer --limit 5

# Добавить плагин в харнес
# лицензия, ключевые слова и жанр определяются автоматически из клонированного репо
byoh catalog vendor obra/superpowers --genre developer
```

ИИ-агент (через MCP-инструменты `catalog_search` / `catalog_vendor`) может выполнить весь этот поток самостоятельно — или вы управляете им напрямую через CLI.

## Полный справочник CLI

```bash
# Профиль
byoh profile init <slug> [--paths ...]      # неразрушающее сканирование проекта
byoh profile interview <slug>               # интерактивное интервью
byoh profile confirm <slug> --genre <g>     # подтвердить и зафиксировать профиль

# Сборка
byoh compile <slug> [--dry-run]             # валидация + генерация HarnessBundle
byoh render <slug> --target <host>          # claude | codex | agy | all
byoh install <slug> [--host <dir>]          # развернуть в dist/ или в папку плагина

# Запуск и эволюция
byoh run <slug>
byoh evolve <slug>

# Навыки сообщества
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Каталог
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<запрос>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## Как это работает под капотом

Движок синтеза BYOH сопоставляет теги вашего профиля с реестром навыков, выстраивает их в пайплайн с разрешёнными зависимостями и выдаёт `HarnessBundle` — git-готовый артефакт, который рендерится в нативный формат любого поддерживаемого хоста.

- **4 кольца безопасности** — от встроенных навыков (Ring 1) до навыков сообщества/ненадёжных (Ring 4), каждое с нарастающей валидацией
- **3 шлюза эволюции** — каждый цикл `evolve` проходит шлюзы Critic (качество), Seesaw (регрессия) и Stagnation (плато); обход невозможен
- **Цель-ориентированные пайплайны** — объявив 30-дневную цель (запуск продукта, исследовательский отчёт, безопасная доставка…), вы автоматически получаете соответствующую лестницу навыков

Архитектура: гексагональная — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Подробное описание — в `AGENTS.md`.

## Сборка и разработка

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # юнит + e2e
cvp                               # параллельно: check → clippy → test → fmt → build
```

Фича `mcp` (stdio MCP-сервер) включена по умолчанию. BYOH не поставляется со встроенной базой знаний — для ретривала укажите сгенерированному харнесу на док-сервер вроде [alcove](https://github.com/epicsagas/alcove).

## Лицензия

Apache-2.0.
