> هذا المستند هو النسخة العربية من [README.md](../../../README.md). النسخة الإنجليزية هي المرجع الأصلي.

<div align="center">

[English](../../../README.md) | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | **العربية**

# BuildYourOwnHarness (BYOH)

### وكيلك الذكي، مبنيٌّ حولك

*ليس قالبًا عامًا — بل harness مُجمَّع من دورك وخبرتك وأهدافك.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

معظم إعدادات الذكاء الاصطناعي تسلّمك مجموعة ثابتة من الأدوات وتقول لك «حظًا موفقًا». يدفع BYOH هذا النهج رأسًا على عقب: يستجوبك، يتعلّم ما تفعلله فعلًا، ثم يولّد لك harness وكيل مخصّص — مهارات (skills) ووكلاء (agents) وخطوط أنابيب أهداف (goal pipelines) — يناسب سير عملك منذ اللحظة الأولى.

## لمن هذا المشروع؟

- **المطوّرون** الذين يريدون وكيلًا يعرف مسبقًا حزمتهم التقنية (stack) ونمط اختباراتهم وإيقاق تسليمهم
- **الباحثون** الذين يحتاجون إلى مراجعة للأدبيات وتتبّع الاستشهادات والتوليف بينها في تكامل واحد
- **المبدعون** الذين يريدون شريكًا في الكتابة يطابق أسلوبهم وهيكل مشاريعهم
- **محلّلو الأعمال** الذين يحتاجون أطر قرار وخطوط أنابيب تقارير، لا دردشة خامدة

إن سبق لك أن قلت: «ليت ذكائي الاصطناعي يعرف سياقي فعلًا» — فهذا بالضبط ما يفعله BYOH.

## كيف يعمل في 60 ثانية

صُمِّم BYOH ليُقاد بواسطة وكيلك الذكي — لا بكتابتك للأوامر. ثبّت الـ plugin، ثم تحدّث فحسب. المحادثة *هي* المقابلة، والمعالج (wizard)، والبناء.

```
1. Install the plugin      # Claude Code / Codex / agy — auto-installs the binary
2. "Build me a harness"    # your agent scans your repo and compiles the result
```

في الجلسة التالية، يحمّل المضيف (host) الـ harness تلقائيًا — وكلاء ومهارات وخطوط أنابيب أهداف معدّة خصيصًا لك.

## تثبيت الـ plugin (موصى به)

هل تستخدم **Claude Code أو Codex أو agy**؟ ثبّت الـ plugin. فهو يجمّع خادم MCP و**يثبّت الـ binary تلقائيًا عند أول تحميل** — دون الحاجة إلى سلسلة أدوات Rust أو إعداد يدوي:

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

### هل تستخدم مضيفًا آخر متوافقًا مع MCP؟

