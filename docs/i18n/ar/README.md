> هذا المستند هو النسخة العربية من [README.md](../../../README.md). النسخة الإنجليزية هي المرجع الأصلي.

<div align="center">

[English](../../../README.md) | [한국어](../ko/README.md) | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | **العربية**

# BuildYourOwnHarness (BYOH)

### وكيل الذكاء الاصطناعي الخاص بك، مصمم لك

*ليس قالبًا جاهزًا — بل نظام مُجمَّع وفق دورك وخبرتك وأهدافك.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

معظم أدوات الذكاء الاصطناعي تمنحك مجموعة ثابتة من الأدوات وتقول "بال توفيق". BYOH يقلب المعادلة: يقابلك، يتعلّم ما تفعلله فعلاً، ثم يولّد نظام وكيل مخصصًا — مهارات وذاكرة وخطوط معالجة — يناسب طريقة عملك من اللحظة الأولى.

## لمن هذا؟

- **المطورون** الذين يريدون وكيلًا يعرف مسبقًا بنيتهم التقنية وأسلوب الاختبار ودورة التسليم
- **الباحثون** الذين يحتاجون مراجعة الأدبيات وتتبع المصادر والتوليف في خط معالجة واحد متكامل
- **المبدعون** الذين يريدون شريك كتابة يتناسب مع أسلوبهم وهيكل مشروعهم
- **محللو الأعمال** الذين يحتاجون أطر قرار وخطوط تقارير حقيقية، لا مجرد محادثة

إذا خطر ببالك يومًا "ليت ذكائي الاصطناعي يفهم سياقي فعلاً" — فهذا بالضبط ما يقدّمه BYOH.

## كيف يعمل في 60 ثانية

صُمِّم BYOH ليُقاد بواسطة وكيل الذكاء الاصطناعي لديك. ثبّته، وصِل خادمك عبر MCP، ثم تحدّث فقط — المحادثة *هي* المقابلة والمعالج والبناء في آنٍ واحد.

```
1. Install byoh              # تثبيت بسطر واحد (انظر أدناه)
2. Connect your host via MCP # byoh serve — أي وكيل متوافق مع MCP
3. "Build me a harness"      # وكيلك يفحص مستودعك ويُجمّع النتيجة
```

في الجلسة التالية يحمّل خادمك النظام تلقائيًا — وكلاء ومهارات وذاكرة وخطوط معالجة، كلها مضبوطة لك.

**تفضّل الطرفية؟** التدفق نفسه من CLI:
```
byoh profile init me        # يفحص مشروعك — قراءة فقط، لا تعديل
byoh profile interview me   # محادثة قصيرة عن دورك وأهدافك
byoh compile me             # يولّد نظامك الشخصي
byoh install me             # ينشره في Claude / Codex / agy
```

**تعرف ما تحتاجه مسبقًا؟** تصفّح كتالوج المجتمع:
```bash
byoh catalog index                                 # تنزيل قائمة أفضل 100 إضافة (ثوانٍ)
byoh catalog search "code review"                  # ابحث عن الإضافات المناسبة
byoh catalog vendor anthropics/claude-code-review  # أضف واحدة إلى نظامك
```

## التثبيت

### البرنامج الثنائي (موصى به — لا يتطلب سلسلة أدوات Rust)

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

### وصّل خادم الذكاء الاصطناعي

يتحدّث BYOH لغة MCP، لذا أي وكيل متوافق مع MCP يمكنه قيادته. ثبّت البرنامج الثنائي أعلاه، شغّل الخادم، فيستدعي خادمك كل أداة في BYOH مباشرةً:

```bash
byoh serve   # خادم MCP عبر stdio
```

