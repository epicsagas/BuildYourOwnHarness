> هذا المستند هو النسخة العربية من [README.md](../../../README.md). النسخة الإنجليزية هي المرجع الأصلي.

# BuildYourOwnHarness (BYOH)

> **وكيل الذكاء الاصطناعي الخاص بك، مصمم لك** — ليس قالبًا جاهزًا، بل نظام مُجمَّع وفق دورك ومهاراتك وأهدافك.

معظم أدوات الذكاء الاصطناعي تعطيك مجموعة ثابتة من الميزات وتقول "وفّق نفسك". BYOH يقلب المعادلة: مقابلة قصيرة تتعلم كيف تعمل فعلاً، ثم تولّد نظام وكيل مخصصًا — مهارات وذاكرة وخطوط معالجة — يناسب طريقة عملك من اللحظة الأولى.

## لمن هذا؟

- **المطورون** — وكيل يعرف مسبقًا بنيتك التقنية وأسلوب الاختبار ودورة التسليم
- **الباحثون** — مراجعة الأدبيات وتتبع المصادر والتوليف في خط معالجة واحد متكامل
- **المبدعون** — شريك كتابة يتناسب مع أسلوبك وهيكل مشروعك
- **محللو الأعمال** — أطر قرار وخطوط تقارير حقيقية، لا مجرد محادثة

إذا خطر ببالك يومًا "ليتني أملك ذكاءً اصطناعيًا يفهم سياقي فعلاً" — فهذا بالضبط ما يقدمه BYOH.

## ابدأ في 60 ثانية

```
1. byoh profile init me        # يفحص مشروعك — قراءة فقط، لا تعديل
2. byoh profile interview me   # محادثة قصيرة عن دورك وأهدافك
3. byoh compile me             # يولّد نظامك الشخصي
4. byoh install me             # ينشره في Claude / Codex / agy
```

في الجلسة التالية يحمّل الخادم نظامك تلقائيًا — وكلاء ومهارات وذاكرة وخطوط معالجة، كلها مضبوطة لك.

**تعرف ما تحتاجه؟** تصفّح كتالوج المجتمع مباشرةً:
```bash
byoh catalog index                              # تنزيل قائمة أفضل 100 إضافة (ثوانٍ)
byoh catalog search "code review"               # ابحث عما تحتاجه
byoh catalog vendor anthropics/claude-code-review   # أضفه إلى نظامك
```

## التثبيت

### البرنامج الثنائي (موصى به — لا يتطلب Rust)

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
byoh --version   # تحقق من التثبيت
```

### تحميل الإضافة في خادم الذكاء الاصطناعي

BYOH إضافة متعددة اللغات تعمل مع Claude Code و Codex و agy — مستودع واحد للأنظمة الثلاثة.

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

تثبّت الإضافة الملف الثنائي `byoh` تلقائيًا عند أول تشغيل — لا حاجة لـ Rust على جهازك.

> **ملاحظة:** المستودع خاص حاليًا. استخدم المسارات أعلاه. عند الإتاحة للعموم سيظهر في سوق `epicsagas/plugins`.

## أول نظام لك — خطوة بخطوة

### الخطوة 1 — الملف الشخصي
```bash
byoh profile init me --paths ./src ./docs   # فحص تلقائي للمشروع
byoh profile interview me                   # محادثة ~5 دقائق
byoh profile confirm me --genre developer   # تثبيت التصنيف
```

يسألك BYOH عن دورك ومستوى خبرتك وأدواتك وهدفك لمدة 30 يومًا. تتكيف المقابلة — الباحث يحصل على أسئلة مختلفة عن المطور.

### الخطوة 2 — التجميع والتثبيت
```bash
byoh compile me                  # توليد HarnessBundle (تحقق + بوابات)
byoh render me --target claude   # أو: codex | agy | all
byoh install me                  # تثبيت آمن في dist/
```

### الخطوة 3 — التشغيل والتطور
```bash
byoh run me       # التشغيل مع تفعيل النظام
byoh evolve me    # تحسين النظام بناءً على تغذية راجعة
```

يُشغّل `evolve` دورة ثلاثية البوابات (Critic / Seesaw / Stagnation) لا يمكن تجاوزها — التطور آمن وقابل للمراجعة.

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

يستطيع وكيل الذكاء الاصطناعي (عبر أدوات MCP: `catalog_search` / `catalog_vendor`) تنفيذ هذا التدفق كاملاً باستقلالية — أو يمكنك توجيهه مباشرةً من CLI.

## وضع الوكيل

يشغّل `byoh serve` خادم MCP عبر stdio. بدلاً من كتابة الأوامر يدويًا، يستدعي خادم الذكاء الاصطناعي 14 أداة من BYOH مباشرةً — المحادثة *هي* المقابلة والمعالج والتنفيذ في آنٍ واحد.

```bash
byoh serve   # يتصل Claude / Codex / agy ويتولى كل شيء
```

الأدوات المتاحة: `profile_create`، `profile_scan`، `profile_interview`، `profile_confirm`، `compile`، `evolve_cycle`، `rag_index`، `rag_search`، `genre_list`، `registry_clone_skill`، `catalog_search`، `catalog_vendor` وغيرها.

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
byoh catalog search "<استعلام>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]

# قاعدة المعرفة (RAG)
byoh index <slug> [--corpus <dir>] [--force]
byoh search <slug> "<استعلام>" [--genre <g>] [--k N]
```

## كيف يعمل من الداخل

يطابق محرك التوليف في BYOH وسوم ملفك الشخصي مع سجل المهارات، ويرتبها في خط معالجة محلول التبعيات، ثم يصدر `HarnessBundle` — مصنوع للـ git ويُصيَّر بالصيغة الأصلية لأي خادم مدعوم.

- **4 حلقات أمان** — من المهارات المدمجة (Ring 1) إلى مهارات المجتمع/غير الموثوقة (Ring 4)، مع تصاعد التحقق في كل حلقة
- **3 بوابات تطور** — كل دورة `evolve` تمر ببوابات Critic (الجودة) و Seesaw (الانحدار) و Stagnation (الركود)؛ لا تجاوز ممكن
- **RAG مستمر** — إعادة تضمين تدريجية عند التغييرات (`+مضاف ~متغير -محذوف`)؛ يعيد البحث استخدام الفهرس المحفوظ دون إعادة تضمين
- **خطوط معالجة موجّهة بالأهداف** — بمجرد تحديد هدف 30 يومًا (إطلاق منتج، تقرير بحثي، شحن آمن…) تُطبَّق سلم المهارات المناسب تلقائيًا

البنية: سداسية — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. الدليل الكامل في `AGENTS.md`.

## البناء والتطوير

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # وحدات + e2e
cvp                               # متوازي: check → clippy → test → fmt → build
```

ميزات اختيارية: `--features mcp` (خادم MCP)، `--features native-rag` (تضمين محلي)، `--features rag-openai` (تضمين OpenAI). الثنائيات الإصدارية تتضمن جميع الميزات.

## الرخصة

Apache-2.0.
