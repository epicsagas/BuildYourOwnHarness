# BYOH Roadmap — Agent-Led Harness Generation

> 작성일: 2026-06-25 · 최종 갱신: 2026-06-25 (PR #3 구현 완료)
> 제어권 역전(Inversion of Control): **CLI(byoh)가 주도 → LLM 에이전트가 주도**
> 상태: PR #1 · PR #2 · **PR #3(에이전트 주도) 구현 완료 — 병합 대기**

---

## 1. 핵심 방향 전환

```
현재(완료):  CLI(byoh)가 주도 ──호출──> LLM(인터뷰 답변 생성 등 보조 도구)
목표(PR #3): LLM 에이전트가 주도 ──호출──> byoh(검색/컴파일/검증 도구 = MCP tools)
```

이것은 UI 문제가 아니라 **제어권의 역전**이다. BYOH가 LLM에게 무엇을 하라고 지시하는 게 아니라, LLM이 BYOH의 능력을 MCP 도구로 가져다 쓰는 구조가 되어야 한다. "CLI는 보조, LLM 에이전트가 주도로 생성"이 최종 목표.

## 2. 현재 구현 상태 (PR #1 + #2)

| 부품 | 상태 | LLM 주도에서의 역할 |
|------|------|---------------------|
| `byoh compile` (4-Ring 번들 생성) | ✅ PR #1 | LLM이 호출하는 "번들 생성" 도구 |
| `byoh search` / 자체 RAG (llm-kernel) | ✅ PR #2 | LLM이 "사용자 자료 검색" 도구 |
| `byoh profile` (init/interview/confirm) | ✅ PR #1 | LLM이 "프로파일 읽기/쓰기" 도구 |
| 진화 엔진 (3중 안전장치) | ✅ PR #1 | LLM이 "진화 사이클 실행" 도구 |
| **MCP 서버 노출 (`byoh serve`)** | ❌ **부재** | **빈 자리** — LLM이 도구를 발견·호출하는 통로 |
| **플러그인 매니페스트** (`.claude-plugin/`, `.codex/`) | ❌ 부재 | Claude Code/Codex가 진입 감지 → BYOH MCP 도구로 연결 |
| **복제 레지스트리 실구현** | ❌ 추상적 | LLM이 "검증된 스킬을 사용자 장르에 복제" 도구 |
| 인터랙티브 UX | ❌ CLI 텍스트 | LLM 대화 자체가 인터뷰/위자드 경로 (별도 UI 불필요) |

## 3. 엔드유저 플로우 (완성 목표)

```mermaid
flowchart LR
    U([엔드유저가 BYOH 플러그인 설치<br/>Claude Code / Codex])
    U -->|하네스 만들어줘| P[플러그인 매니페스트<br/>감지 → BYOH MCP 도구 진입]
    P --> A[LLM 에이전트 주도]
    A --> S1[1. 자료 수집<br/>byoh profile + search 도구]
    S1 --> S2[2. 재분류·인덱싱<br/>rag index 도구]
    S2 --> S3[3. 인터랙티브<br/>대화 = 인터뷰/위자드]
    S3 --> S4[4. 종합 분석<br/>ConfirmedProfile]
    S4 --> S5[5. 하네스 생성<br/>compile + 복제 도구]
    S5 --> S6[6. dry-run 검증 + 설치]
    S6 --> H([커스터마이즈드 하네스<br/>+ 자가 진화])

    style A fill:#fff3cd,stroke:#b8860b,stroke-width:2px
```

**핵심**: 인터랙티브 경로는 LLM 주도로 가면 자동 해결 — LLM 에이전트와의 대화가 곧 인터뷰/위자드이므로 CLI 텍스트나 별도 UI(Svelte)를 만들 필요가 없다.

## 4. PR 구조 (스택)

```
main
 └── PR #1  feat/byoh-full-impl      (생성 계층 — 컴파일러/진화/프로파일)  ← 병합 대기
      └── PR #2  feat/byoh-native-rag (자체 RAG — llm-kernel 직접 의존)   ← 병합 대기
           └── PR #3  feat/byoh-agent-led (에이전트 주도 — MCP 서버 + 플러그인 + 복제)  ← 병합 대기
                └── PR #4  orbit-synthesis-engine (합성 엔진 — 레지스트리 재조립)  ← 병합 대기
```

- PR #1 머지 → PR #2 자동 main 리베이스 → … → PR #4 자동 main 리베이스
- 각 PR은 독립된 아키텍처 결정 단위. 합치지 않는다.
- **PR #4(합성 엔진)**: PR #3의 "에이전트 주도" 위에 "레지스트리 스킬 재조립 → 고유 하네스" 비전의 첫 구현. Council 오르빗으로 생성. 상세는 PR #4 본문.

## 5. PR #3 상세 — `feat/byoh-agent-led`

base = `feat/byoh-native-rag` (PR #2). 세 가지를 하나의 일관된 스토리로 묶는다.

### 5.1 MCP 서버 (`byoh serve`) — 제어권 역전의 핵심
- BYOH의 능력을 MCP 도구로 노출: `profile.read`, `profile.write`, `profile.scan`, `rag.index`, `rag.search`, `compile`, `compile.dry_run`, `evolve.cycle`, `registry.clone_skill`
- LLM이 `tools/list`로 발견하고 주도적으로 호출
- stdio MCP 서버 (Claude Code/Codex 표준)
- **이것이 빈 자리**: 현재는 CLI만 있고 LLM이 발견할 MCP 진입점이 없다

### 5.2 플러그인 매니페스트 — 진입 감지
- `.claude-plugin/plugin.json` + `skills/byoh-build-harness/SKILL.md` (korean-law-rag 패턴 차용)
- `.codex/` 설정
- 사용자가 "내 하네스 만들어줘" / "build my harness" 감지 → BYOH MCP 도구로 진입 안내

### 5.3 복제 레지스트리 실구현 — "복제 또는 생성"
- 컴파일러가 장르 템플릿 **생성(Generate)**뿐 아니라, 검증된 스킬(epic-harness tdd/debug, 커뮤니티 awesome-claude-plugins)을 사용자 장르에 맞게 **복제(Clone)** 주입
- `registry/presets/` → 장르별 스킬 복제 소스
- LLM이 "이 검증된 스킬을 복제해 줘" 도구로 호출

### 5.4 인터랙티브 경로는 별도 구축 불필요
- LLM 에이전트와의 대화 = 인터뷰(S2) + 위자드(S3)
- 기존 `RuleInterview` / `StaticWizard` 로직이 LLM 프롬프트 컨텍스트로 재사용됨

## 6. 의사결정 기록 (왜 이 구조인가)

1. **PR #1 + #2를 합치지 않는 이유**: 서로 다른 아키텍처 결정(외부 의존 vs 자체 RAG). 분리 시 리뷰/유지보수/롤백 단위 명확.
2. **PR #3을 #2 위에 쌓는 이유**: 세 기능(MCP/플러그인/복제)이 동일 목표(LLM 주도)에 묶임 — 일관된 하나의 PR.
3. **LLM 주도 전환의 근거**: 사용자의 명시적 요구("CLI는 보조, LLM 에이전트가 주도로 생성"). BYOH의 부품은 이미 MCP 도구로 노출될 준비가 됨(순수 함수 + file 기반 상태).
4. **인터랙티브 UI를 만들지 않는 이유**: LLM 대화가 곧 인터랙티브 경로. 별도 TUI/Web UI는 도메인 밖(스펙 §Out).

## 7. 검증 기준 (PR #3 AC 초안)

- AC1: `byoh serve`가 stdio MCP 서버를 띄우고 `tools/list`에 BYOH 도구들이 보인다
- AC2: Claude Code가 `.claude-plugin/` 매니페스트를 인식하고 BYOH 진입을 제안한다
- AC3: LLM이 MCP 도구만으로 (CLI 직접 호출 없이) profile → rag.index → rag.search → compile 플로우를 주도한다
- AC4: 복제 도구가 외부 검증 스킬을 장르 번들에 주입한다 (생성과 복제가 공존)
- AC5: default 빌드에서 fmt + clippy -D warnings + test green 유지
- AC6: `native-rag` feature에서도 컴파일 + test green 유지

## 7.1 구현 결과 (PR #3 — 2026-06-25 완료)

AC 1–6 전부 달성. 커밋 `b5d2352` → 후속(spawn_blocking/프리셋 확충/문서) 추가 후 push.

| AC | 결과 | 근거 |
|----|------|------|
| AC1 | ✅ | `byoh serve`(rmcp 1.8 stdio)가 **12개 도구** 노출 — stdio 스모크 `tools/list` 확인 |
| AC2 | ✅ | `.claude-plugin/plugin.json` + `skills/build-harness/SKILL.md` + `.mcp.json` + `.codex/config.toml` |
| AC3 | ✅ | MCP 도구만으로 profile_create→confirm→rag_index→rag_search→compile→dry_run→clone 주도 (`tests/mcp_tools.rs`) |
| AC4 | ✅ | `registry_clone_skill` + `registry/presets/` (4장르 7프리셋), id 기반 중복 제거(증강/클론) |
| AC5 | ✅ | default 빌드: fmt + clippy -D warnings + 115 단위/14 e2e green |
| AC6 | ✅ | `native-rag,mcp` 빌드 green; embedder 팩토리 cfg 보존 |

**구현된 12개 MCP 도구**: `profile_read` · `profile_create` · `profile_scan` · `profile_interview` ·
`profile_confirm` · `rag_index` · `rag_search` · `genre_list` · `compile` · `compile_dry_run` ·
`evolve_cycle` · `registry_clone_skill`

**구조 결정 (구현 후 확정)**:
- 무거운 I/O 도구(`profile_scan`/`rag_index`/`rag_search`/`compile_dry_run`)는 `async fn` +
  `tokio::task::spawn_blocking`로 전환 — tokio 런타임 블록 방지. 동기 본체는 별도 `*_sync`/`*_blocking` 헬퍼로 분리.
- 도메인 타입은 `Serialize`만 있으므로 MCP 응답은 opaque `serde_json::Value` (도메인 derive 변경 없음).
- 프리셋은 `include_str!` 컴파일 타임 임베드 — 네트워크/git clone 제외 (spec §Out).
- `mcp` 피처는 opt-in — 기본 빌드는 비동기 런타임 없이 가벼움 유지.

**검증 매트릭스**: default / `--features mcp` / `--features native-rag,mcp` 전부 build + test green.

## 7.2 병합 현황 + 잔여 항목 (2026-06-25 최종 갱신)

### ✅ 완료 — main 병합됨
PR #1~#5 전부 `main`에 merge-commit 병합 완료. 스택 브랜치(원격+로컬) + worktree 정리 완료. `main`만 잔존.

| PR | 내용 | 상태 |
|----|------|------|
| #1 | 생성 계층 (M0–M5: 컴파일러/진화/프로파일) | ✅ merged |
| #2 | 자체 RAG (llm-kernel 직접 의존) | ✅ merged |
| #3 | MCP 서버 + 플러그인 매니페스트 + 복제 | ✅ merged |
| #4 | 합성 엔진 (레지스트리 스킬 재조립) | ✅ merged |
| #5 | 타겟 렌더러 (Claude/Codex/agy 배포 플러그인) | ✅ merged |

사용자 경험 사슬: `요청 → 인터뷰/RAG → 합성 → 타겟별 플러그인 렌더 → git push로 공개` 까지 완성.

### ✅ 코어 루프 완료 (2026-06-25, orbit `remaining-items`)

A의 4개 CLI 스텁이 전부 안전하게 실구현됨 — **생성→설치→진화 루프가 닫힘**.

| 항목 | 결과 |
|------|------|
| `install` | ✅ `deploy/install.rs` — render→staging→원자적 rename. 기본 프로젝트-로컬 `dist/`, HOME은 `--host` 옵트인. `.byoh-manifest` 마커로 비-BYOH 디렉토리 보호(`--force` 필요). 슬러그 새니타이즈 |
| `evolve` | ✅ `evolve/state.rs` + `application/evolve_run.rs` — seesaw/stagnation 상태 영속(cycle 간), 정직한 `EvolutionDecision` 출력, Rejected/RolledBack는 **비-0 exit**(기존 "항상 approved" 거짓 수정). malformed 상태는 백업+거부 |
| `run` | ✅ thin honest — 설치된 플러그인/매니페스트 경로 보고 |
| `hook` | ✅ 알려진 훅 디스패치, unknown은 에러(비-0 exit) |
| 공유 validator | ✅ `store::sanitize_slug` (install/evolve/run 공유) |
| MCP 노출 | ✅ `install_plugin` 도구 신규 + `evolve_cycle`이 영속 상태 사용 |

검증: default/mcp/native-rag,mcp 빌드 green, clippy -D warnings + fmt clean, 153 단위 + 14 e2e + 6 mcp 테스트, E2E(install→dist HOME 무영향, evolve cycle_n 영속, RolledBack→exit 1) 확인.

### ✅ Issue #6 완료 (2026-06-25) — agent 프리셋 카탈로그 + synthesis 주입

스킬 프리셋 패턴(`deploy/presets.rs`)을 **에이전트**로 미러링. 합성 엔진이 이제 스킬뿐 아니라 **에이전트**도 프로파일 키워드로 재조립한다 — 장르 기본 에이전트 위에 매칭된 에이전트를 augment/clone.

| 항목 | 결과 |
|------|------|
| 에이전트 프리셋 마크다운 | ✅ `registry/agents/<genre>/<id>.md` — 7개(developer 3 / creator 2 / researcher 1 / business 1), `include_str!` 컴파일 타임 임베드, SKILL 4섹션 스타일 |
| `agent_catalog()` + `AgentPresetMeta` | ✅ 키워드 태그 카탈로그(`deploy/agent_presets.rs`) |
| `inject_agent()` | ✅ id 기반 dedupe — 기존 에이전트는 augment(본문/이름/설명 교체), 미존재는 clone. 멱등 |
| synthesis 통합 | ✅ `select_agents()` + `synthesize()` 4b 단계에서 매칭 에이전트 주입, `synthesis_base_agent_count` 기록(재현성) |
| 안전 | ✅ 합성 후 `static_gate` 재실행 유지 — 3중 안전장치 우회 불가 |

부수 수정: `tests/mcp_tools.rs` 가 프로세스 전역 env(`BYOH_HOME`/`BYOH_DIST_DIR`)를 병렬 조작해 간헐 실패하던 숨은 결함을 `serial_test` 직렬화로 정정(main에서 타이밍 운으로 통과했던 것).

검증: default 165 단위 + 14 e2e + (mcp) 6 mcp_tools 전부 green, fmt clean, clippy -D warnings clean(default + mcp), native-rag,mcp 빌드 green.

### ⏳ 남은 후속 (설계상 연기)

**B. 등록된 이슈**
- ~~[Issue #6](https://github.com/epicsagas/BuildYourOwnHarness/issues/6): agent 프리셋 카탈로그 + synthesis agent 주입 (스킬 프리셋 패턴 미러)~~ ✅ 완료 (아래 "Issue #6 완료" 섹션 참조)

**C. 합성/렌더러 후속**
- ~~커뮤니티 스킬 페치/캐시~~ ✅ M1/M1b 완료 (PR #15/#16, RFC #14) — `byoh vendor add` + sha256 MANIFEST + 정적 검증 + 합성 통합 + Ring 3 격리. 남은: `build.rs` 배포 임베드, 소스 허용목록
- ~~장르 enum 일반화~~ ✅ 완료 (PR #12 — `GenreProfile` 테이블)
- ~~파이프라인 라이브러리~~ ✅ 완료 (PR #17 도메인 파이프라인 + 목표 파이프라인 6종: product-launch/market-analysis/decision/research-report/content-create/secure-ship)
- ~~스킬 카탈로그 확장~~ ✅ 완료 — 7→21 (epiccounty 범용 14 시딩: developer 9 + business 5)
- ~~정식 DAG 순환 감지~~ ✅ 완료 (PR #9 — 3색 DFS, 순서 무관; forward dependency는 유효)
- ~~agy 포맷 실검증~~ ✅ 완료 (PR #11 — BYOH→agy install/load 증명; commands 디렉토리만 사소 skip)

**권장 다음 단계**: 코어 루프 + Issue #6(에이전트 재조립, PR #8) + DAG(#9) + 커뮤니티 스킬 M1/M1b(#15/#16) + 파이프라인(#17, 도메인+목표 6종) + 스킬 카탈로그 7→21 + Ring 3(#18) + agy 검증(#11)까지 완료. 남은 정리: 스킬 본문 범용화(epic 워크플로 잔존), creator/researcher 풀 보강, vendored `build.rs` 배포 임베드, 소스 허용목록/`--trust`.

## 8. 참조

- 기획서: `docs/00_RESEARCH_REPORT.md` ~ `docs/04_ROADMAP.md`
- 분석(이 방향 전환의 근거): `docs/ANALYSIS_AGENT_LED.md`
- korean-law-rag 플러그인 패턴(차용원): `.claude-plugin/`, `.codex/` 구조
