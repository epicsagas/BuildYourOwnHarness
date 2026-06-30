> 이 문서는 [README.md](../../../README.md)의 한국어 번역입니다. 영문 버전이 권위 있는 원본입니다.

<div align="center">

**[English](../../../README.md)** | **한국어** | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### 나만의 AI 에이전트

*범용 템플릿이 아니라, 내 역할·전문 분야·목표에 맞게 컴파일되는 하네스.*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

대부분의 AI 도구는 고정된 기능 묶음을 주고 "알아서 써보세요"라고 합니다. BYOH는 반대입니다. 짧은 인터뷰로 실제로 어떤 일을 하는지 파악하고, 그에 맞는 에이전트 하네스(스킬·메모리·파이프라인)를 자동으로 생성합니다. 바로 꺼내 쓸 수 있도록 내 워크플로에 맞춰진 상태로요.

## 이런 분께 맞습니다

- **개발자** — 내 스택, 테스트 스타일, 배포 패턴을 이미 아는 에이전트가 필요하다면
- **연구자** — 문헌 검색·인용 추적·합성이 하나로 연결된 파이프라인이 필요하다면
- **크리에이터** — 내 문체와 프로젝트 구조에 맞춰진 글쓰기 파트너가 필요하다면
- **비즈니스 분석가** — 날것의 채팅이 아닌, 의사결정 프레임워크와 보고서 파이프라인이 필요하다면

"AI가 내 맥락을 좀 알아줬으면..."이라는 생각을 해봤다면, BYOH가 바로 그걸 합니다.

## 60초 시작

BYOH는 AI 에이전트가 주도하도록 설계됐습니다. 설치하고, MCP로 호스트를 연결한 뒤, 그냥 대화하세요 — 대화가 곧 인터뷰이자 위자드이자 빌드입니다.

```
1. Install byoh              # 원라인 설치 (아래 참고)
2. Connect your host via MCP # byoh serve — 모든 MCP 호환 에이전트
3. "Build me a harness"      # 에이전트가 레포를 스캔해서 결과를 컴파일
```

다음 세션부터 호스트가 하네스를 자동으로 로드합니다 — 에이전트, 스킬, 메모리, 파이프라인 모두 나에게 맞춰진 상태로.

**터미널을 선호하시나요?** CLI에서 같은 흐름을 실행할 수 있습니다:
```
byoh profile init me        # 프로젝트 스캔 (읽기 전용, 변경 없음)
byoh profile interview me   # 역할과 목표에 대한 짧은 대화
byoh compile me             # 개인 하네스 생성
byoh install me             # Claude / Codex / agy에 배포
```

**이미 원하는 게 정해졌다면** 커뮤니티 카탈로그를 둘러보세요:
```bash
byoh catalog index                                 # 상위 100개 플러그인 목록 다운로드 (수 초)
byoh catalog search "code review"                  # 관련 플러그인 검색
byoh catalog vendor anthropics/claude-code-review  # 하네스에 추가
```

## 설치

### 바이너리 (권장 — Rust 툴체인 불필요)

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

### AI 호스트에 연결

BYOH는 MCP를 사용하므로, 모든 MCP 호환 에이전트가 BYOH를 주도할 수 있습니다. 위 바이너리를 설치하고 서버를 시작하면, 호스트가 모든 BYOH 도구를 직접 호출합니다:

```bash
byoh serve   # stdio MCP 서버
```

