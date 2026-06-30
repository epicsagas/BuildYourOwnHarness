> 이 문서는 [README.md](../../../README.md)의 한국어 번역입니다. 영문 버전이 권위 있는 원본입니다.

**[English](../../../README.md)** | **한국어** | [日本語](../ja/README.md) | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

> **나만의 AI 에이전트** — 범용 템플릿이 아니라, 내 역할·전문 분야·목표에 맞게 컴파일되는 하네스.

대부분의 AI 도구는 고정된 기능 묶음을 주고 "알아서 써보세요"라고 합니다. BYOH는 반대입니다. 짧은 인터뷰로 실제로 어떤 일을 하는지 파악하고, 그에 맞는 에이전트 하네스(스킬·메모리·파이프라인)를 자동으로 생성합니다.

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

## 이런 분께 맞습니다

- **개발자** — 내 스택, 테스트 스타일, 배포 패턴을 이미 아는 에이전트가 필요하다면
- **연구자** — 문헌 검색·인용 추적·합성이 하나로 연결된 파이프라인이 필요하다면
- **크리에이터** — 내 문체와 프로젝트 구조에 맞춰진 글쓰기 파트너가 필요하다면
- **비즈니스 분석가** — 날것의 채팅이 아닌, 의사결정 프레임워크와 보고서 파이프라인이 필요하다면

"AI가 내 맥락을 좀 알아줬으면..."이라는 생각을 해봤다면, BYOH가 바로 그걸 합니다.

## 60초 시작

```
1. byoh profile init me        # 프로젝트 스캔 (읽기 전용, 변경 없음)
2. byoh profile interview me   # 역할과 목표에 대한 짧은 대화
3. byoh compile me             # 개인 하네스 생성
4. byoh install me             # Claude / Codex / agy에 배포
```

다음 세션부터 호스트가 하네스를 자동으로 로드합니다 — 에이전트, 스킬, 메모리, 파이프라인 모두 나에게 맞춰진 상태로.

**이미 원하는 게 있다면** 커뮤니티 카탈로그를 바로 써보세요:
```bash
byoh catalog index                              # 상위 100개 플러그인 목록 다운로드 (수 초)
byoh catalog search "code review"               # 검색
byoh catalog vendor anthropics/claude-code-review   # 하네스에 추가
```

## 설치

### 바이너리 (권장 — Rust 불필요)

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

### AI 호스트에 플러그인 로드

BYOH는 Claude Code, Codex, agy를 모두 지원하는 폴리글랏 플러그인입니다 — 하나의 레포로 세 호스트 전부.

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

플러그인이 처음 로드될 때 `byoh` 바이너리를 자동 설치합니다. Rust가 없어도 됩니다.

> **참고:** 현재 레포는 비공개입니다. 공개 후에는 `epicsagas/plugins` 마켓플레이스에도 등록됩니다.

## 첫 하네스 만들기

### 1단계 — 프로파일

```bash
byoh profile init me --paths ./src ./docs   # 프로젝트 자동 분석
byoh profile interview me                   # 약 5분 대화
byoh profile confirm me --genre developer   # 장르 확정
```

인터뷰는 역할, 전문 분야, 사용 도구, 30일 목표를 물어봅니다. 개발자는 개발자에 맞는 질문을, 연구자는 연구자에 맞는 질문을 받습니다.

### 2단계 — 컴파일 & 설치

```bash
byoh compile me                  # HarnessBundle 생성 (검증 + 게이트 통과)
byoh render me --target claude   # 또는 codex | agy | all
byoh install me                  # dist/에 안전하게 설치
```

### 3단계 — 실행 & 진화

```bash
byoh run me       # 하네스가 활성화된 상태로 실행
byoh evolve me    # 세션 피드백 기반으로 하네스 개선
```

`evolve`는 Critic(품질) / Seesaw(회귀) / Stagnation(정체) 3중 게이트를 반드시 통과해야 반영됩니다. 우회는 불가능합니다.

## 플러그인 카탈로그

Stars 순 상위 100개 Claude 플러그인을 오프라인으로 검색하고 하네스에 바로 추가할 수 있습니다.

```bash
# 최초 1회 인덱싱 (사전 빌드 번들 다운로드 — 수 초)
byoh catalog index

# 인덱싱 후에는 네트워크 없이 검색 가능
byoh catalog search "memory" --genre developer --limit 5

# 하네스에 추가 — license, keywords, genre 자동 추출
byoh catalog vendor obra/superpowers --genre developer
```

MCP 도구(`catalog_search` / `catalog_vendor`)를 통해 LLM 에이전트가 검색 → 추가 흐름을 완전 자율로 처리할 수 있고, CLI로 직접 지정하는 것도 물론 가능합니다.

## 에이전트 주도 모드

`byoh serve`를 실행하면 stdio MCP 서버가 시작됩니다. AI 호스트가 14개 도구를 직접 호출해서 인터뷰, 위자드, 실행을 모두 대화로 처리합니다 — CLI는 보조 수단이 됩니다.

```bash
byoh serve   # Claude / Codex / agy가 연결해서 모든 걸 주도
```

사용 가능한 도구: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `evolve_cycle`, `rag_index`, `rag_search`, `genre_list`, `registry_clone_skill`, `catalog_search`, `catalog_vendor` 등.

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

# 지식베이스 (RAG)
byoh index <slug> [--corpus <dir>] [--force]
byoh search <slug> "<쿼리>" [--genre <g>] [--k N]
```

## 내부 동작 원리

합성 엔진이 프로파일 태그를 스킬 레지스트리와 매칭해 의존성 순서가 잡힌 파이프라인을 만들고, 각 호스트의 네이티브 포맷으로 렌더링되는 `HarnessBundle`을 생성합니다.

- **4-ring 보안 모델** — 내장 스킬(Ring 1)부터 커뮤니티/미신뢰 스킬(Ring 4)까지 단계별로 검증 수위가 높아짐
- **3중 게이트 진화** — Critic(품질), Seesaw(회귀), Stagnation(정체) 세 게이트를 모두 통과해야 반영, 우회 불가
- **영속 RAG** — 변경된 문서만 재임베딩(`+추가 ~변경 -삭제`); 이후 검색은 저장된 인덱스 그대로 재사용
- **목표 지향 파이프라인** — 30일 목표(제품 출시, 리서치 리포트, 보안 배포 등) 선언 시 매칭되는 스킬 래더를 자동으로 얹어줌

아키텍처: 헥사고날 — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. 전체 가이드는 `AGENTS.md` 참고.

## 빌드 & 개발

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # 단위 + e2e
cvp                               # 병렬 실행: check → clippy → test → fmt → build
```

선택 피쳐: `--features mcp`(MCP 서버), `--features native-rag`(로컬 임베딩), `--features rag-openai`(OpenAI 임베딩). 릴리즈 바이너리는 전체 피쳐를 포함합니다.

## 라이선스

Apache-2.0.
