# 숨김 (Sumgim) — Meeting Mode

> 회의 시작/종료를 자동 감지해서 **알림과 상태(status)를 한 번에 관리**해주는 Windows 트레이 앱

작성일: 2026-04-23
최근 갱신: 2026-05-08
상태: Draft v0.2 — v0.4 빌드 기준 구현 반영
담당: Minju
앱 이름: 숨김(Sumgim)
  - 유래: "보임(Boim, 회사 문서 자동화 서비스)"으로 자료를 띄우고 → "숨김"으로 방해 요소 차단
과금모델: 오픈소스

---

## 0. Status — 한눈에 보기

| 영역 | 상태 |
|------|------|
| Core MVP (회의 모드 자동 ON/OFF) | ✅ 구현 완료 |
| 디스플레이 복제 / PPT / 전체화면 자동 감지 | ✅ |
| Mattermost / Slack 연동 (DND + 상태 메시지 + 놓친 메시지 카운트) | ✅ |
| KakaoTalk 프로세스 suspend (PRD에 없던 보너스) | ✅ |
| 자동 업데이트 (signed Tauri updater + GitHub Actions) | ✅ |
| 최초 실행 온보딩 마법사 | ✅ |
| 외부 디스플레이 연결 제안 토스트 | ✅ |
| 놓친 알림 인앱 카드 + 메신저 딥링크 | ✅ |
| **캘린더 연동 (Google / Outlook)** | ❌ 미구현 — v0.5+ |
| **앱별 화이트/블랙리스트, 회의 유형별 프로필** | ❌ v1.0+ |
| Beta 공개 (피드백 채널, 랜딩 페이지) | ❌ v0.5 |

---

## 1. Why — 왜 만드는가

### 문제

Windows 기본 방해금지 모드가 있음에도 사용자는 회의 중 알림으로 인한 사고/스트레스를 반복적으로 경험.

### 관찰된 페인포인트 (1차 사용자 인터뷰)

| # | 문제 | 근본 원인 |
|---|------|-----------|
| P1 | 회의 시작 시 방해금지 켜는 걸 **까먹음** → 화면 공유에 카톡 뜸 | 수동 트리거 의존 |
| P2 | 회의 끝나고 **다시 끄는 것도 까먹음** → MM/업무 메시지 놓침 | 자동 해제 신호 없음 |
| P3 | 방해금지 **설정 과정 자체가 귀찮음** (알림 패널 열면 내용 보임) | OS 기본 UX 한계 |
| P4 | 앱마다 중요도가 다른데 **OS는 전부 ON/OFF만 가능** | 세밀한 제어 부재 |

### 기존 해결책의 한계

- **OS 방해금지**: 수동 + 전부 끄기/켜기만 가능
- **온라인 미팅 감지 (줌/팀즈 프로세스)**: 오프라인 미팅에선 무쓸모
- **캘린더 기반 자동화 (맥 Focus)**: 갑자기 잡히는 회의 대응 불가, Windows 옵션 적음

---

## 2. Who — 타겟 사용자

### Primary (MVP)
**오프라인 회의가 잦은 사무직 직장인**
- 주간 보고, 팀 미팅에서 노트북으로 자료 공유
- 카카오톡 / Mattermost / Slack 등 다수 메신저 병행
- Windows 환경, 듀얼/트리플 모니터 가능성

### Secondary
- 온라인 미팅 많은 리모트 워커
- 컨설턴트, 외국계 직장인
- 프레젠테이션/데모가 잦은 세일즈·PM

---

## 3. What — 제품 정의

### 한 줄
> **"회의 시작/종료를 자동 감지해서, OS 알림과 메신저 상태를 한 번에 제어하는 Windows 트레이 앱"**

### 제품 원칙

1. **사용자가 규칙을 정의** — "무엇을 회의로 볼지"는 워크플로우마다 다름. 일괄 자동화 ❌
2. **자동화는 조용하게, 수동은 즉각적으로** — 확실하면 자동, 애매하면 제안, 언제나 단축키로 오버라이드
3. **재진입 비용 0** — 회의 끝나면 놓친 알림을 요약 + 메신저 바로가기
4. **가벼움** — 항상 상주. 메모리 낮고 배터리 부담 적게 (→ Tauri)

---

## 4. Core Features

### 4.1. 회의 감지 트리거

