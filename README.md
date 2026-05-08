# 숨김 (Sumgim)

> 회의 시작/종료를 자동 감지해서 Windows 방해금지 모드를 한 번에 관리해주는 트레이 앱

"보임(Boim)"으로 자료를 준비해 띄우고, "숨김(Sumgim)"으로 화면에 뜨면 안 되는 알림과 방해 요소를 차단합니다.

- 상태: v0.4 (PRD MVP 완성: 온보딩 + 자동 감지 + 메신저 연동 + 자동 업데이트)
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

## 자동 업데이트 셋업 (배포자용)

자동 업데이트는 **서명 키 페어**가 있어야 작동합니다. 1회만 셋업하면 됩니다.

### 1. 키 페어 생성

```powershell
npm run tauri signer generate -- -w sumgim.key
```

- `sumgim.key` (개인키) — **저장소에 커밋 금지**, 안전하게 보관
- `sumgim.key.pub` (공개키) — 이 파일 내용을 `src-tauri/tauri.conf.json`의 `plugins.updater.pubkey`에 붙여넣기

### 2. `tauri.conf.json` 채우기

- `plugins.updater.endpoints[0]`의 `OWNER/REPO`를 실제 GitHub 저장소로 교체
- `plugins.updater.pubkey`를 위에서 만든 `sumgim.key.pub` 내용으로 교체

### 3. GitHub Secrets 등록

저장소 **Settings → Secrets and variables → Actions → New repository secret**

| 이름 | 값 |
|------|-----|
| `TAURI_SIGNING_PRIVATE_KEY` | `sumgim.key` 파일 **내용 전체** |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | 키 생성 시 입력한 암호 (없으면 빈 값) |

### 4. 릴리스

```powershell
git tag v0.2.0
git push origin v0.2.0
```

워크플로가 빌드 → 서명 → `latest.json` 포함해 Draft Release 생성. 검토 후 **Publish**하면 기존 사용자의 설정 화면에서 업데이트가 보입니다.

> 키를 잃으면 새 키 페어로 재서명한 빌드를 기존 사용자가 받을 수 없습니다 (서명 검증 실패). 백업 필수.

## v0.3+ 로드맵

- Mattermost / Slack status 연동
- PPT 슬라이드쇼 감지, 전체화면 감지
- 놓친 알림 요약 토스트
- 앱별 화이트/블랙리스트

GitHub: https://github.com/minju-kim98/sumgim.git
