> هذه ترجمة لـ [README.md](../../README.md). النسخة الإنجليزية هي المصدر الموثوق وقد تكون أحدث.
>
> ⚠️ Auto-translation pending — the English source below awaits translation via the i18n workflow.

# BuildYourOwnHarness (BYOH)

> Interactively collect a user's tacit knowledge, data, business genre, and goals — then **generate, deploy, operate, and evolve a personalized AI agent harness**.

BYOH adds a **generation layer** on top of the validated building blocks of the [epiccounty](https://github.com/epicsagas) workspace. Instead of shipping a fixed skill/memory/pipeline set, it compiles a *unique* harness per user from an interview.

## What it does

A confirmed user profile (genre + expertise + 30-day goal) drives a synthesis engine that **recombines registry skills by keyword** into an ordered pipeline, producing a `HarnessBundle` that is *not* a fixed genre template. The whole pipeline is closed-loop and gated by three safety gates (Critic / Seesaw / Stagnation) that can never be bypassed.

```
profile (interview → genre → confirm) → compile → synthesize → render → install → run → evolve
```

## Build & verify

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                       # unit + e2e
./target/release/byoh --help
```

Hexagonal architecture: `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`.

## CLI

```bash
byoh profile init <slug> [--paths ...]   # S1 autoscan (non-destructive)
byoh profile interview <slug>            # S2 interview (Suggest + Council)
byoh profile confirm <slug> --genre <g>  # S3 wizard confirm
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>
byoh compile <slug> [--dry-run]          # static gate + dry-run gate → HarnessBundle
byoh render <slug> --target claude       # claude | codex | agy | all (git-ready)
byoh install <slug>                      # safe dist/ install (--host for live plugin dir)
byoh run <slug>
byoh evolve <slug>                       # 3-gate evolution cycle
byoh catalog index [--limit N]           # تحليل README أفضل-100 من quemsah → ~/.byoh/catalog.json
byoh catalog search "<استعلام>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

### وضع الأتمتة عبر MCP (خادم MCP)

`byoh serve` (`--features mcp`) يُشغّل خادم MCP عبر stdio ليتولى وكيل LLM **قيادة BYOH** مباشرةً — يصبح سطر الأوامر ثانويًا (عكس التحكم). 14 أداة (`profile_*`، `rag_*`، `genre_list`، `compile`، `evolve_cycle`، `registry_clone_skill`، `catalog_search`، `catalog_vendor`) قابلة للاكتشاف عبر `tools/list`. المحادثة *هي* المقابلة/المعالج.

```bash
cargo build --release --features mcp
byoh serve
```

## الجوهر: التوليف والاستيراد وكتالوج المكونات

- **محرك التوليف** — `synthesize(profile)` يطابق مهارات السجل مع وسوم الملف الشخصي، يرتّبها في خط أنابيب، ويُجبر على إعادة المرور عبر بوابات الأمان الثلاث (بلا تجاوز). تُوفّر خطوط الأنابيب الموجّهة بالأهداف (إطلاق منتج / قرار / تقرير بحثي / شحن آمن / …) سلّمًا من المهارات ومجموعة وكلاء عند تطابق هدف الـ 30 يومًا.
- **استيراد مهارات المجتمع** (RFC M3) — `byoh vendor add` يجلب ملف `SKILL.md` خارجيًا (مسار محلي أو رابط git)، يُجري التحقق الثابت + sha256، ويُدمجه في **Ring 3** (الأشد تقييدًا) وقت البناء عبر `build.rs`. تنضم المهارات الخارجية إلى التوليف كشيفرة غير موثوقة.
- **كتالوج المكونات** — `byoh catalog index` يبني ذاكرة تخزين مؤقت في `~/.byoh/catalog.json` من README منسّق [quemsah/awesome-claude-plugins](https://github.com/quemsah/awesome-claude-plugins) (أفضل 100 حسب النجوم، يُحدَّث يوميًا). جلب + تحليل واحد (بدون زحف صفحة بصفحة)، وكل مدخل يحمل `stars` حقيقية. افتراضيًا يحمّل أولاً **حزمة جاهزة من المسؤول** (أصل GitHub Release أسبوعي — ثوانٍ) ولا يحلّل README مباشرة إلا عند عدم توفرها. بعد الفهرسة يعمل `catalog search` و`catalog vendor` بالكامل بدون اتصال. خلال مقابلة المعالج S2، يُضمّن `profile_interview` تلقائيًا `catalog_suggestions` — ما يصل إلى 5 مكونات مطابقة للنوع يمكن للـ LLM التوصية بها دون استدعاءات أدوات إضافية.

  يقوم `catalog vendor` **بإثراء ذاكرة التخزين المؤقت للكتالوج عند الاستيراد**: بعد استنساخ مستودع المكوِّن، يستخرج `license` و`keywords` من `.claude-plugin/plugin.json` ويسجّل `genre` المحدَّد. تُكتب هذه القيم إلى `catalog.json` فقط عندما تكون القيمة المخزّنة `"unknown"` أو فارغة، مما يجعل نتائج `catalog search` أكثر ثراءً مع كل عملية استيراد. يستطيع وكيل LLM تنفيذ سير العمل بأكمله (بحث → استيراد) بشكل مستقل عبر أدوات MCP `catalog_search` / `catalog_vendor`، أو يمكن للمستخدم تحديد ذلك مباشرةً عبر واجهة سطر الأوامر.

  ```bash
  # فهرسة لمرة واحدة — تحميل الحزمة أولاً، ثم تحليل README كخيار احتياطي
  byoh catalog index                       # الحزمة أولاً، README كخيار احتياطي
  byoh catalog index --no-bundle           # تحليل README مباشرةً
  byoh catalog index --no-bundle --limit 20   # أول 20 فقط

  # تجاوز للاختبار مع مرآة محلية:
  #   BYOH_BUNDLE_URL=http://localhost:18099/catalog.json.gz byoh catalog index

  # بحث بدون اتصال
  byoh catalog search "test driven development" --genre developer --limit 5

  # استيراد المكوِّن إلى registry/vendored/ (يُستخرج license وkeywords وgenre تلقائيًا)
  byoh catalog vendor obra/superpowers --genre developer
  ```

## Status

Rust implementation of the generation layer: profiler + interview + genre templates + compiler (4-ring, MCP-tool codegen, static gate) + evolution engine + self-contained RAG (optional `native-rag` feature) + MCP server (optional `mcp` feature). See `AGENTS.md` for the architecture guide.

The RAG layer is a **persistent knowledge base**: `byoh index` saves the genre index + a corpus sidecar under `$BYOH_HOME/indexes/`, and a later `byoh search` (or the `rag_search` MCP tool) with no `--corpus` reuses it via `load_index` — no re-embedding. Re-indexing is **incremental** — a content-hash manifest re-embeds only added/changed docs and drops removed ones (reported as `+a ~c -r`); `--force` does a full rebuild.

## License

Apache-2.0.
