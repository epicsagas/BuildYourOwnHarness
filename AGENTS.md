# AGENTS.md — BuildYourOwnHarness (BYOH)

> AI 에이전트(Claude Code 등)가 이 코드베이스에서 작업할 때 읽어야 할 아키텍처 가이드.
> 기획 근거는 `docs/00..04`, 이 파일은 구현 관점의 운용 지침.

## 1. 역할

BYOH는 **생성 계층(generation layer)** 이다. 사용자 프로파일(`truth`/`candidates`/`derived`)을
취합하고, 장르 템플릿과 결합해 **실행 가능한 HarnessBundle(4-Ring)** 을 컴파일하며,
설치 후 3중 안전장치(Critic/Seesaw/Stagnation) 하에 진화시킨다.

**실행 계층은 외부 도구에 위임** — obsidian-forge(수집), alcove(RAG), Episteme(지식그래프),
epic-harness(실행·진화 원형), claudy(런처)는 별도 설치된 프로세스로 `CommandPort` 뒤에서 호출한다.
BYOH가 이들을 재구현하지 않는다(스펙 §Out).

## 2. 모듈 지도 (헥사고날)

```
src/
├── domain/          순수 타입 (IO 없음)
│   ├── profile      UserProfile 스키마 + 4상태 머신
│   ├── bundle       HarnessBundle, Ring, McpTool, HookSpec
│   ├── genre        Genre, SafetyGate, GenreTemplate
│   ├── evidence     ObservationRecord, AbMetric
│   └── state        BuildState, 45분 크래시 임계치
├── ports/           trait 경계 (LlmPort, ProfileSource, InterviewPort, WizardPort, CommandPort)
├── adapters/        구현체 (RuleLlm=오프라인 결정론, FilesystemSource, RuleInterview, StaticWizard, StdCommand)
├── application/     ProfileOrchestrator — S1/S2/S3 순차 실행
├── compiler/        render(4-Ring) · validate(정적 게이트) · dryrun · incremental(3a/3b/3c)
├── evolve/          gates(Critic/Seesaw/Stagnation) · lifecycle · recall(B11) · compress(B13) · skills(SkillOpt)
├── templates/       base + 4 자식(developer/creator/researcher/business) + 상속 머지
├── deploy/          registry · bootstrap(install.sh/ps1/cargo-binstall) · provider(B14) · state(B9)
├── i18n/            B17 ko/en 카탈로그
├── obs/             관측 로그 + 상태 facade
├── security/        시크릿 마스킹
└── cli.rs / main.rs clap 트리 + 디스패치
```

## 3. 핵심 불변량 (절대 위반 금지)

1. **3중 안전장치 강제** — `safety_gates`에 critic/seesaw/stagnation 세 개가 **모두** 있어야 컴파일·진화가 진행된다(`SafetyGate::validate_all_present`, `SafetyGateSet::validate_all`). 하나라도 빠지면 거부.
2. **진실/파생 분리(B6)** — 자동 추출값은 항상 `derived`, 사용자 확정값만 `truth`. 역유도 불변량.
3. **비파괴 취합(B1)** — 자동분석은 자료를 읽기만 하고 이동/수정하지 않는다.
4. **`#![forbid(unsafe_code)]`** — unsafe 금지.

## 4. 작업 시 규칙

- 새 기능은 **port 뒤에** 둔다 — 외부 LLM/검색 엔진은 trait으로 추상화하고, 테스트는 결정론적
  rule-based 어댑터를 쓴다(네트워크 없이 `cargo test`가 통과해야 한다).
- 커밋은 Conventional Commits. `--no-verify` 금지.
- 시크릿/PII는 코드·로그·테스트 데이터에 절대 평문으로 넣지 않는다. `security::mask` 활용.
- 검증: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`가 모두 green이어야 한다.

## 5. 테스트

- 단위 테스트: 각 모듈 내 `#[cfg(test)]` (84개).
- 통합 테스트: `tests/end_to_end.rs` — 4 장르 전체 M0 경로 + 게이트/재컴파일/프로바이더/복구/마스킹 (14개).

## 6. 의존성

Rust edition 2021, rust-version 1.82. 주요 크레이트: `clap`(derive), `serde`/`serde_yaml`/`serde_json`/`toml`,
`anyhow`/`thiserror`, `sha2`, `regex`, `walkdir`, `chrono`, `tracing`. dev: `tempfile`, `pretty_assertions`.
