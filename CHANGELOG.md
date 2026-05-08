# Changelog

이 프로젝트의 모든 주목할 만한 변경사항이 이 파일에 기록됩니다.
형식은 [Keep a Changelog](https://keepachangelog.com/ko/1.1.0/)를 따르고,
버저닝은 [SemVer](https://semver.org/lang/ko/)를 따릅니다.

## [0.4.0] — 2026-05-08

### Added — PRD MVP 완성
- **최초 실행 온보딩 마법사** (3스텝): 자동 트리거 선택 → MM/Slack 연동 (스킵 가능) → 단축키 안내. `onboarding_done` 플래그로 한 번만 표시 (PRD 5.1)
- **외부 디스플레이 연결 제안 토스트**: 복제는 아니지만 모니터 수가 늘어났을 때 회의 모드 단축키 안내 토스트 (PRD 4.1)
- **놓친 알림 인앱 카드 + [열기] 딥링크**: 회의 종료 시 main window 자동 표시, Mattermost(저장된 서버 URL) / Slack(`slack://` scheme + 웹 fallback)으로 바로 점프 (`tauri-plugin-shell`)

### Changed — 자동 업데이트 UX
- 앱 시작 30초 후 + 24시간마다 백그라운드 업데이트 체크. 새 버전 발견 시 토스트 알림
- 트레이 메뉴 "업데이트 확인" 항목 추가 (Settings 창 자동 열고 체크 트리거)
- Settings 다운로드 진행률에 시각적 progress bar 추가

## [0.3.0] — 2026-05-08

### Added
- **자동 업데이트 인프라**: GitHub Releases에서 서명된 빌드를 다운로드하고 재시작 시 적용 (`tauri-plugin-updater`, minisign 서명 검증)
- **GitHub Actions 릴리스 워크플로**: 태그 푸시 (`v*`) 또는 수동 트리거 시 Windows MSI/EXE를 자동 빌드하고 Draft Release로 업로드. `latest.json`도 함께 첨부되어 자동 업데이트 엔드포인트 역할
- Settings 화면의 "업데이트 확인" / "다운로드 및 설치" UI

### Changed
- 빌드 산출물에 minisign 서명 (`.sig`) 동시 생성
- README에 자동 업데이트 셋업 가이드 (키 페어 생성, GitHub Secrets 등록 절차)

### Notes
- 기존 v0.2.x 설치본은 updater 플러그인이 없어 자동 업데이트되지 않습니다. 한 번만 수동으로 v0.3.0 이상을 받아 설치하면 이후부터는 자동으로 갱신됩니다.

## [0.2.0] — Initial release

### Added
- 트레이 상주 (좌클릭 토글, 우클릭 메뉴)
- 글로벌 단축키 (기본 `Ctrl+Alt+M`, 커스터마이징 가능)
- Floating 토글 버튼 (반투명, 드래그 이동)
- Windows 방해금지 (ToastEnabled) + Presentation Mode 자동 제어
- 디스플레이 복제(Clone) 모드 자동 감지 → 회의 모드 자동 ON/OFF
- PowerPoint 슬라이드쇼 / 전체화면 자동 감지 (옵션)
- KakaoTalk 일시 정지 (옵션)
- Mattermost / Slack 연동 (DND, 커스텀 상태 메시지, 놓친 알림 카운트)
- 기본 60분 타임아웃 후 자동 해제
- Windows 시작 프로그램 등록
- 설정 영속화 (`tauri-plugin-store`) + 크래시 복구

[0.4.0]: https://github.com/minju-kim98/sumgim/releases/tag/v0.4.0
[0.3.0]: https://github.com/minju-kim98/sumgim/releases/tag/v0.3.0
[0.2.0]: https://github.com/minju-kim98/sumgim/releases/tag/v0.2.0
