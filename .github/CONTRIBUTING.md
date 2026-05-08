# 기여 가이드 (Contributing)

숨김(Sumgim)에 관심 가져주셔서 감사합니다 🙏
어떤 형태로든 기여를 환영합니다.

## 환영하는 기여

- 🐞 **버그 리포트** — [Bug Report 이슈 템플릿](https://github.com/minju-kim98/sumgim/issues/new?template=bug_report.yml)
- ✨ **기능 제안** — [Feature Request 이슈 템플릿](https://github.com/minju-kim98/sumgim/issues/new?template=feature_request.yml)
- 💬 **사용 후기 / 자유 토론** — [Discussions](https://github.com/minju-kim98/sumgim/discussions)
- 🔧 **코드 PR** — 작은 오타 수정부터 큰 기능까지 모두 환영
- 📖 **문서 개선** — README / PRD / 코드 주석

## 사전 셋업

```powershell
# 사전 요구사항
# - Node 22+, npm 10+
# - Rust 1.80+ (rustup)
# - Windows 10/11 (1809 이상)

git clone https://github.com/minju-kim98/sumgim.git
cd sumgim
npm install
npm run tauri dev   # 개발 모드 실행
```

자세한 디렉토리 구조는 [README의 디렉토리 구조 섹션](../README.md#디렉토리-구조) 참고.

## PR 흐름

1. **이슈를 먼저 열어주세요** (큰 변경이면 더더욱)
   - 작은 오타/문서 수정은 곧바로 PR도 OK
2. 브랜치 이름은 자유 (예: `fix/mattermost-status-restore`, `feat/calendar-integration`)
3. 변경사항이 동작하는지 로컬에서 검증
   - `npm run build` (frontend tsc + vite)
   - `npm run tauri dev` 또는 `npm run tauri build`
4. PR 작성 시 [PR 템플릿](PULL_REQUEST_TEMPLATE.md)을 따라 작성
5. 리뷰 후 머지

## 커밋 메시지 컨벤션

엄격하지는 않지만, 기존 커밋과 비슷한 결을 유지해주세요:

```
짧은 한 줄 요약 (영어 또는 한국어, 명령형 어조)

본문: 왜 이 변경이 필요한지, 어떻게 구현했는지.
변경 *내용*보다 *동기*를 적어주는 게 미래의 우리에게 더 도움이 됩니다.
```

좋은 예:
- `Add Tauri auto-updater and GitHub Actions release workflow`
- `Fix Mattermost custom status restore when meeting ends`
- `Bump version to 0.4.0`

## 코드 스타일

- **TypeScript**: `npm run build`의 `tsc` 단계가 통과해야 합니다.
- **Rust**: `cargo fmt` + `cargo clippy` (CI에서 강제하진 않지만 PR 전에 한 번 돌려주세요)
- **주석**: 무엇을 하는지(WHAT)는 코드가 말하게 하고, 왜 하는지(WHY)만 주석으로

## 보안 / 책임

- 토큰 / API 키 / 개인키(`sumgim.key`) 등은 **절대 커밋 금지**
- `.gitignore`에 이미 있는 항목 외에 비밀이 들어갈 위치가 보이면 PR로 알려주세요

보안 취약점을 발견하셨다면 공개 이슈 대신 직접 연락 부탁드립니다 (메인테이너 GitHub 프로필의 이메일).

## 행동 강령

서로를 존중하고 친절하게 대해주세요. 인종, 성별, 성적 지향, 종교, 경력 등 어떤 이유로든 차별/괴롭힘은 용인되지 않습니다. 보고는 메인테이너에게 직접.

## 라이선스

기여하시면 [MIT License](../LICENSE) 하에 배포됩니다.

---

질문 / 막힘 / 그냥 인사 모두 환영입니다. [Discussions](https://github.com/minju-kim98/sumgim/discussions)에서 만나요!
