# BYOH 아키텍처 분석 — Agent-Led 전환 근거

> 작성일: 2026-06-25
> 목적: PR #1 + #2 완료 후 "다음 단계" 의사결정의 분석 근거를 기록.
> 짝 파일: `docs/ROADMAP_AGENT_LED.md` (실행 계획)

---

## 1. 원래 질문과 답

### Q1. 엔드유저가 플러그인 설치 후 하네스 구축을 요청하면, 자료 수집 → 재분류/인덱싱 → 인터랙티브 → 종합 분석 → 훅/스킬/에이전트 복제 또는 생성 → 커스터마이즈드 하네스 구축인가?

**A. 맞다.** 사용자가 그린 6단계 플로우는 BYOH의 정확한 설계 목표(ARCH §3.3 시퀀스 + §5 컴파일)이다. PR #1 + #2가 이 6단계 전부를 동작 가능한 Rust 코드로 구현해 둔 상태.

| 단계 | 구현 상태 |
|------|-----------|
| 1. 자료 수집 (`profile init --paths`) | ✅ `FilesystemSource` 비파괴 스캔 |
| 2. 재분류·인덱싱 (`rag index`) | ✅ PR #2, 장르별 벡터 인덱스 + BM25 |
| 3. 인터랙티브 (인터뷰+위자드) | ✅ Suggest-don't-move + Council + 자기-설명 옵션 |
| 4. 종합 분석 (ConfirmedProfile) | ✅ 4상태 머신 `confirmed` 전이 |
| 5. 하네스 생성 (compile) | ✅ 4-Ring 훅/스킬/MCP 도구 렌더링 |
| 6. dry-run + 설치 | ✅ 샌드박스 검증 + 부트스트래퍼 |

**"완벽한"을 실현하려면 3개 갭(플러그인 래퍼 / 복제 경로 / 인터랙티브 경로)이 남아 있었다** — 그리고 이 갭들은 LLM 주도 전환으로 대부분 자연스럽게 해결된다(§3).

### Q2. PR #1 + #2를 합칠 수 있는가?

**A. 합치면 안 된다.** 기술적으로 스쿼시 머지 가능하지만, 두 PR은 서로 다른 아키텍처 결정이다:
- PR #1 = 생성 계층(외부 도구 호출에 의존)
- PR #2 = 자체 RAG(llm-kernel 직접, 그 의존 제거)

분리 시: (a) 리뷰어가 각 결정을 독립 평가, (b) RAG 방식 갈아엎어도 생성 계층 보존, (c) PR #1 머지 → PR #2 자동 main 리베이스. **이미 그 구조로 되어 있다.**

### Q3. 다음 작업(플러그인/복제/인터랙티브)은 #2에 쌓는가, 별도인가?

**A. #2 위에 새 브랜치(PR #3)로 쌓는다.** 세 가지가 동일 목표(LLM 주도 하네스 생성 경험 전달)에 묶여 있어 하나의 일관된 PR이 낫다.

### Q4. CLI가 보조이고 LLM 에이전트가 주도라는 것의 의미?

**A. 제어권 역전(Inversion of Control).** 현재는 반대로 되어 있다:

```
현재:  CLI(byoh)가 주도 ──> LLM(보조)
목표:  LLM 에이전트가 주도 ──> byoh(MCP 도구 = 보조)
```

이것은 단순히 UI 문제가 아니라 아키텍처 전환이다. 좋은 소식: **이 역전을 위한 부품이 이미 다 구현되어 있다.** 빈 자리는 단 하나, **MCP 서버 노출**뿐이다(§3 표 참조).

## 2. 갭 분석 (3가지, 현재 → LLM 주도 전환 시)

| 갭 | 현재 상태 | LLM 주도에서의 해결 |
|----|-----------|---------------------|
| **플러그인 래퍼 부재** | `byoh` CLI만 있음. Claude Code/Codex가 BYOH 플로우로 자동 안내하는 매니페스트 없음 | PR #3에서 `.claude-plugin/`, `.codex/` 추가 (korean-law-rag 패턴 차용) |
| **복제 경로 부재** | 컴파일러가 장르 템플릿에서 **생성(Generate)**만 함. 검증된 스킬 **복제(Clone)** 주입은 추상적 | PR #3에서 `registry/presets/` + 복제 도구 실구현. 생성과 복제가 공존 |
| **인터랙티브가 CLI 텍스트** | 인터뷰/위자드가 CLI stdout 기반 → 비기술 사용자에게 거침 | **LLM 주도 전환으로 자동 해결** — LLM 대화 = 인터뷰/위자드. 별도 UI 불필요 |

**핵심 통찰**: 인터랙티브 갭은 별도 구축이 필요 없다. LLM 에이전트와의 대화 자체가 인터랙티브 경로이기 때문. 이것이 "CLI는 보조"라는 방향성과 정확히 일치한다.

## 3. 부품 매핑 — 현재 구현이 LLM 주도에서 어떻게 재사용되는가

BYOH의 모든 핵심 능력이 이미 **순수 함수 + file 기반 상태**로 구현되어 있어, MCP 도구로 노출하기가 매우 쉽다(헥사고날 아키텍처의 배당이 이 전환을 위해 의도된 것):

| BYOH 부품 | 현재 인터페이스 | MCP 도구로 노출 시 |
|-----------|-----------------|---------------------|
| `application::ProfileOrchestrator` | Rust API | `profile.scan`, `profile.interview_next`, `profile.confirm` |
| `rag::build_index` / `rag::hybrid_search` | Rust API | `rag.index`, `rag.search` |
| `compiler::compile_profile` + `static_gate` + `dry_run` | Rust API | `compile`, `compile.dry_run`, `compile.validate` |
| `evolve::run_cycle` | Rust API | `evolve.cycle` |
| `templates::TemplateLibrary` | Rust API | `genre.list`, `genre.skills` |
| `deploy::registry` | Rust API | `registry.clone_skill` (PR #3 신규) |

**이 표가 보여주는 것**: LLM 주도 전환은 **새 기능을 만드는 게 아니라, 이미 있는 Rust API에 MCP 래퍼를 씌우는 것**이다. 빈 자리는 MCP 서버 자체뿐.

## 4. korean-law-rag 차용 근거

korean-law-rag은 같은 워크스페이스에서 이미 **REST 서버 + Claude Code/Codex 플러그인 + 공유 AGENTS.md** 패턴을 검증했다. BYOH는 이 패턴에서:
- `.claude-plugin/plugin.json` + `skills/` 구조
- `.codex/` 설정
- stdio MCP 노출 방식 (REST 대신 — 에이전트 직접 연동이 목적이므로)

를 차용한다. 단 BYOH는 REST 서버가 아니라 **stdio MCP 서버**로 간다 — LLM 에이전트가 직접 도구를 호출하는 게 목표이기 때문.

## 5. 결론

PR #1(생성 계층) + PR #2(자체 RAG)로 **골격이 완전히 동작**한다. 남은 것은 **PR #3 하나**로:
1. MCP 서버 노출(`byoh serve`) — 빈 자리 채우기
2. 플러그인 매니페스트 — 진입 감지
3. 복제 레지스트리 실구현 — "복제 또는 생성"의 복제 경로

이 세 가지가 끝나면 엔드유저 시나리오(플러그인 설치 → LLM 주도 하네스 생성 → 완성)가 end-to-end로 닫힌다. 인터랙티브 UI는 LLM 대화로 대체되므로 별도 구축 불필요.

실행 계획은 `docs/ROADMAP_AGENT_LED.md`.