| 트리거 | 감지 방식 | 자동 강도 | 구현 |
|--------|-----------|----------|------|
| 디스플레이 **복제(Clone) 모드** | `QueryDisplayConfig` + `WM_DISPLAYCHANGE`. `active_paths > SM_CMONITORS`로 검출 | 자동 ON | ✅ |
| **PPT 슬라이드쇼** | `EnumWindows` + `screenClass` 윈도우 클래스 | 자동 ON | ✅ |
| **전체화면 앱** | 포어그라운드 윈도우 크기 = 모니터 크기 | 자동 ON (기본 OFF, 오탐 위험) | ✅ |
| **외부 디스플레이 연결** (복제 X) | `SM_CMONITORS` 증가 감지 | 제안 토스트 | ✅ |
| **캘린더 일정** | OAuth + 폴링 | 제안 토스트 | ❌ v0.5+ |

**자동 ON** = 감지 즉시 회의 모드 활성화
**제안 토스트** = "회의 시작? 단축키 누르세요" 안내

### 4.2. 수동 트리거

- ✅ Floating 토글 버튼 (반투명, 드래그 이동, 숨김 옵션)
- ✅ 글로벌 단축키 (기본 `Ctrl+Alt+M`, 커스터마이징)
- ✅ 트레이 아이콘 클릭

### 4.3. 회의 모드 ON 시 동작

| 동작 | 구현 |
|------|------|
| Windows 알림 차단 (시스템 방해금지 + Presentation Mode) | ✅ |
| Mattermost status → DND + 커스텀 상태 메시지 | ✅ |
| Slack `dnd.setSnooze` + `users.setPresence(away)` + 커스텀 상태 | ✅ |
| KakaoTalk 프로세스 일시 정지 (옵션) | ✅ — *PRD엔 없던 보너스. 카톡 API 부재 회피책* |
| 앱별 세부 제어 (화이트/블랙리스트) | ❌ v1.0+ |

### 4.4. 자동 해제 트리거

다음 중 **가장 먼저 발생**하는 신호로 해제:

| 신호 | 구현 |
|------|------|
| 디스플레이 복제 해제 | ✅ |
| PPT 슬라이드쇼 종료 | ✅ |
| 전체화면 앱 종료 | ✅ |
| 캘린더 일정 종료 시간 | ❌ v0.5+ |
| 기본 타임아웃 (60분, 사용자 설정) | ✅ |

**규칙**: 자동 ON된 회의만 자동 OFF. 수동 ON은 사용자가 직접 끔 (의도 보존).

### 4.5. 회의 종료 시 후처리

| 동작 | 구현 |
|------|------|
| Mattermost / Slack 놓친 메시지 수 토스트 | ✅ |
| KakaoTalk 카운트 | ❌ — 데스크톱 API 없음. 대신 회의 중 프로세스 suspend로 *팝업 차단* |
| 인앱 카드에서 [Mattermost 열기] / [Slack 열기] 딥링크 | ✅ — `tauri-plugin-shell` |
| 메신저로 점프 시 읽지 않음 상태 유지 | ✅ — 단순 URL 열기라 자동 보존 |

---

## 5. User Flow

### 최초 설치 (온보딩 3스텝) — ✅ 구현됨

```
설치 → 첫 실행 → main 윈도우 자동 표시 (onboarding_done=false)
  Step 1: 자동 트리거 선택 (복제 / PPT / 전체화면 토글)
  Step 2: Mattermost / Slack 연동 (선택, 스킵 가능)
  Step 3: 단축키 안내 (`Ctrl+Alt+M`)
→ "시작하기" → onboarding_done=true → Settings 화면 → 트레이 상주 시작
```

### 일반 사용 (자동 감지)

```
[회의실 도착]
  → 노트북 HDMI 연결 → 디스플레이 복제 설정
  → [자동 감지] 회의 모드 ON
  → Windows 알림 OFF / MM DND / Slack DND / KakaoTalk suspend
[회의 진행]
  → 알림 없음, 화면 공유 안전
[회의 종료]
  → HDMI 분리 → [자동 해제] 회의 모드 OFF
  → 토스트: "Mattermost 3개, Slack 1개"
  → main 윈도우 자동 표시 → 인앱 카드에서 [Mattermost 열기] [Slack 열기]
```

### 갑작스런 회의

```
[회의 소집] → Ctrl+Alt+M (1회) → 회의 모드 ON
[종료] → Ctrl+Alt+M 또는 60분 타임아웃
```

### 외부 모니터 연결만 (복제 아님)