**다른 에이전트**(Cursor, Zed, Continue, …)는 호스트의 MCP 설정에 `byoh`를 추가하세요:
```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

**Claude Code, Codex, agy**를 쓰시나요? 플러그인으로 설치하세요 — MCP 서버를 함께 묶어서 처음 로드할 때 바이너리까지 자동 설치합니다 (Rust 불필요):

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

> **참고:** 현재 레포는 비공개입니다. 위 경로를 사용하세요. 공개 후에는 `epicsagas/plugins` 마켓플레이스에도 등록됩니다.

## 에이전트 주도 모드

호스트가 연결되면 명령어를 치지 않아도 됩니다 — 그냥 대화하세요. 에이전트가 BYOH의 14개 도구를 직접 호출하고, 대화가 곧 인터뷰이자 빌드이자 진화 사이클입니다:

사용 가능한 도구: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `evolve_cycle`, `genre_list`, `registry_clone_skill`, `catalog_search`, `catalog_vendor` 등.

## 첫 하네스 만들기 — CLI에서

터미널에서 같은 단계를 실행합니다:

### 1단계 — 프로파일
```bash
byoh profile init me --paths ./src ./docs   # 프로젝트 자동 분석
byoh profile interview me                   # 약 5분 대화
byoh profile confirm me --genre developer   # 장르 확정
```

인터뷰는 역할, 전문 분야, 사용 도구, 30일 목표를 물어봅니다. 개발자는 개발자에 맞는 질문을, 연구자는 연구자에 맞는 질문을 받습니다.

### 2단계 — 컴파일 & 설치
```bash
byoh compile me          # HarnessBundle 생성 (검증 + 게이트 통과)
byoh render me --target claude   # 또는 codex | agy | all
byoh install me          # dist/에 안전하게 설치
```

### 3단계 — 실행 & 진화
```bash
byoh run me              # 하네스가 활성화된 상태로 실행
byoh evolve me           # 세션 피드백 기반으로 하네스 개선
```

`evolve`는 Critic(품질) / Seesaw(회귀) / Stagnation(정체) 3중 게이트를 반드시 통과해야 반영됩니다. 우회는 불가능합니다.

## 플러그인 카탈로그

카탈로그는 상위 100개 Claude 플러그인을(Stars 순, 매일 갱신) 큐레이션해서, 터미널을 떠나지 않고도 커뮤니티 스킬을 발견하고 추가할 수 있게 해줍니다.

```bash
# 최초 1회 인덱싱 (사전 빌드 번들 다운로드 — 수 초)
byoh catalog index

# 인덱싱 후에는 네트워크 없이 검색 가능
byoh catalog search "memory" --genre developer --limit 5

# 하네스에 추가
# license, keywords, genre는 클론한 레포에서 자동 추출
byoh catalog vendor obra/superpowers --genre developer
```

LLM 에이전트가(`catalog_search` / `catalog_vendor` MCP 도구로) 이 흐름 전체를 자율으로 처리할 수도 있고, CLI에서 직접 실행할 수도 있습니다.

## 전체 CLI 레퍼런스

```bash
# 프로파일
byoh profile init <slug> [--paths ...]      # 읽기 전용 프로젝트 스캔
byoh profile interview <slug>               # 인터뷰
byoh profile confirm <slug> --genre <g>     # 프로파일 확정

# 빌드
byoh compile <slug> [--dry-run]             # 검증 + HarnessBundle 생성
byoh render <slug> --target <host>          # claude | codex | agy | all
byoh install <slug> [--host <dir>]          # dist/ 또는 실제 플러그인 디렉토리에 배포

# 실행 & 진화
byoh run <slug>
byoh evolve <slug>

# 커뮤니티 스킬
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# 카탈로그
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<쿼리>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## 내부 동작 원리

합성 엔진이 프로파일 태그를 스킬 레지스트리와 매칭해 의존성 순서가 잡힌 파이프라인을 만들고, 지원되는 모든 호스트의 네이티브 포맷으로 렌더링되는 `HarnessBundle`을 생성합니다.

- **4-ring 보안 모델** — 내장 스킬(Ring 1)부터 커뮤니티/미신뢰 스킬(Ring 4)까지 단계별로 검증 수위가 높아짐
- **3중 게이트 진화** — 매 `evolve` 사이클이 Critic(품질), Seesaw(회귀), Stagnation(정체) 게이트를 모두 통과해야 반영, 우회 불가
- **목표 지향 파이프라인** — 30일 목표(제품 출시, 리서치 리포트, 보안 배포 등) 선언 시 매칭되는 스킬 래더를 자동으로 얹어줌

아키텍처: 헥사고날 — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. 전체 가이드는 `AGENTS.md` 참고.

## 빌드 & 개발

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # 단위 + e2e
cvp                               # 병렬 실행: check → clippy → test → fmt → build
```

`mcp` 피쳐(stdio MCP 서버)는 기본으로 켜져 있습니다. BYOH는 내장 문서 검색 색인을 포함하지 않습니다 — 검색 기능이 필요하면 생성된 하네스를 [alcove](https://github.com/epicsagas/alcove) 같은 문서 서버로 연결하세요.

## 라이선스

Apache-2.0.
