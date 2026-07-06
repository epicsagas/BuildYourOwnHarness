> 이 문서는 [README.md](../../../README.md)의 한국어 번역입니다. 영문 버전이 권위 있는 원본입니다.

<div align="center">

**[English](../../../README.md)** | **한국어** | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### 나를 중심으로 만들어지는 AI 에이전트

*범용 템플릿이 아니라, 내 역할·전문 분야·목표에 맞게 컴파일되는 하네스.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

대부분의 AI 도구는 고정된 기능 묶음을 주고 "알아서 써보세요"라고 합니다. BYOH는 정반대입니다. 짧은 인터뷰로 실제로 어떤 일을 하는지 파악하고, 그에 맞는 에이전트 하네스 — 스킬, 에이전트, 목표 파이프라인 — 를 자동으로 생성합니다. 바로 꺼내 쓸 수 있도록 내 워크플로에 맞춰진 상태로요.

## 이런 분께 맞습니다

- **개발자** — 내 스택, 테스트 스타일, 배포 주기를 이미 아는 에이전트가 필요하다면
- **연구자** — 문헌 검색, 인용 추적, 합성이 하나로 연결된 파이프라인이 필요하다면
- **크리에이터** — 내 문체와 프로젝트 구조에 맞춰진 글쓰기 파트너가 필요하다면
- **비즈니스 분석가** — 날것의 채팅이 아닌, 의사결정 프레임워크와 보고서 파이프라인이 필요하다면

"AI가 내 맥락을 좀 알아줬으면..."이라는 생각을 해봤다면, BYOH가 바로 그걸 합니다.

## 60초 시작

BYOH는 AI 에이전트가 주도하도록 설계됐습니다 — 사용자가 명령어를 치는 게 아니라요. 플러그인을 설치하고, 그냥 대화하세요. 대화가 곧 인터뷰이자 위자드이자 빌드입니다.

```
1. Install the plugin      # Claude Code / Codex / agy — 바이너리까지 자동 설치
2. "Build me a harness"    # 에이전트가 레포를 스캔해서 번들을 빌드
```

다음 세션부터 호스트가 하네스를 자동으로 로드합니다 — 에이전트, 스킬, 목표 파이프라인 모두 나에게 맞춰진 상태로.

## 플러그인으로 설치 (권장)

**Claude Code, Codex, agy**를 쓰시나요? 플러그인으로 설치하세요. MCP 서버를 함께 묶어서 **첫 로드 시 바이너리까지 자동 설치**합니다 — Rust 툴체인도, 수동 설정도 필요 없습니다:

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

### 그 외 MCP 호환 호스트를 쓰신다면?

