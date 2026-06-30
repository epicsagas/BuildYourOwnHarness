> Этот документ — русская версия [README.md](../../../README.md). Английская версия является авторитетным источником.

# BuildYourOwnHarness (BYOH)

> **Ваш ИИ-агент, созданный под вас** — не универсальный шаблон, а харнес, скомпилированный под вашу роль, экспертизу и цели.

Большинство ИИ-инструментов дают фиксированный набор функций и говорят «разбирайтесь сами». BYOH делает наоборот: краткое интервью узнаёт, как вы работаете на самом деле, и генерирует персонализированный агентный харнес — навыки, память, пайплайны — который сразу вписывается в ваш рабочий процесс.

## Для кого это?

- **Разработчики** — агент, который уже знает ваш стек, стиль тестирования и цикл поставки
- **Исследователи** — обзор литературы, отслеживание источников и синтез в одном пайплайне
- **Авторы контента** — партнёр по написанию, который соответствует вашему стилю и структуре проекта
- **Бизнес-аналитики** — фреймворки принятия решений и пайплайны отчётности, а не просто чат

Если вы когда-нибудь думали «хотел бы я, чтобы мой ИИ реально знал мой контекст» — именно это делает BYOH.

## Старт за 60 секунд

```
1. byoh profile init me        # сканирует ваш проект — не разрушает, только читает
2. byoh profile interview me   # короткий разговор о вашей роли и целях
3. byoh compile me             # генерирует ваш персональный харнес
4. byoh install me             # разворачивает в Claude / Codex / agy
```

На следующей сессии ваш хост автоматически загружает харнес — агенты, навыки, память и пайплайны, настроенные под вас.

**Знаете, что ищете?** Загляните в каталог сообщества:
```bash
byoh catalog index                        # скачать список топ-100 плагинов (секунды)
byoh catalog search "code review"         # найти нужный плагин
byoh catalog vendor anthropics/claude-code-review   # добавить в харнес
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

### Подключить plugin к вашему ИИ-хосту

BYOH поставляется как полиглот-plugin и работает в Claude Code, Codex и agy — один репозиторий, все три хоста.

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

Plugin автоматически устанавливает бинарник `byoh` при первой загрузке — Rust на вашей машине не нужен.

> **Замечание:** Репозиторий пока приватный. Используйте пути выше. После публикации он появится в маркетплейсе `epicsagas/plugins`.

## Ваш первый харнес — шаг за шагом

### Шаг 1 — Профиль
```bash
byoh profile init me --paths ./src ./docs   # автосканирование проекта
byoh profile interview me                   # ~5 минут разговора
byoh profile confirm me --genre developer   # зафиксировать жанр
```

BYOH спрашивает о вашей роли, уровне экспертизы, инструментах и 30-дневной цели. Интервью адаптируется — исследователь получит другие вопросы, чем разработчик.

### Шаг 2 — Компиляция и установка
```bash
byoh compile me                  # сгенерировать HarnessBundle (валидация + шлюзы)
byoh render me --target claude   # или: codex | agy | all
byoh install me                  # безопасная установка в dist/
```

### Шаг 3 — Запуск и эволюция
```bash
byoh run me       # запустить с активным харнесом
byoh evolve me    # улучшить харнес на основе обратной связи
```

`evolve` запускает цикл из трёх шлюзов (Critic / Seesaw / Stagnation), который нельзя обойти — эволюция безопасна и аудируема.

## Каталог плагинов

Каталог предоставляет кураторский список топ-100 Claude-плагинов (отсортированных по звёздам, обновляемых ежедневно) — находите и добавляйте навыки сообщества, не выходя из терминала.

```bash
# Одноразовая индексация — загружает готовый bundle за секунды
byoh catalog index

# Офлайн-поиск — сеть после индексации не нужна
byoh catalog search "memory" --genre developer --limit 5

# Добавить plugin в харнес
# лицензия, ключевые слова и жанр определяются автоматически из клонированного репо
byoh catalog vendor obra/superpowers --genre developer
```

ИИ-агент (через MCP-инструменты `catalog_search` / `catalog_vendor`) может выполнить весь этот поток самостоятельно — или вы управляете им напрямую через CLI.

## Агентный режим

`byoh serve` запускает stdio MCP-сервер. Вместо того чтобы вводить команды вручную, ваш ИИ-хост напрямую вызывает 14 инструментов BYOH — разговор *и есть* интервью, мастер настройки и исполнение.

```bash
byoh serve   # Claude / Codex / agy подключается и берёт управление на себя
```

Доступные инструменты: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `evolve_cycle`, `rag_index`, `rag_search`, `genre_list`, `registry_clone_skill`, `catalog_search`, `catalog_vendor` и другие.

## Полный справочник CLI

```bash
# Профиль
byoh profile init <slug> [--paths ...]      # неразрушающее сканирование проекта
byoh profile interview <slug>               # интерактивное интервью
byoh profile confirm <slug> --genre <g>     # подтвердить и зафиксировать профиль

# Сборка
byoh compile <slug> [--dry-run]             # валидация + генерация HarnessBundle
byoh render <slug> --target <host>          # claude | codex | agy | all
byoh install <slug> [--host <dir>]          # развернуть в dist/ или в папку plugin

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

# База знаний (RAG)
byoh index <slug> [--corpus <dir>] [--force]
byoh search <slug> "<запрос>" [--genre <g>] [--k N]
```

## Как это работает под капотом

Движок синтеза BYOH сопоставляет теги вашего профиля с реестром навыков, выстраивает их в пайплайн с разрешёнными зависимостями и выдаёт `HarnessBundle` — git-готовый артефакт, который рендерится в нативный формат любого поддерживаемого хоста.

- **4 кольца безопасности** — от встроенных навыков (Ring 1) до навыков сообщества/ненадёжных (Ring 4), каждое с нарастающей валидацией
- **3 шлюза эволюции** — каждый цикл `evolve` проходит шлюзы Critic (качество), Seesaw (регрессия) и Stagnation (плато); обход невозможен
- **Персистентный RAG** — инкрементальное переиндексирование при изменениях (`+добавлено ~изменено -удалено`); поиск переиспользует сохранённый индекс без повторного эмбеддинга
- **Цель-ориентированные пайплайны** — объявив 30-дневную цель (запуск продукта, исследовательский отчёт, безопасная доставка…), вы автоматически получаете соответствующую лестницу навыков

Архитектура: гексагональная — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. Подробное описание — в `AGENTS.md`.

## Сборка и разработка

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # юнит + e2e
cvp                               # параллельно: check → clippy → test → fmt → build
```

Опциональные фичи: `--features mcp` (MCP-сервер), `--features native-rag` (локальные эмбеддинги), `--features rag-openai` (эмбеддинги OpenAI). Релизные бинарники включают все фичи.

## Лицензия

Apache-2.0.