يتحدّث BYOH لغة MCP، لذلك يعمل Cursor وZed وContinue وغيرها أيضًا. ثبّت الـ [binary](#installation) مرة واحدة، ثم وجّه مضيفك إلى الخادم:

```bash
byoh serve   # stdio MCP server
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **ملاحظة:** المستودع خاص حاليًا. استخدم المسارات أعلاه. بمجرد أن يصبح عامًا، سيظهر في سوق `epicsagas/plugins` المشترك.

## الوضع المُقاد بالوكيل (Agent-led) — المسار الأساسي

بمجرد اتصال مضيفك، أنت لا تكتب أوامر — بل تتحدّث فحسب. يستدعي وكيلك أدوات MCP الخاصة بـ BYOH مباشرة، والمحادثة *هي* المقابلة والبناء ودورة التطوير (evolve):

> **أنت:** *أنا مطوّر Go خلفيّة (backend) أُطلق واجهة برمجة مدفوعات هذا الشهر. ابنِ لي harness.*
>
> **الوكيل:** *(يبحث مستودعك عبر `profile_scan`، يطرح أسئلة موجّهة قليلة عبر `profile_interview`، يُثبّت الـ genre على `developer`)* ← يُجمّع `HarnessBundle` ← يثبّت وكلاء ومهارات وخط أنابيب هدف secure-ship في Claude Code. تم — في الجلسة التالية، وكيلك يتحدّث لغة حزمتك التقنية فعلًا.

نفس التدفّق، بالترتيب المقترح للأدوات:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → compile → compile_dry_run → render_plugin → install_plugin
           → (optional) registry_clone_skill → (later) evolve_cycle
```

الأدوات المتاحة: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `genre_list`, `compile`, `compile_dry_run`, `render_plugin`, `install_plugin`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`.

هل تريد أن يقودك الوكيل خلال ذلك؟ فقط قُل *"build my harness"* — وكيل `byoh-guide` المُضمَّن ينسّق التدفّق كله.

## فهرس الـ plugins (catalog)

يُبنى الفهرس من README لمستودع [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) — قائمة مجتمعية مرتّبة بحسب النجوم لأفضل 100 مستودع plugin لـ Claude. يأتي BYOH مع حزمة جاهزة (تُعاد بناؤها **أسبوعيًا**، كل اثنين الساعة 03:17 UTC) بحيث ينتهي `byoh catalog index` في ثوانٍ؛ مرّر `--no-bundle` لتحليل القائمة المنبعية مباشرةً.

```bash
# One-time index — downloads a prebuilt bundle in seconds
byoh catalog index

# Search offline — no network needed after indexing
byoh catalog search "memory" --genre developer --limit 5

# Add a plugin to your harness
# license, keywords, and genre are auto-detected from the cloned repo
byoh catalog vendor obra/superpowers --genre developer
```

وكيل الـ LLM (عبر أدوات MCP `catalog_search` / `catalog_vendor`) يستطيع تنفيذ هذه التدفّق كله بشكل ذاتي — *"add a memory plugin to my harness"* — أو يمكنك قيادته مباشرةً من الـ CLI.

بضع أدوات مرافقة تُغرَس في نتائج البحث كـ **مرجع** (لا كتبعيات): أدوات BYOH الخاصة بطبقة التنفيذ — [alcove](https://github.com/epicsagas/alcove) (خادم وثائق)، [obsidian-forge](https://github.com/epicsagas/obsidian-forge) (أتمتة الخزائن)، [epic-harness](https://github.com/epicsagas/epic-harness) (تشغيل hook/skill) — تظهر سياقيًا (استعلام عن «خادم وثائق» / «backend بحث» يجد alcove) بحيث يستطيع الوكيل توصيتها عند الحاجة. أضِف واحدًا (vendor) فقط إن كنت تريده فعلًا؛ الحزم تُشحن بلا تبعيات على أي حال.

## المستخدمون المتقدّمون: الـ CLI (اختياري)

كل تدفّق أعلاه قابل للوصول أيضًا من الطرفية. الـ CLI **مساعد** — مفيد للبرمجة النصية أو CI أو حين لا تفضّل الدردشة — لكن المسار المُقاد بالوكيل هو المسار المقصود.

### أول harness لك — من الـ CLI

```bash
byoh profile init me --paths ./src ./docs   # auto-scans your project
byoh profile confirm me --genre developer   # lock in your genre (+ optional --goal)

byoh compile me                             # gates (static + dry-run) always run; writes the HarnessBundle
byoh render me --target claude              # or: codex | agy | all (default: all)
byoh install me --scope local               # render to dist/, activate into this project's .claude/ only
byoh install me --scope global              # ...or ~/.claude + ~/.codex + ~/.gemini (was --host)
byoh install me --scope publish             # ...or add LICENSE + .gitignore and print git instructions
```

المقابلة نفسها مُقادة بالوكيل (أداة MCP `profile_interview`) — المحادثة هي المقابلة، لذا لا توجد مقابلة تفاعلية في الـ CLI. التطوير (`evolve_cycle` أداة MCP) يشغّل دورة بثلاث بوابات (Critic / Seesaw / Stagnation) لا يمكن تجاوزها أبدًا — وبذلك يكون التطوير آمنًا وقابلًا للتدقيق.

## كيف يعمل تحت الغطاء

محرّك التوليف في BYOH يطابق وسوم ملفّك الشخصي مع سجل المهارات، يرتّبها في خط أنابيب محلول التبعيات، ويُنتج `HarnessBundle` — قطعة (artifact) جاهزة لـ git تُعرَض إلى التنسيق الأصلي لأي مضيف مدعوم.

- **نموذج أمان من 4 حلقات** — مواصفات دورة الحياة (Ring 0) ومهارات خط الأنابيب المدمجة (Ring 1) مرورًا بالمهارات المجتمعية/غير الموثوقة (Ring 3)، لكلٍّ منها تحقّق متصاعد؛ المهارات المُورَّدة (vendored) مُثبَّتة بـ sha256 ومُتحقَّق منها عند القراءة + الإدراج
- **تطوير بثلاث بوابات** — كل دورة `evolve` تمرّ ببوابات Critic (الجودة)، وSeesaw (التراجع)، وStagnation (الهضبة)؛ لا تجاوز
- **خطوط أنابيب موجَّهة بالأهداف** — إعلان هدف ثلاثين يومًا (إطلاق منتج، تقرير بحثي، تسليم آمن…) يُطبِّق تلقائيًا سلّم مهارات مطابقًا

البنية: سداسية — `domain / ports / adapters / application / compiler / evolve / templates / deploy / catalog / mcp / i18n / security / cli`. راجع `AGENTS.md` للدليل الكامل.

## مرجع CLI الكامل

الـ CLI صغير عمدًا: نقاط دخول آلية (`serve`، و`catalog index` في CI، و`vendor` للمشرفين) بالإضافة إلى مرآة قابلة للبرمجة لتدفّق البناء الأساسي. المقابلة والتطوير حصران لـ MCP (مُقادان بالوكيل).

```bash
# Profile
byoh profile init <slug> [--paths ...]      # non-destructive project scan
byoh profile confirm <slug> --genre <g> [--goal <text>]  # confirm and lock profile
byoh profile show <slug>                    # print the profile YAML

# Build (static gate + read-only dry-run gate always run)
byoh compile <slug> [--out <dir>]           # write the HarnessBundle
byoh render <slug> [--target <host>]        # claude | codex | agy | all (default: all)
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

الملفات الشخصية وذاكرة الفهرس توجد تحت `~/.byoh` افتراضيًا (تجاوزها بـ `BYOH_HOME`).

## التثبيت

مطلوب فقط إن كنت **لا** تستخدم الـ plugin (الذي يثبّت الـ binary تلقائيًا) أو أردت BYOH على مضيف MCP لا يعتمد الـ plugins.

### الـ binary (دون الحاجة إلى سلسلة أدوات Rust)

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**من المصدر:**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verify
```

## البناء والتطوير

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unit + e2e
cvp                               # parallel: check → clippy → test → fmt → build
```

ميزة `mcp` (خادم MCP عبر stdio) مفعّلة افتراضيًا. لا يُشحن BYOH قاعدة معرفة مُضمَّنة — للاسترجاع، وجّه harness المُولَّد لديك إلى خادم وثائق مثل [alcove](https://github.com/epicsagas/alcove).

## شكر وتقدير

يقف BYOH على أكتاف جهود مجتمعية عدة:

- **فهرس الـ plugins** — مصدره [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins)، قائمة مجتمعية مرتّبة بحسب النجوم لأفضل 100 مستودع plugin لـ Claude. لولاه، لما وُجد الفهرس.
- **الأدوات المرافقة** — مصمّمة لتتعاون مع [alcove](https://github.com/epicsagas/alcove) (خادم وثائق / RAG)، و[Episteme](https://github.com/epicsagas/Episteme) (رسم معرفي)، و[obsidian-forge](https://github.com/epicsagas/obsidian-forge) (أتمتة الخزائن).
- **حزمة مفتوحة المصدر** — مبنية على [clap](https://docs.rs/clap)، و[serde](https://serde.rs)، و[ureq](https://docs.rs/ureq)، ومنظومة Rust.

عناصر الفهرس والمهارات المجتمعية المُورَّدة تحتفظ برخصها الخاصة (تُكتشَف تلقائيًا وقت الـ vendor). BYOH نفسه تحت رخصة Apache-2.0.

## الرخصة

Apache-2.0.