```
HDMI 연결 → 토스트: "외부 모니터 연결됨. 회의면 Ctrl+Alt+M으로 켜세요"
→ 사용자 판단으로 단축키 (자동 ON 안 함, 외부 모니터 = 회의가 아닐 수도 있음)
```

---

## 6. 다음 우선순위 (v0.5 / v1.0)

### 🔴 v0.5 — Beta 공개 (가장 중요)

- [ ] **사용자 피드백 채널** (Discord 또는 폼) — 출시 후 PRD 갱신의 입력 신호
- [ ] **랜딩 페이지** — 다운로드 + 핵심 기능 30초 데모
- [ ] **캘린더 연동** (Google Calendar 우선, Outlook은 후속)
  - 일정 시작 시 제안 토스트, 종료 시간 도달 시 자동 해제
  - 작업량: OAuth + 토큰 갱신 + 백그라운드 폴링
- [ ] **출시 후 첫 베타 사용자 1명 → 매일 사용 5일/주 검증**

### 🟡 v1.0 — 안정성 + 세부 컨트롤

- [ ] **앱 화이트리스트/블랙리스트** (PRD 6장에서도 "DLL 후킹 재검토" 명시 — 기술 리스크)
  - 카톡: 완전 차단 (현재 suspend로 부분 구현)
  - MM: 알림만 차단, 아이콘 유지
  - Slack: 특정 채널만 차단, DM은 유지
- [ ] **회의 유형별 프로필** ("발표 모드" / "듣기만 하는 회의" / "1:1 미팅")
- [ ] **VIP 예외 규칙** (특정 사람 DM은 항상 받기)

### 🟢 v2.0+ — 확장

- macOS / Linux 지원 (현재 Windows 전용 — `winreg`, Win32 API 의존)
- 팀 캘린더 동기화 (B2B)
- 회의 시간 자동 로깅 + 주간 리포트
- 회의 중 놓친 메시지 AI 요약

---

## 7. Tech Stack

| 영역 | 선택 | 이유 |
|------|------|------|
| Framework | **Tauri 2.x** | Electron 대비 가벼운 메모리(상주형 앱에 필수), Rust 네이티브 API |
| Backend | Rust 1.80+ | Win32 API, 저수준 이벤트, 강한 타입 |
| Frontend | React 18 + TypeScript | 설정 UI / 온보딩 / Floating 토글 |
| Windows API | `windows` 0.61 | `Win32_Devices_Display`, `Win32_UI_WindowsAndMessaging`, 등 |
| 메신저 API | Mattermost REST + Slack Web API | PAT (MM) / User OAuth Token (Slack) |
| 배포 | GitHub Releases + `tauri-plugin-updater` (minisign 서명) | 무료 + 검증된 공급망 |

### 구현 중 검증된 기술 리스크 (PRD 7장 갱신)

- ✅ **Windows DND API**: `HKCU\Software\Microsoft\Windows\CurrentVersion\Notifications\Settings\NOC_GLOBAL_SETTING_TOAST_ENABLED` 레지스트리 + Presentation Mode (`SHQueryUserNotificationState`)로 충분.
- ✅ **디스플레이 복제 검출**: `active_paths > SM_CMONITORS` 가 안정적 시그니처. 3개 모니터 복제 같은 엣지 케이스도 처리.
- ⚠️ **앱별 차단 (DLL 후킹)**: 여전히 미해결. v1.0에서 다시 평가. 임시로 KakaoTalk은 프로세스 suspend (NtSuspendProcess)로 우회.
- ⚠️ **Bizbox 등 폐쇄 그룹웨어**: API 미지원. 캘린더 CalDAV 지원 여부는 회사 환경에 따라 다름.

---

## 8. Roadmap

### v0.1 — Core MVP (✅ 완료)
- ✅ Tauri 세팅 + 트레이 + Floating + 단축키 + 수동 ON/OFF + Win DND

### v0.2 — 자동 감지 (✅ 완료)
- ✅ 디스플레이 복제 감지 + 자동 해제 + 규칙 커스터마이징 UI

### v0.3 — 메신저 + 자동 업데이트 인프라 (✅ 완료)
- ✅ Mattermost / Slack DND + 놓친 알림 토스트 + PPT / 전체화면 감지
- ✅ Tauri 자동 업데이터 + GitHub Actions 릴리스 워크플로

### v0.4 — UX 완성 (✅ 완료, 현재 빌드)
- ✅ 최초 실행 온보딩 마법사
- ✅ 외부 디스플레이 연결 제안 토스트
- ✅ 놓친 알림 인앱 카드 + 메신저 딥링크
- ✅ 자동 업데이트 백그라운드 체크 + 트레이 메뉴 + progress bar

