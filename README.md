<div align="center">

<img src="src/assets/logo.png" alt="Sumgim logo" width="120" />

# 숨김 (Sumgim)

**회의 시작/종료를 자동 감지해서 Windows 알림 + 메신저 상태를 한 번에 관리해주는 트레이 앱**

> _"보임은 자료를 보이고, 숨김은 알림을 숨깁니다."_

[![Release](https://img.shields.io/github/v/release/minju-kim98/sumgim?include_prereleases&style=flat-square&color=6366f1)](https://github.com/minju-kim98/sumgim/releases)
[![Downloads](https://img.shields.io/github/downloads/minju-kim98/sumgim/total?style=flat-square&color=22c55e)](https://github.com/minju-kim98/sumgim/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-yellow.svg?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D6?style=flat-square&logo=windows&logoColor=white)](#)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app)
[![CI](https://img.shields.io/github/actions/workflow/status/minju-kim98/sumgim/release.yml?style=flat-square&label=build)](https://github.com/minju-kim98/sumgim/actions)

[다운로드](#-다운로드) · [핵심 기능](#-핵심-기능) · [개발](#%EF%B8%8F-개발-환경) · [로드맵](#-로드맵) · [PRD](docs/meeting-mode-PRD.md)

</div>

---

## 🎯 왜 만들었나

> 회의 중 화면 공유에 카톡이 떠서 발표 망친 적, 한 번씩 있으시죠?

Windows 기본 방해금지 모드가 있는데도 흔히 일어나는 4가지:

- 🔴 회의 시작할 때 → 방해금지 켜는 걸 **까먹는다**
- 🟡 회의 끝났는데 → **다시 끄는 것도 까먹는다** (메시지 놓침)
- 🟠 알림 패널 열면 → **내용이 보인다** (회의 중에 못 엶)
- 🔵 OS 방해금지는 → **전부 ON / 전부 OFF만 가능** (앱별 제어 ❌)

**숨김**은 이 4가지를 다 해결하는 가장 단순한 도구입니다.

## ✨ 무엇을 해주나

- 🖥️ **자동 감지** — 노트북에 외부 모니터를 복제 모드로 연결하면 회의 모드 자동 ON
- 🔕 **OS 알림 차단** — Windows 방해금지 + Presentation Mode 자동 활성
- 💬 **메신저 상태 동기화** — Mattermost / Slack 자동으로 DND + "회의 중" 커스텀 상태
- 🚫 **카카오톡 일시 정지** — 데스크톱 카톡은 API가 없어 프로세스 자체를 멈춰 팝업 원천 차단
- 📨 **회의 후 0초 복구** — 놓친 메시지 수 토스트 + `[Mattermost 열기]` `[Slack 열기]` 한 번에 점프
- 🪟 **상주형 트레이 앱** — 메모리 ~30MB, Tauri 기반의 가벼움
- 🔄 **자동 업데이트** — GitHub Releases에서 서명된 빌드를 백그라운드로 받아 재시작 시 적용

## 📦 다운로드

[**최신 릴리스 (Latest release) →**](https://github.com/minju-kim98/sumgim/releases/latest)

| 형식 | 설명 |
|------|------|
| `.msi` | 깔끔한 설치 (권장, 기업 환경) |
| `.exe` | NSIS 설치 (일반 사용자) |

> ⚠️ 코드 서명이 없는 빌드라 SmartScreen 경고가 뜰 수 있습니다. **추가 정보 → 실행**으로 진행하세요.

## ⚡ 빠른 시작

```
1. 다운로드 → 설치 → 첫 실행
2. 3단계 온보딩 마법사
   ├─ 자동 트리거 선택 (디스플레이 복제 / PPT / 전체화면)
   ├─ Mattermost·Slack 연동 (선택, 스킵 가능)
   └─ 단축키 안내 (Ctrl+Alt+M)
3. 트레이 상주 → 자동 감지 또는 단축키로 토글
```

## 🚀 핵심 기능

### 회의 자동 감지

| 트리거 | 강도 | 비고 |
|--------|------|------|
| 디스플레이 복제 (Clone) 모드 | 🟢 자동 ON | 가장 정확한 시그널 |
| PowerPoint 슬라이드쇼 | 🟢 자동 ON | `EnumWindows` + 윈도우 클래스 감지 |
| 전체화면 앱 | 🟡 자동 ON (기본 OFF) | 영상/게임에서 오탐 위험 |
| 외부 모니터 연결 (복제 X) | 🔵 제안 토스트 | 사용자가 단축키로 결정 |
| 캘린더 일정 | 🔜 v0.5 | OAuth + 폴링 기반 |

### 회의 모드 ON 시 동작

```
┌─ Windows ─────────────────────────────────────┐
│  ✓ 토스트 알림 차단 (HKCU\...\NOC_GLOBAL...)   │
│  ✓ Presentation Mode 활성                       │
└─────────────────────────────────────────────────┘
┌─ Messengers ──────────────────────────────────┐
│  Mattermost  → DND + 커스텀 상태 (PAT)          │
│  Slack       → dnd.setSnooze + presence away    │
│  KakaoTalk   → 프로세스 일시 정지 (옵션)         │
└─────────────────────────────────────────────────┘
```

### 회의 종료 시

- 📊 토스트로 `Mattermost: 3개 · Slack: 1개` 카운트
- 📲 메인 윈도우 자동 표시 + 인앱 카드에 `[Mattermost 열기]` `[Slack 열기]` 딥링크
- ⏰ 60분 자동 해제 타임아웃 (사용자 설정)

### 자동 해제 트리거

가장 먼저 발생하는 신호로 회의 모드 OFF — 디스플레이 복제 해제 / PPT 종료 / 전체화면 종료 / 캘린더 일정 종료 (v0.5+) / 타임아웃.

> 💡 **자동 ON된 회의만 자동 OFF.** 수동 ON은 사용자가 직접 끔 (의도 보존).

## 📸 스크린샷

> _GIF / 스크린샷은 v0.5 베타 공개와 함께 추가 예정_

| 트레이 | 온보딩 | 놓친 알림 카드 |
|:---:|:---:|:---:|
| _coming soon_ | _coming soon_ | _coming soon_ |

## 🛠️ 개발 환경

### 사전 요구사항

- **Node** 22+, **npm** 10+
- **Rust** 1.80+ (`rustup`)
- **Windows** 10/11 (1809 이상 권장)

### 빌드

```powershell
npm install
npm run tauri dev      # 개발 모드 (HMR)
npm run tauri build    # 릴리스 빌드 → src-tauri/target/release/bundle/{msi,nsis}/
```

### 디렉토리 구조

```
src/                    React + TypeScript 프론트엔드
├─ App.tsx              윈도우 라벨 + onboarding_done 기반 라우팅
├─ pages/
│  ├─ Onboarding.tsx    3단계 첫 실행 마법사
│  ├─ Settings.tsx      설정 / 업데이트 확인 / 놓친 알림 카드
│  └─ FloatingToggle.tsx 항상 위 토글 버튼
└─ lib/api.ts           Tauri invoke / event 헬퍼

src-tauri/              Rust 백엔드
├─ src/
│  ├─ lib.rs            앱 부팅 + setup + 백그라운드 스레드들
│  ├─ commands.rs       Tauri command (settings, meeting, messenger 테스트)
│  ├─ state.rs          AppSettings + MeetingState
│  ├─ dnd.rs            Win32 방해금지 모드 제어
│  ├─ display_monitor.rs 디스플레이 복제 / 외부 모니터 감지
│  ├─ trigger_monitor.rs PPT 슬라이드쇼 / 전체화면 감지
│  ├─ messenger.rs      Mattermost / Slack API
│  ├─ kakao_suspend.rs  KakaoTalk 프로세스 suspend
│  ├─ tray.rs           트레이 아이콘 + 메뉴
│  ├─ shortcut.rs       글로벌 단축키
│  └─ window_hider.rs   상시 윈도우 가시성 제어
└─ capabilities/        Tauri permission allowlist

.github/workflows/      GitHub Actions
└─ release.yml          태그 푸시 시 빌드 + 서명 + Draft Release
```

### 자동 업데이트 셋업 (배포자용)

자동 업데이트는 **minisign 키 페어**가 필요합니다. 1회만 셋업.

```powershell
# 1. 키 페어 생성 (안전한 곳에 보관)
npm run tauri signer generate -- -w sumgim.key

# 2. tauri.conf.json
#    plugins.updater.endpoints[0] → GitHub repo URL
#    plugins.updater.pubkey       → sumgim.key.pub 내용

# 3. GitHub Secrets 등록
#    TAURI_SIGNING_PRIVATE_KEY          = sumgim.key 파일 내용
#    TAURI_SIGNING_PRIVATE_KEY_PASSWORD = 키 생성 시 입력한 암호

# 4. 릴리스
git tag v0.x.0
git push origin v0.x.0   # → Actions 자동 빌드 → Draft Release → 검토 후 Publish
```

> ⚠️ 키 분실 = 기존 사용자 자동 업데이트 영구 차단. 비밀번호 매니저 백업 필수.

## 📋 로드맵

| 버전 | 상태 | 핵심 |
|:---:|:---:|---|
| v0.1~v0.2 | ✅ | Core MVP + 디스플레이 복제 자동 감지 |
| v0.3 | ✅ | 자동 업데이트 + GitHub Actions 릴리스 |
| **v0.4** | ✅ **현재** | 온보딩 / 외부 디스플레이 제안 / 놓친 알림 딥링크 |
| v0.5 | 🔜 | **Beta 공개** — 캘린더 연동, 피드백 채널, 랜딩 페이지 |
| v1.0 | 🔜 | 앱별 화이트/블랙리스트, 회의 유형별 프로필, VIP 예외 |
| v2.0+ | 🔜 | macOS / Linux 지원, 팀 캘린더 동기화, AI 요약 |

상세 기획은 [PRD](docs/meeting-mode-PRD.md), 변경 이력은 [CHANGELOG](CHANGELOG.md), 발표용 자료는 [presentation.md](docs/presentation.md) 참고.

## 🤝 기여하기

베타 사용자 / 피드백 / 버그 리포트 모두 환영입니다.

- 🐞 [Issues](https://github.com/minju-kim98/sumgim/issues)
- 💬 [Discussions](https://github.com/minju-kim98/sumgim/discussions)
- 🔧 PR 자유롭게

## 🧰 Tech Stack

<table>
<tr>
<td>

**Frontend**
- React 18 + TypeScript
- Vite

</td>
<td>

**Backend**
- Rust + `windows` crate (Win32)
- [Tauri 2.x](https://tauri.app)

</td>
<td>

**APIs**
- Mattermost REST
- Slack Web API

</td>
<td>

**Distribution**
- GitHub Releases
- minisign signed updates

</td>
</tr>
</table>

## 🌗 Sumgim과 Boim — 한 짝의 도구

이 프로젝트는 우리 팀의 다른 서비스 **보임 (Boim)** 의 짝입니다.

|  | 보임 (Boim) | 숨김 (Sumgim) |
|---|---|---|
| **무엇을** | 자료를 보임 | 알림을 숨김 |
| **언제** | 발표를 위해 자료를 띄울 때 | 자료를 띄울 때 옆의 사적인 알림을 가릴 때 |
| **형태** | 문서 자동화 서비스 | Windows 트레이 앱 |

> _"보임은 자료를 보이고, 숨김은 알림을 숨깁니다."_

화면 공유 = 보여줄 것과 숨길 것이 동시에 생기는 순간. **두 도구가 한 짝으로 동작합니다.**

## 📄 License

[MIT](LICENSE) © 2026 Minju Kim

---

<div align="center">
  <sub>Built with ❤️ in Seoul · Powered by <a href="https://tauri.app">Tauri</a></sub>
</div>
