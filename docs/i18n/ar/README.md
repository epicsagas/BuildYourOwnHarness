> هذا المستند هو النسخة العربية من [README.md](../../../README.md). النسخة الإنجليزية هي المرجع الأصلي.

<div align="center">

[English](../../../README.md) | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | **العربية**

# BuildYourOwnHarness (BYOH)

### وكيل الذكاء الاصطناعي الخاص بك، مصمّم على مقاسك

*ليس قالبًا جاهزًا — بل هارنس (harness) مُجمَّع وفق دورك وخبرتك وأهدافك.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

معظم أدوات الذكاء الاصطناعي تمنحك مجموعة ثابتة من الأدوات وتقول لك "حظًا موفقًا." يقلب BYOH هذه القاعدة: يقابلك، يتعلّم ما تفعله فعلًا، ثم يولّد الهارنس وكيلًا مخصّصًا — مهارات وذاكرة وخطوط معالجة — يناسب طريقة عملك من اللحظة الأولى.

## لمن هذا المشروع؟

- **المطورون** الذين يريدون وكيلًا يعرف مسبقًا بنيتهم التقنية وأسلوب الاختبار ودورة التسليم
- **الباحثون** الذين يحتاجون إلى مراجعة الأدبيات وتتبع الاستشهادات والتوليف في خط معالجة واحد متكامل
- **المبدعون** الذين يريدون شريك كتابة يتناسب مع أسلوبهم وهيكل مشروعهم
- **محللو الأعمال** الذين يحتاجون أطر قرار وخطوط تقارير حقيقية، لا مجرد محادثة خام

إن خطر ببالك يومًا "ليت ذكائي الاصطناعي يعرف سياقي فعلًا" — فهذا بالضبط ما يقدّمه BYOH.

## كيف يعمل في 60 ثانية

صُمِّم BYOH ليُقاد بواسطة وكيل الذكاء الاصطناعي لديك — لا بكتابتك للأوامر. ثبّت الإضافة، ثم تحدّث فقط. المحادثة *هي* المقابلة ومعالج الإعداد والبناء.

```
1. Install the plugin      # Claude Code / Codex / agy — يُثبّت البرنامج الثنائي تلقائيًا
2. "Build me a harness"    # وكيلك يفحص مستودعك ويُجمّع النتيجة
```

في الجلسة التالية يحمّل مضيفك الهارنس تلقائيًا — وكلاء ومهارات وذاكرة وخطوط معالجة، كلها مضبوطة لك.

## ثبّت الإضافة (موصى به)

هل تستخدم **Claude Code أو Codex أو agy**؟ ثبّت الإضافة. فهي تجمع خادم MCP **وتُثبّت البرنامج الثنائي تلقائيًا عند أول تحميل** — فلا حاجة لسلسلة أدوات Rust ولا لإعداد يدوي:

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

### هل تستخدم خادمًا آخر متوافقًا مع MCP؟