BYOH는 MCP를 사용하므로, Cursor, Zed, Continue 같은 호스트에서도 잘 동작합니다. [바이너리](#설치)를 한 번 설치한 뒤, 호스트가 서버를 바라보게 하면 됩니다:

```bash
byoh serve   # stdio MCP 서버
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **참고:** 현재 레포는 비공개입니다. 위 경로를 사용하세요. 공개 후에는 공용 `epicsagas/plugins` 마켓플레이스에도 등록됩니다.

## 에이전트 주도 모드 — 메인 경로

호스트가 연결되면 명령어를 치지 않아도 됩니다 — 그냥 대화하세요. 에이전트가 BYOH의 MCP 도구를 직접 호출하고, 대화가 곧 인터뷰이자 빌드이자 진화 사이클입니다:

> **나:** *이번 달에 결제 API를 출시할 백엔드 Go 개발자야. 하네스 하나 만들어줘.*
>
> **에이전트:** *(`profile_scan`으로 레포를 스캔하고, `profile_interview`로 타깃팅된 질문 몇 개를 던지고, 장르를 `developer`로 고정)* → `HarnessBundle`을 빌드(synthesize + matched/skeleton 스킬 분류) → 에이전트, 스킬, 보안 배포 목표 파이프라인을 Claude Code에 설치. 완료 — 다음 세션부터 에이전트가 내 스택을 이미 알고 있습니다.

같은 흐름을, 권장 도구 호출 순서대로 보면:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → build → install_plugin
```

`build`는 번들을 합성(컴파일 + 프리셋 주입 + static 게이트)하면서, 각 스킬을 `matched`(실제 프리셋 본문) 또는 `skeleton`(장르 템플릿 자리표시자)으로 분류합니다 — 에이전트는 이 결과를 보고 지금 설치할지, 프로파일을 더 다듬을지 결정합니다.

사용 가능한 도구: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `build`, `render_plugin`, `install_plugin`, `catalog_search`, `catalog_vendor`.

에이전트가 처음부터 끝까지 안내해 주길 원하신다면, 그냥 *"build my harness"*라고 말하세요 — 번들된 `byoh-guide` 에이전트가 전체 흐름을 오케스트레이션합니다.

## 플러그인 카탈로그

카탈로그는 [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) README에서 만듭니다 — 커뮤니티가 관리하는, 상위 100개 Claude 플러그인 레포지토리의 Stars 순 목록입니다. BYOH는 사전 빌드 번들을(매주 월요일 03:17 UTC에 갱신) 제공해서 `byoh catalog index`가 수 초 만에 끝나며, `--no-bundle`을 주면 upstream 목록을 직접 파싱합니다.

```bash
# 최초 1회 인덱싱 — 사전 빌드 번들을 수 초 만에 다운로드
byoh catalog index

# 인덱싱 후에는 네트워크 없이 검색 가능
byoh catalog search "memory" --genre developer --limit 5

# 하네스에 플러그인 추가
# license, keywords, genre는 클론한 레포에서 자동 추출
byoh catalog vendor obra/superpowers --genre developer
```

LLM 에이전트가(`catalog_search` / `catalog_vendor` MCP 도구로) 이 흐름 전체를 자율으로 처리할 수도 있습니다 — *"내 하네스에 메모리 플러그인 추가해줘"* — 물론 CLI에서 직접 실행할 수도 있습니다.

몇 가지 컴패니언 도구가 **참고 자료**(의존성이 아님)로 검색 결과에 시드됩니다: BYOH 자신의 실행 계층 도구 — [alcove](https://github.com/epicsagas/alcove)(문서 서버), [obsidian-forge](https://github.com/epicsagas/obsidian-forge)(볼트 자동화), [epic-harness](https://github.com/epicsagas/epic-harness)(훅/스킬 런타임) — 가 문맥에 맞춰 노출됩니다("doc server" / "search backend" 쿼리로 alcove 발견) 에이전트가 관련 있을 때 추천할 수 있도록요. 실제로 원할 때만 vendor하세요; 번들은 어느 쪽이든 의존성 없이 배포됩니다.

## 파워 유저: CLI (선택)

위에서 설명한 모든 흐름은 터미널에서도 접근할 수 있습니다. CLI는 **보조 수단**입니다 — 스크립팅, CI, 혹은 대화가 귀찮을 때 유용하지만, 에이전트 주도 경로가 기본 설계입니다.

### 첫 하네스 만들기 — CLI에서

```bash
byoh profile init me --paths ./src ./docs   # 프로젝트 자동 분석
byoh profile confirm me --genre developer   # 장르 확정 (+ 선택 --goal)

byoh render me --target claude              # 또는: codex | agy | all (기본: all); synthesize + static 게이트 내장
byoh install me --scope local               # dist/에 렌더 후 이 프로젝트의 .claude/에만 활성화
byoh install me --scope global              # ...또는 ~/.claude + ~/.codex + ~/.gemini (구 --host)
byoh install me --scope publish             # ...또는 LICENSE + .gitignore 추가 후 git 안내 출력
```

인터뷰 자체는 에이전트 주도입니다(`profile_interview` MCP 도구) — 대화가 곧 인터뷰이므로 별도의 인터랙티브 CLI 인터뷰는 없습니다. 빌드의 static 게이트(MCP 스키마 / 훅 입력 / Critic·Seesaw·Stagnation 게이트 존재)는 항상 실행되어 우회할 수 없습니다 — 덕분에 번들이 배포되기 전 구조적으로 유효함이 보장됩니다. 설치 후 개선은 별도 도구 호출이 아니라 이후 세션의 대화형 회고로 이뤄집니다.

## 내부 동작 원리

BYOH의 합성 엔진은 프로파일 태그를 스킬 레지스트리와 매칭하고, 의존성이 정리된 파이프라인으로 정렬한 뒤, `HarnessBundle` — 모든 지원 호스트의 네이티브 포맷으로 렌더링되는 git-ready 산출물 — 을 만들어냅니다.

- **4-ring 보안 모델** — 라이프사이클 스펙(Ring 0)과 내장 파이프라인 스킬(Ring 1)부터 커뮤니티/미신뢰 스킬(Ring 3)까지, 단계마다 검증 수위가 올라감; 벤더링된 스킬은 sha256 핀이 읽기·임베드 시점에 실검증됨
- **3중 게이트 안전 기준선** — 매 빌드의 static 게이트가 Critic(품질), Seesaw(회귀), Stagnation(정체) 게이트의 존재를 확인, 우회 불가
- **목표 지향 파이프라인** — 30일 목표(제품 출시, 리서치 리포트, 보안 배포 등)를 선언하면 매칭되는 스킬 래더를 자동으로 얹어줌

아키텍처: 헥사고날 — `domain / ports / adapters / application / compiler / evolve / templates / deploy / catalog / mcp / i18n / security / cli`. 전체 가이드는 `AGENTS.md` 참고.

## 전체 CLI 레퍼런스

CLI는 의도적으로 작게 유지합니다: 기계 진입점(`serve`, CI의 `catalog index`, 유지보수자용 `vendor`)과 핵심 빌드 흐름의 스크립트용 미러만 남겼습니다. 인터뷰와 진화는 MCP 전용(에이전트 주도)입니다.

```bash
# 프로파일
byoh profile init <slug> [--paths ...]      # 읽기 전용 프로젝트 스캔
byoh profile confirm <slug> --genre <g> [--goal <텍스트>]  # 프로파일 확정
byoh profile show <slug>                    # 프로파일 YAML 출력

# 빌드 (static 게이트는 항상 실행; render가 합성함: compile + preset 주입)
byoh render <slug> [--target <host>]        # claude | codex | agy | all (기본: all); HarnessBundle 작성
byoh install <slug> [--target <host>] [--scope local|global|publish] [--host] [--force]  # dist/ 트리; --scope가 설치 위치 결정 (local=이 프로젝트, global=HOME, publish=+LICENSE/.gitignore+git 단계). --host는 --scope global의 레거시.

# 커뮤니티 스킬 (유지보수자/빌드타임; sha256 핀이 읽기·임베드 시점에 검증됨)
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# 카탈로그
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<쿼리>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]

# 진단 / 서버
byoh doctor                                 # 실행 계층 도구 점검
byoh serve                                  # stdio MCP 서버 (에이전트 주도 모드)
```

프로파일과 카탈로그 캐시는 기본적으로 `~/.byoh` 아래에 저장됩니다(`BYOH_HOME`으로 재정의).

## 설치

플러그인을 쓰지 않을 때(플러그인이 바이너리를 자동 설치하므로)이거나, 플러그인 미지원 MCP 호스트에 BYOH를 올릴 때만 필요합니다.

### 바이너리 (Rust 툴체인 불필요)

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**소스 빌드:**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # 설치 확인
```

## 빌드 & 개발

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # 단위 + e2e
cvp                               # 병렬 실행: check → clippy → test → fmt → build
```

`mcp` 피쳐(stdio MCP 서버)는 기본으로 켜져 있습니다. BYOH는 내장 문서 검색 색인을 포함하지 않습니다 — 검색 기능이 필요하면 생성된 하네스를 [alcove](https://github.com/epicsagas/alcove) 같은 문서 서버로 연결하세요.

## 감사의 글

BYOH는 여러 커뮤니티 노력 위에 세워졌습니다:

- **플러그인 카탈로그** — [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins)에서 가져옵니다. 상위 100개 Claude 플러그인 레포지토리의, Stars 순으로 정렬된 커뮤니티 목록입니다. 이 목록이 없었다면 카탈로그도 존재할 수 없습니다.
- **동반 도구** — [alcove](https://github.com/epicsagas/alcove)(문서 서버 / RAG), [Episteme](https://github.com/epicsagas/Episteme)(지식 그래프), [obsidian-forge](https://github.com/epicsagas/obsidian-forge)(볼트 자동화)와 함께 동작하도록 설계됐습니다.
- **오픈소스 스택** — [clap](https://docs.rs/clap), [serde](https://serde.rs), [ureq](https://docs.rs/ureq)와 Rust 생태계 위에 빌드됩니다.

카탈로그 항목과 도입한 커뮤니티 스킬은 각자의 라이선스를 따릅니다(도입 시점에 자동 감지). BYOH 자체는 Apache-2.0입니다.

## 라이선스

Apache-2.0.
