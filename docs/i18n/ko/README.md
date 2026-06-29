> 이 문서는 [README.md](../../README.md)의 한국어 번역입니다. 영문 버전이 권위 있는 원본이며, 더 최신일 수 있습니다.

# BuildYourOwnHarness (BYOH)

> 사용자의 암묵지·데이터·비즈니스 장르·목표를 인터랙티브하게 취합하여, **사용자만의 맞춤형 AI 에이전트 하네스를 생성·배포·운영·진화**합니다.

BYOH는 [epiccounty](https://github.com/epicsagas) 워크스페이스의 검증된 빌딩 블록 위에 **생성 계층**을 추가합니다. 고정된 스킬/메모리/파이프라인 세트를 배포하는 대신, 인터뷰를 통해 사용자별로 *고유한* 하네스를 컴파일합니다.

## 하는 일

확정된 사용자 프로파일(장르 + 전문 분야 + 30일 목표)이 합성 엔진을 구동하여, 레지스트리 스킬을 **키워드로 재조립**해 순서화된 파이프라인을 만들고, 고정 장르 템플릿이 아닌 `HarnessBundle`을 생성합니다. 전체 파이프라인은 폐루프이며, 우회할 수 없는 세 개의 안전장치(Critic / Seesaw / Stagnation)가 감쌉니다.

```
profile (interview → genre → confirm) → compile → synthesize → render → install → run → evolve
```

## 빌드 및 검증

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                       # 단위 + e2e
./target/release/byoh --help
```

헥사고날 아키텍처: `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`.

## CLI

```bash
byoh profile init <slug> [--paths ...]   # S1 자동분석 (비파괴)
byoh profile interview <slug>            # S2 인터뷰 (Suggest + Council)
byoh profile confirm <slug> --genre <g>  # S3 위자드 확정
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>
byoh compile <slug> [--dry-run]          # 정적 게이트 + dry-run 게이트 → HarnessBundle
byoh render <slug> --target claude       # claude | codex | agy | all (git-ready)
byoh install <slug>                      # 안전한 dist/ 설치 (--host 로 실제 플러그인 디렉토리)
byoh run <slug>
byoh evolve <slug>                       # 3중 게이트 진화 사이클
```

`--language` 옵션(기본 `auto`)은 `LC_ALL`/`LANG`에서 언어를 자동 감지합니다.

### 에이전트 주도 모드 (MCP 서버)

`byoh serve`(`--features mcp`)가 stdio MCP 서버를 띄워 **LLM 에이전트가 BYOH를 주도**합니다 — CLI는 보조로 전환됩니다(제어권 역전). 12개 도구(`profile_*`, `rag_*`, `genre_list`, `compile`, `evolve_cycle`, `registry_clone_skill`)를 `tools/list`로 발견해 호출합니다. 대화 자체가 인터뷰/위자드입니다.

```bash
cargo build --release --features mcp
byoh serve
```

## 핵심: 합성 + 벤더링

- **합성 엔진** — `synthesize(profile)`이 레지스트리 스킬을 프로필 태그로 매칭·순서화하고, 3중 게이트 재통과를 강제합니다(우회 불가). 목표 지향 파이프라인(product-launch / decision / research-report / secure-ship / …)이 30일 목표가 매칭되면 스킬 사다리 + 에이전트 세트를 overlay 합니다.
- **커뮤니티 스킬 벤더링** (RFC M3) — `byoh vendor add`가 외부 `SKILL.md`(로컬 경로 또는 git URL)를 가져와 정적 검증 + sha256을 거치고, `build.rs`가 빌드 타임에 **Ring 3**(최제한 링)으로 임베드합니다. 외부 스킬은 신뢰할 수 없는 코드로 합성에 참여합니다.

## 상태

생성 계층의 Rust 구현: 프로파일러 + 인터뷰 + 장르 템플릿 + 컴파일러(4-ring, MCP 도구 자동생성, 정적 게이트) + 진화 엔진 + 자체 RAG(선택 `native-rag`) + MCP 서버(선택 `mcp`). 아키텍처 가이드는 `AGENTS.md`를 참고하세요.

RAG 계층은 **영속 지식베이스**다. `byoh index`가 장르 인덱스 + corpus 사이드카를 `$BYOH_HOME/indexes/`에 저장하고, 이후 `byoh search`(또는 `rag_search` MCP 도구)를 `--corpus` 없이 호출하면 `load_index`로 재사용한다 — 재임베딩 없음. 재인덱싱은 **증분**으로 동작한다 — 콘텐츠 해시 매니페스트가 추가/변경된 문서만 재임베딩하고 삭제된 문서는 제거한다(`+a ~c -r`로 보고). `--force`는 전체 재빌드.

## 라이선스

Apache-2.0.