يتحدّث BYOH لغة MCP، لذا يعمل مع Cursor وZed وContinue وغيرها. ثبّت [الملف الثنائي](#التثبيت) مرة واحدة، ثم وجّه خادمك نحو الخادم:

```bash
byoh serve   # خادم MCP عبر stdio
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **ملاحظة:** المستودع خاص حاليًا. استخدم المسارات أعلاه. عند الإتاحة للعموم سيظهر في سوق `epicsagas/plugins` المشترك.

## وضع قيادة الوكيل — المسار الرئيسي

بمجرد أن يتصل مضيفك، أنت لا تكتب الأوامر — أنت تتحدّث فقط. يستدعي وكيلك أدوات BYOH MCP مباشرةً، والمحادثة *هي* المقابلة والبناء ودورة التطور:

> **أنت:** *أنا مطوّر Go خلفيّ أُطلق واجهة برمجية للمدفوعات هذا الشهر. ابنِ لي هارنس.*
>
> **الوكيل:** *(يفحص مستودعك عبر `profile_scan`، ويطرح أسئلة موجّهة عبر `profile_interview`، ثم يُثبّت التصنيف على `developer`)*، ثم يُجمّع `HarnessBundle`، ثم يُثبّت وكلاء ومهارات وذاكرة وخط شحن آمن في Claude Code. انتهى — في الجلسة التالية، وكيلك يتقن حِزَمتك التقنية.

التدفق نفسه، بالترتيب المقترح للأدوات:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → compile → compile_dry_run → render_plugin → install_plugin
           → (اختياري) registry_clone_skill → (لاحقًا) evolve_cycle
```

الأدوات المتاحة: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `genre_list`, `compile`, `compile_dry_run`, `render_plugin`, `install_plugin`, `evolve_cycle`, `registry_clone_skill`, `catalog_search`, `catalog_vendor`.

هل تريد أن يأخذك الوكيل خطوة بخطوة؟ فقط قُل *"build my harness"* — فوكيل `byoh-guide` المُدمج ينسّق التدفق كاملًا.

## كتالوج الإضافات

يُبنى الكتالوج من README لمستودع [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) — قائمة مجتمعية مرتّبة حسب النجوم لأفضل 100 مستودع لإضافات Claude. يوفّر BYOH حزمة جاهزة (تُعاد بنيتها **أسبوعيًا**، كل اثنين 03:17 بالتوقيت العالمي) بحيث يُنهي `byoh catalog index` عمله في ثوانٍ؛ مرّر `--no-bundle` لتحليل القائمة الأصلية مباشرةً.

```bash
# فهرسة مرة واحدة — تنزيل حزمة جاهزة في ثوانٍ
byoh catalog index

# بحث دون اتصال — لا حاجة للشبكة بعد الفهرسة
byoh catalog search "memory" --genre developer --limit 5

# إضافة إضافة إلى هارنسك
# الترخيص والكلمات المفتاحية والتصنيف تُكتشف تلقائيًا من المستودع المستنسخ
byoh catalog vendor obra/superpowers --genre developer
```

يستطيع وكيل LLM (عبر أدوات MCP: `catalog_search` / `catalog_vendor`) تنفيذ هذا التدفق كاملًا باستقلالية — فقط *"add a memory plugin to my harness"* — أو يمكنك توجيهه مباشرةً من CLI.

## المستخدمون المتقدّمون: الـ CLI (اختياري)

كل تدفّق مما سبق متاح أيضًا من الطرفية. الـ CLI **مساعد فقط** — مفيد للسكربتات وCI أو حين لا ترغب في المحادثة — لكن مسار قيادة الوكيل هو المسار المقصود.

### أول هارنس لك — من CLI

```bash
byoh profile init me --paths ./src ./docs   # فحص تلقائي لمشروعك
byoh profile interview me                   # محادثة ~5 دقائق
byoh profile confirm me --genre developer   # تثبيت تصنيفك

byoh compile me --no-dry-run                # يتحقق + يكتب HarnessBundle (dry-run هو الافتراضي)
byoh render me --target claude              # أو: codex | agy | all (الافتراضي: all)
byoh install me --scope local               # يُصيّر إلى dist/ ويُفعّله فقط في .claude/ لهذا المشروع
byoh install me --scope global              # ...أو ‎~/.claude + ~/.codex + ~/.gemini (سابقاً --host)
byoh install me --scope publish             # ...أو يُضيف LICENSE + .gitignore ويطبع تعليمات git

byoh run me                                 # التشغيل مع تفعيل هارنسك
byoh evolve me                              # تحسين الهارنس بناءً على تغذية الجلسة الراجعة
```

يسألك BYOH عن دورك ومستوى خبرتك وأدواتك وهدفك لمدة 30 يومًا. تتكيّف المقابلة — فالباحث يحصل على أسئلة مختلفة عن المطور. يُشغّل `evolve` دورة من ثلاث بوابات (Critic / Seesaw / Stagnation) لا يمكن تجاوزها أبدًا — فالتطور آمن وقابل للمراجعة.

## كيف يعمل من الداخل

يطابق محرك التوليف في BYOH وسوم ملفك الشخصي مع سجل المهارات، ويرتّبها في خط معالجة محلول التبعيات، ثم يُصدر `HarnessBundle` — قطعة جاهزة لـ git تُصيَّر بالصيغة الأصلية لأي خادم مدعوم.

- **نموذج أمان رباعي الحلقات** — من المهارات المدمجة (Ring 1) إلى مهارات المجتمع/غير الموثوقة (Ring 4)، مع تصاعد التحقّق في كل حلقة
- **تطور ثلاثي البوابات** — كل دورة `evolve` تمر ببوابات Critic (الجودة) و Seesaw (الانحدار) و Stagnation (الركود)؛ لا تجاوز ممكن
- **خطوط معالجة موجّهة بالأهداف** — بمجرد تحديد هدف 30 يومًا (إطلاق منتج، تقرير بحثي، شحن آمن…) تُطبَّق سلم المهارات المناسب تلقائيًا

البنية: سداسية — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. الدليل الكامل في `AGENTS.md`.

## مرجع CLI الكامل

```bash
# الملف الشخصي
byoh profile init <slug> [--paths ...]      # فحص المشروع دون تعديل
byoh profile interview <slug>               # مقابلة موجّهة
byoh profile confirm <slug> --genre <g>     # تأكيد الملف الشخصي وتثبيته

# البناء
byoh compile <slug> [--no-dry-run]          # dry-run افتراضي؛ استخدم --no-dry-run لكتابة الحزمة
byoh render <slug> [--target <host>]        # claude | codex | agy | all (الافتراضي: all)
byoh install <slug> [--target <host>] [--scope local|global|publish] [--host] [--force]  # شجرة dist/؛ --scope يُحدد الوجهة (local=هذا المشروع، global=HOME، publish=+LICENSE/.gitignore+خطوات git). --host هو الاسم القديم لـ --scope global.

# التشغيل والتطور
byoh run <slug>
byoh evolve <slug>

# مهارات المجتمع
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# الكتالوج
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<query>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## التثبيت

مطلوب فقط إذا كنت **لا** تستخدم الإضافة (التي تُثبّت البرنامج الثنائي تلقائيًا) أو إذا أردت BYOH على خادم MCP لا يدعم الإضافات.

### الملف الثنائي (لا يتطلب سلسلة أدوات Rust)

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
byoh --version   # تحقق
```

## البناء والتطوير

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # وحدات + e2e
cvp                               # متوازي: check → clippy → test → fmt → build
```

ميزة `mcp` (خادم MCP عبر stdio) مُفعّلة افتراضيًا. لا يُضمّن BYOH قاعدة معرفة مدمجة — للاسترجاع، وجّه هارنسك المولّد نحو خادم وثائق مثل [alcove](https://github.com/epicsagas/alcove).

## شكر وتقدير

يقف BYOH على أكتاف جهود مجتمعية عديدة:

- **كتالوج الإضافات** — مصدره [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins)، قائمة مجتمعية مرتّبة حسب النجوم لأفضل 100 مستودع لإضافات Claude. بدونه، لما وُجد الكتالوج.
- **أدوات مرافقة** — مصمّم للعمل مع [alcove](https://github.com/epicsagas/alcove) (خادم وثائق/RAG) و[Episteme](https://github.com/epicsagas/Episteme) (رسم معرفي) و[obsidian-forge](https://github.com/epicsagas/obsidian-forge) (أتمتة المخازن).
- **حزمة مفتوحة المصدر** — مبني على [clap](https://docs.rs/clap) و[serde](https://serde.rs) و[ureq](https://docs.rs/ureq) ومنظومة Rust.

تحتفظ مدخلات الكتالوج والمهارات المجتمعية المُدمجة برخصها الخاصة (تُكتشف تلقائيًا عند الدمج). BYOH نفسه تحت Apache-2.0.

## الرخصة

Apache-2.0.
