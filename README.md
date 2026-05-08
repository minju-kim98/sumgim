# 숨김 (Sumgim)

> 회의 시작/종료를 자동 감지해서 Windows 방해금지 모드를 한 번에 관리해주는 트레이 앱

"보임(Boim)"으로 자료를 준비해 띄우고, "숨김(Sumgim)"으로 화면에 뜨면 안 되는 알림과 방해 요소를 차단합니다.

- 상태: v0.2 (MVP + 디스플레이 복제 자동 감지)
- 플랫폼: Windows 10/11 (1809 이상 권장)
- 라이선스: 오픈소스

## 핵심 기능

- 트레이 상주, 좌클릭 토글 / 우클릭 메뉴
- 글로벌 단축키 (기본 `Ctrl+Alt+M`, 커스터마이징 가능)
- Floating 토글 버튼 (선택, 반투명, 드래그 이동)
- Windows 방해금지(ToastEnabled) + Presentation Mode 자동 제어
- 디스플레이 복제(Clone) 모드 자동 감지 → 회의 모드 자동 ON/OFF
- 기본 60분 타임아웃 후 자동 해제
- 설정 영속화 (`tauri-plugin-store`) + 크래시 복구

## 개발 환경

- Node 22+, npm 10+
- Rust 1.80+, Cargo
- Windows 11 권장

## 개발 명령어

```bash
npm install
npm run tauri dev      # 개발 모드 실행
npm run tauri build    # 릴리스 빌드 (MSI/EXE 생성)
```

## 디렉토리 구조

```
src/             React + TypeScript 프론트엔드 (설정 화면, Floating 토글)
src-tauri/       Rust 백엔드 (Win32 DND, 디스플레이 모니터, 트레이, 단축키)
```

## v0.3+ 로드맵

- Mattermost / Slack status 연동
- PPT 슬라이드쇼 감지, 전체화면 감지
- 놓친 알림 요약 토스트
- 앱별 화이트/블랙리스트

GitHub: https://github.com/minju-kim98/sumgim.git