للخوادم **الأخرى** (Cursor، Zed، Continue، …)، أضف `byoh` إلى إعداد MCP في خادمك:
```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

هل تستخدم **Claude Code أو Codex أو agy**؟ ثبّت الإضافة بدلاً من ذلك — فهي تجمع خادم MCP وتُثبّت البرنامج الثنائي تلقائيًا عند أول تحميل (دون الحاجة إلى Rust):

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

> **ملاحظة:** المستودع خاص حاليًا. استخدم المسارات أعلاه. عند الإتاحة للعموم سيظهر في سوق `epicsagas/plugins` المشترك.

## وضع قيادة الوكيل

بمجرد وصل خادمك، أنت لا تكتب الأوامر — أنت فقط تتحدّث. يستدعي وكيلك أدوات BYOH الأربع عشرة مباشرةً، والمحادثة *هي* المقابلة والبناء ودورة التطور:

الأدوات المتاحة: `profile_create`، `profile_scan`، `profile_interview`، `profile_confirm`، `compile`، `evolve_cycle`، `genre_list`، `registry_clone_skill`، `catalog_search`، `catalog_vendor` وغيرها.

## أول نظام لك — من CLI

الخطوات نفسها، تُقاد من الطرفية:

### الخطوة 1 — الملف الشخصي
```bash
byoh profile init me --paths ./src ./docs   # فحص تلقائي للمشروع
byoh profile interview me                   # محادثة ~5 دقائق
byoh profile confirm me --genre developer   # تثبيت تصنيفك
```

يسألك BYOH عن دورك ومستوى خبرتك وأدواتك وهدفك لمدة 30 يومًا. تتكيّف المقابلة — فالباحث يحصل على أسئلة مختلفة عن المطور.

### الخطوة 2 — التجميع والتثبيت
```bash
byoh compile me          # توليد HarnessBundle (مُتحقَّق + مُبوَّب)
byoh render me --target claude   # أو: codex | agy | all
byoh install me          # تثبيت آمن في dist/
```

### الخطوة 3 — التشغيل والتطور
```bash
byoh run me              # التشغيل مع تفعيل نظامك
byoh evolve me           # تحسين النظام بناءً على تغذية الجلسة الراجعة
```

يُشغّل `evolve` دورة ثلاثية البوابات (Critic / Seesaw / Stagnation) لا يمكن تجاوزها أبدًا — فالتطور آمن وقابل للمراجعة.

## كتالوج الإضافات

يمنحك الكتالوج قائمة منتقاة بأفضل 100 إضافة لـ Claude (مرتبة حسب النجوم، تُحدَّث يوميًا) لتكتشف مهارات المجتمع وتضيفها دون مغادرة الطرفية.

```bash
# فهرسة مرة واحدة — تنزيل حزمة جاهزة في ثوانٍ
byoh catalog index

# بحث دون اتصال — لا شبكة بعد الفهرسة
byoh catalog search "memory" --genre developer --limit 5

# إضافة إضافة إلى نظامك
# الترخيص والكلمات المفتاحية والتصنيف تُكتشف تلقائيًا من المستودع المستنسخ
byoh catalog vendor obra/superpowers --genre developer
```

يستطيع وكيل LLM (عبر أدوات MCP: `catalog_search` / `catalog_vendor`) تنفيذ هذا التدفق كاملاً باستقلالية — أو يمكنك توجيهه مباشرةً من CLI.

## مرجع CLI الكامل

```bash
# الملف الشخصي
byoh profile init <slug> [--paths ...]      # فحص المشروع (قراءة فقط)
byoh profile interview <slug>               # مقابلة موجّهة
byoh profile confirm <slug> --genre <g>     # تأكيد الملف الشخصي وتثبيته

# البناء
byoh compile <slug> [--dry-run]             # تحقق + توليد HarnessBundle
byoh render <slug> --target <host>          # claude | codex | agy | all
byoh install <slug> [--host <dir>]          # نشر في dist/ أو مجلد الإضافة الفعلي

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

## كيف يعمل من الداخل

يطابق محرك التوليف في BYOH وسوم ملفك الشخصي مع سجل المهارات، ويرتّبها في خط معالجة محلول التبعيات، ثم يُصدر `HarnessBundle` — قطعة جاهزة لـ git تُصيَّر بالصيغة الأصلية لأي خادم مدعوم.

- **نموذج أمان رباعي الحلقات** — من المهارات المدمجة (Ring 1) إلى مهارات المجتمع/غير الموثوقة (Ring 4)، مع تصاعد التحقّق في كل حلقة
- **تطور ثلاثي البوابات** — كل دورة `evolve` تمر ببوابات Critic (الجودة) و Seesaw (الانحدار) و Stagnation (الركود)؛ لا تجاوز ممكن
- **خطوط معالجة موجّهة بالأهداف** — بمجرد تحديد هدف 30 يومًا (إطلاق منتج، تقرير بحثي، شحن آمن…) تُطبَّق سلم المهارات المناسب تلقائيًا

البنية: سداسية — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. الدليل الكامل في `AGENTS.md`.

## البناء والتطوير

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # وحدات + e2e
cvp                               # متوازي: check → clippy → test → fmt → build
```

ميزة `mcp` (خادم MCP عبر stdio) مُفعّلة افتراضيًا. لا يُضمّن BYOH قاعدة معرفة مدمجة — للاسترجاع، وجّه نظامك المولّد نحو خادم وثائق مثل [alcove](https://github.com/epicsagas/alcove).

## الرخصة

Apache-2.0.