### v0.5 — Beta 공개 (다음)
- [ ] 캘린더 연동 (Google)
- [ ] 사용자 피드백 채널 (Discord or 폼)
- [ ] 랜딩 페이지
- [ ] 베타 모집 + 첫 사용자 5명 매일 사용 검증

### v1.0 — 공식 출시
- [ ] 앱별 세부 컨트롤
- [ ] 안정성 검증 + 재현 가능한 버그 트래킹
- [ ] 사용자 피드백 반영한 PRD v0.3

---

## 9. Success Metrics

### MVP 단계 (v0.1 ~ v0.4) — 본인 도그푸딩
- ✅ **본인 매일 사용**: 주 5일 이상 (현재 모니터링 중)
- ✅ **자동 감지 정확도**: 오탐 < 10% (디스플레이 복제는 정확, PPT 자동 감지는 일부 오탐)
- ✅ **"까먹음" 사건 감소**: 주간 1회 이하 (수동 트리거 의존 시 흔했음 → 자동화로 거의 없음)

### Beta 단계 (v0.5 ~)
- **WAU**: 100명 이상
- **재방문율 (7일)**: 50% 이상
- **NPS**: "OS 방해금지보다 낫다" 응답 비율 > 70%
- **피드백 우선순위 신호**: 캘린더 vs 앱별 차단 vs 회의 유형 프로필 — 어느 게 먼저 요구되는지 데이터로 결정

### v1.0 단계
- 익명 텔레메트리 (사용자 동의 후): 자동 트리거별 ON 빈도, 평균 회의 모드 지속 시간

---

## 10. 출시 메시지 (발표·랜딩 페이지용 핵심 카피)

### Hook (1줄)
> 회의 중 카톡이 화면 공유에 뜨면 끝. **숨김**은 그게 안 뜨게 하는 가장 단순한 방법입니다.

### Why now (왜 지금 만들었나)
- 발표·미팅 사고는 모두가 한두 번씩 경험. OS 방해금지 모드는 *수동 + 전체*라 부족.
- 줌·팀즈 같은 *온라인 회의 자동화*는 많지만 **오프라인 회의 자동화는 비어 있는 시장**.
- Tauri로 가벼운 트레이 앱 + Windows API + 메신저 통합 = 한 명이 만들 수 있는 규모.

### Demo flow (30초)
1. 노트북 HDMI 연결 → 자동으로 회의 모드 켜짐 (트레이 색 변화 + 토스트)
2. Mattermost / Slack 상태가 "회의 중"으로 자동 변경
3. KakaoTalk 일시 정지 (팝업 원천 차단)
4. 회의 종료 → HDMI 분리 → 자동 해제 + 놓친 메시지 카운트 + [Mattermost 열기] [Slack 열기] 버튼

### 차별점
- **오프라인 회의 = 디스플레이 복제 감지**라는 매우 단순하고 정확한 시그널 사용
- **메신저 상태도 같이 바뀜** (단순 OS 방해금지가 아니라 *상대방도 알 수 있는* 알림)
- **회의 끝나면 못 받은 메시지 즉시 복구** (재진입 비용 0)
- **오픈소스 + Windows 전용** = 기업 보안팀 friendly

### v0.5 출시 시점에 답할 질문
- 오프라인 회의가 잦은 직장인이 한 번 깔면 매일 켜놓는가?
- 캘린더 연동 없이도 충분히 가치를 주는가? (아니면 캘린더가 진짜 핵심인가?)
- 카톡 suspend가 진짜 답인가, 아니면 사용자가 다른 방식을 더 원하는가?

---

## 11. Appendix

### 경쟁 제품
- **Windows Focus Assist** — OS 기본, 세밀도 부족, 메신저 상태 동기화 ❌
- **macOS Focus** — 맥 전용, 캘린더 연동 우수
- **Serene (macOS)** — 집중 모드 앱, Windows ❌
- **Microsoft Viva Insights** — 엔터프라이즈, 개인 사용 부담

### 참고 API
- Windows Display Config: `QueryDisplayConfig`, `WM_DISPLAYCHANGE`
- Mattermost API: `PUT /users/{id}/status`, `PUT /users/me/status/custom`
- Slack API: `users.setPresence`, `dnd.setSnooze`, `users.profile.set`
- Tauri docs: https://tauri.app
- 자동 업데이트 minisign: https://jedisct1.github.io/minisign/
