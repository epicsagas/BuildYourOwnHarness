# RFC: 커뮤니티 스킬 페치/캐시

- **상태:** Draft (설계 제안, 미구현)
- **관련:** `docs/ROADMAP_AGENT_LED.md` C-group "커뮤니티 스킬 페치/캐시"
- **작성:** 2026-06-25

## 1. 배경

BYOH는 스킬/에이전트 본문을 컴파일 타임 `include_str!`로 바이너리에 임베드한다
(`deploy/presets.rs`, `deploy/agent_presets.rs`). 이는 **오프라인·재현성·감사 가능성**을
보장하지만, 외부(커뮤니티) 스킬 통합은 현재 out-of-scope다. 본 RFC는 커뮤니티 스킬을
BYOH로 가져오는 설계를 탐색한다.

## 2. 목표 / 비목표

**목표**
- 외부 스킬 소스(awesomeclaudeplugins 등, 직접 git URL)에서 스킬을 가져온다.
- BYOH의 오프라인 원칙을 위반하지 않는다(런타임 네트워크 호출 없음).
- 외부(신뢰할 수 없는) 코드의 보안 위험을 통제한다.

**비목표**
- 런타임 자동 업데이트.
- BYOH가 호스팅하는 중앙 레지스트리.
- 페치된 스킬의 자동 승격(사람이 검토 후 벤더).

## 3. 현재 아키텍처 (제약)

- `registry/presets/<genre>/<id>.md`, `registry/agents/<genre>/<id>.md` — 컴파일 타임 임베드.
- spec **§Out**: "no remote registry" — **런타임 원격 조회 없음**.
- `inject_preset` / `inject_agent` — id 기반 dedupe(augment/clone).

## 4. 설계 옵션

### 옵션 A — 빌드 타임 벤더링 (권장)

외부 스킬을 **빌드 전**에 로컬 `registry/vendored/`로 가져와, 기존 `include_str!` 경로에
합류시킨다. 런타임 네트워크 호출은 없다.

- 진입점: `byoh vendor add <source> [--genre <g>] [--as <id>]`, `byoh vendor list`, `byoh vendor remove <id>`.
- 소스 유형: (1) awesomeclaudeplugins 카탈로그 항목, (2) 직접 git URL, (3) 로컬 경로.
- 저장: `registry/vendored/<genre>/<id>.md` + `registry/vendored/MANIFEST.toml`(소스 URL·체크섬·라이선스·페치 시각).
- 빌드: `raw_preset`/`raw_agent_preset` 매치 분기에 vendored 항목 추가(또는 빌드 스크립트가 `include_str!` 대상을 생성).

**장점** — 오프라인 원칙 준수, 재현성(MANIFEST 커밋), 감사(벤더된 파일 리뷰 가능).
**단점** — 최신성은 수동(`byoh vendor update`).

### 옵션 B — 런타임 페치 + 캐시 (기각)

런타임에 외부 스페치 페치 + 로컬 캐시. **오프라인 원칙 위반 + 네트워크 의존 + 보안 노출면
확대**로 기각.

## 5. 보안 (핵심 — 구현 선행 조건)

외부 스킬은 **신뢰할 수 없는 코드**다(임의 명령/프롬프트 주입 가능).

1. **Ring 격리** — 벤더된 스킬은 기본 **Ring 3**(가장 제한적)에 배치. 호스트 도구는 최소.
2. **정적 검증** — 본문 내 위험 패턴 플래그: 네트워크(`curl`/`wget`), 파괴적 명령(`rm -rf`),
   시크릿 경로, 난독화. `byoh vendor add` 시 경고/차단.
3. **체크섬/서명** — MANIFEST에 SHA-256 기록. 변경 시 diff 알림.
4. **허용 목록** — 기본적으로 신뢰된 소스(awesomeclaudeplugins 공식)만; 임의 git URL은 `--trust` 명시.
5. **라이선스 기록** — MANIFEST에 라이선스 필드; 호환 불일치 시 경고.

## 6. 오프라인 원칙 충돌 해소

spec §Out "no remote registry"는 **"런타임 원격 조회 없음"**으로 해석한다. 옵션 A의
빌드 타임 벤더링은 런타임 네트워크가 없으므로 원칙을 위반하지 않는다. 단, 벤더 행위 자체는
개발자 머신에서 발생하며, 그 결과(`registry/vendored/` + MANIFEST)가 커밋되어 재현성을 유지한다.

## 7. 통합 지점

- `deploy/presets.rs` / `deploy/agent_presets.rs`: vendored 항목을 catalog에 추가.
- CLI: 새 `vendor` 서브커맨드.
- `inject_*`: 변경 없음(동일 id-dedupe).
- 검증: vendored 스킬도 `static_gate` 통과 대상(본문 형식).

## 8. 제안 마일스톤

- **M1** — `vendor add/list/remove` + `registry/vendored/` + MANIFEST (옵션 A 최소).
- **M2** — 보안 검증(정적 분석 + 체크섬 + 허용목록).
- **M3** — awesomeclaueplugins 카탈로그 스키마 매핑(소스 구조 조사 후).

## 9. 구현 전 조사 항목

- **awesomeclaudeplugins** repo의 실제 구조/스키마(marketplace manifest? 단일 스킬 단위?).
- 라이선스 분포(MIT/Apache/비허가).
- Claude Code/agi 플러그인 포맷과 BYOH 스킬 포맷(SKILL.md 4섹션)의 매핑.

## 10. 결론

**옵션 A(빌드 타임 벤더링)**으로 오프라인 원칙을 유지하면서 커뮤니티 스킬을 통합한다.
보안(Ring 3 격리 · 정적 검증 · 체크섬 · 허용목록)이 구현의 선행 조건이며, M1 직전에
awesomeclaudeplugins 소스 구조 조사가 필요하다. 런타임 페치(옵션 B)는 기각.
