import { useEffect, useRef, useState } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import logo from "../assets/logo.png";
import {
  AppSettings,
  MessengerCreds,
  getMeetingState,
  getMessengerCreds,
  getSettings,
  onMeetingChanged,
  setAutostart,
  setFloatingVisible,
  setShortcut as apiSetShortcut,
  testMattermost,
  testSlack,
  toggleMeeting,
  updateMessengerCreds,
  updateSettings,
} from "../lib/api";

const MODIFIER_LABELS: Record<string, string> = {
  Ctrl: "Ctrl",
  Alt: "Alt",
  Shift: "Shift",
  Meta: "Win",
};

function formatShortcut(s: string): string {
  return s
    .split("+")
    .map((p) => MODIFIER_LABELS[p] ?? p)
    .join(" + ");
}

function captureShortcut(e: KeyboardEvent): string | null {
  const parts: string[] = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Meta");
  const key = e.key;
  if (["Control", "Alt", "Shift", "Meta"].includes(key)) return null;
  let main = key.length === 1 ? key.toUpperCase() : key;
  if (/^F[0-9]+$/.test(key)) main = key;
  parts.push(main);
  if (parts.length < 2) return null;
  return parts.join("+");
}

export default function Settings() {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [active, setActive] = useState(false);
  const [recording, setRecording] = useState(false);
  const [recordingError, setRecordingError] = useState<string | null>(null);
  const [creds, setCreds] = useState<MessengerCreds>({
    mattermost_url: "",
    mattermost_token: "",
    slack_token: "",
    mattermost_status_emoji: "calendar",
    mattermost_status_text: "회의 중",
    slack_status_emoji: "calendar",
    slack_status_text: "회의 중",
  });
  const [mmTestResult, setMmTestResult] = useState<
    { ok: true; value: string } | { ok: false; error: string } | null
  >(null);
  const [slackTestResult, setSlackTestResult] = useState<
    { ok: true; value: string } | { ok: false; error: string } | null
  >(null);
  const [mmTokenVisible, setMmTokenVisible] = useState(false);
  const [slackTokenVisible, setSlackTokenVisible] = useState(false);
  const [mmTesting, setMmTesting] = useState(false);
  const [slackTesting, setSlackTesting] = useState(false);
  const [autostartError, setAutostartError] = useState<string | null>(null);
  const [updateStatus, setUpdateStatus] = useState<
    | { kind: "idle" }
    | { kind: "checking" }
    | { kind: "latest"; current: string }
    | { kind: "available"; update: Update }
    | { kind: "downloading"; downloaded: number; total: number | null }
    | { kind: "ready" }
    | { kind: "error"; message: string }
  >({ kind: "idle" });
  const recordingRef = useRef(recording);
  recordingRef.current = recording;

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    (async () => {
      const s = await getSettings();
      setSettings(s);
      const m = await getMeetingState();
      setActive(m.active);
      const c = await getMessengerCreds();
      setCreds(c);
      unlisten = await onMeetingChanged((state) => setActive(state.active));
    })();
    return () => {
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    if (!recording) return;
    function onKey(e: KeyboardEvent) {
      e.preventDefault();
      if (e.key === "Escape") {
        setRecording(false);
        setRecordingError(null);
        return;
      }
      const captured = captureShortcut(e);
      if (!captured) return;
      (async () => {
        try {
          await apiSetShortcut(captured);
          setSettings((prev) => (prev ? { ...prev, shortcut: captured } : prev));
          setRecording(false);
          setRecordingError(null);
        } catch (err) {
          setRecordingError(String(err));
        }
      })();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [recording]);

  if (!settings) {
    return (
      <div className="settings">
        <p>불러오는 중…</p>
      </div>
    );
  }

  const onToggleMeeting = async () => {
    const next = await toggleMeeting("manual");
    setActive(next.active);
  };

  const onToggleFloating = async (checked: boolean) => {
    await setFloatingVisible(checked);
    const s = await updateSettings({ show_floating: checked });
    setSettings(s);
  };

  const onToggleAutoClone = async (checked: boolean) => {
    const s = await updateSettings({ auto_detect_clone: checked });
    setSettings(s);
  };

  const onToggleDetectPpt = async (checked: boolean) => {
    const s = await updateSettings({ detect_ppt: checked });
    setSettings(s);
  };

  const onToggleDetectFullscreen = async (checked: boolean) => {
    const s = await updateSettings({ detect_fullscreen: checked });
    setSettings(s);
  };

  const onToggleSuspendKakao = async (checked: boolean) => {
    const s = await updateSettings({ suspend_kakaotalk: checked });
    setSettings(s);
  };

  const onChangeTimeout = async (v: number) => {
    const s = await updateSettings({ timeout_minutes: v });
    setSettings(s);
  };

  const onToggleAutostart = async (checked: boolean) => {
    try {
      await setAutostart(checked);
      setSettings((prev) => (prev ? { ...prev, autostart: checked } : prev));
      setAutostartError(null);
    } catch (e) {
      setAutostartError(String(e));
    }
  };

  const saveCreds = async (next: MessengerCreds) => {
    setCreds(next);
    await updateMessengerCreds(next);
  };

  const onTestMattermost = async () => {
    setMmTesting(true);
    setMmTestResult(null);
    try {
      await updateMessengerCreds(creds);
      const name = await testMattermost(creds.mattermost_url, creds.mattermost_token);
      setMmTestResult({ ok: true, value: name });
    } catch (err) {
      setMmTestResult({ ok: false, error: String(err) });
    } finally {
      setMmTesting(false);
    }
  };

  const onTestSlack = async () => {
    setSlackTesting(true);
    setSlackTestResult(null);
    try {
      await updateMessengerCreds(creds);
      const id = await testSlack(creds.slack_token);
      setSlackTestResult({ ok: true, value: id });
    } catch (err) {
      setSlackTestResult({ ok: false, error: String(err) });
    } finally {
      setSlackTesting(false);
    }
  };

  const onToggleMattermost = async (checked: boolean) => {
    if (checked && !(mmTestResult && mmTestResult.ok)) {
      setMmTestResult({ ok: false, error: "연결 테스트를 먼저 성공시켜 주세요" });
      return;
    }
    const s = await updateSettings({ mattermost_enabled: checked });
    setSettings(s);
  };

  const onCheckUpdate = async () => {
    setUpdateStatus({ kind: "checking" });
    try {
      const update = await check();
      if (update) {
        setUpdateStatus({ kind: "available", update });
      } else {
        setUpdateStatus({ kind: "latest", current: "최신" });
      }
    } catch (e) {
      setUpdateStatus({ kind: "error", message: String(e) });
    }
  };

  const onInstallUpdate = async () => {
    if (updateStatus.kind !== "available") return;
    const update = updateStatus.update;
    setUpdateStatus({ kind: "downloading", downloaded: 0, total: null });
    try {
      let downloaded = 0;
      let total: number | null = null;
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") {
          total = event.data.contentLength ?? null;
          setUpdateStatus({ kind: "downloading", downloaded: 0, total });
        } else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          setUpdateStatus({ kind: "downloading", downloaded, total });
        } else if (event.event === "Finished") {
          setUpdateStatus({ kind: "ready" });
        }
      });
      await relaunch();
    } catch (e) {
      setUpdateStatus({ kind: "error", message: String(e) });
    }
  };

  const onToggleSlack = async (checked: boolean) => {
    if (checked && !(slackTestResult && slackTestResult.ok)) {
      setSlackTestResult({ ok: false, error: "연결 테스트를 먼저 성공시켜 주세요" });
      return;
    }
    const s = await updateSettings({ slack_enabled: checked });
    setSettings(s);
  };

  return (
    <div className="settings">
      <header>
        <img src={logo} alt="Sumgim logo" />
        <div>
          <h1>숨김 · Sumgim</h1>
          <p className="sub">회의 모드 자동화</p>
        </div>
      </header>

      <section className="section">
        <h2>회의 모드</h2>
        <div className="row">
          <div className="label">
            <div className="name">현재 상태</div>
            <div className="desc">
              Windows 토스트 알림 차단 + Presentation Mode.
            </div>
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <span className={`state ${active ? "on" : "off"}`}>
              {active ? "ON" : "OFF"}
            </span>
            <button
              className={`btn ${active ? "" : "primary"}`}
              onClick={onToggleMeeting}
            >
              {active ? "끄기" : "켜기"}
            </button>
          </div>
        </div>
        <div className="row">
          <div className="label">
            <div className="name">카카오톡 일시 정지</div>
            <div className="desc">
              회의 모드 동안 KakaoTalk.exe를 일시 정지해 팝업을 원천 차단합니다.
              회의가 끝나면 밀린 메시지가 한꺼번에 수신됩니다.
            </div>
          </div>
          <label className="switch">
            <input
              type="checkbox"
              checked={settings.suspend_kakaotalk}
              onChange={(e) => onToggleSuspendKakao(e.target.checked)}
            />
            <span className="slider" />
          </label>
        </div>
      </section>

      <section className="section">
        <h2>시작 프로그램</h2>
        <div className="row">
          <div className="label">
            <div className="name">Windows 시작 시 자동 실행</div>
            <div className="desc">
              부팅 후 트레이에 바로 대기 상태로 실행됩니다.
            </div>
            {autostartError && (
              <div className="desc" style={{ color: "var(--danger)" }}>
                자동 실행 설정 실패: {autostartError}
              </div>
            )}
          </div>
          <label className="switch">
            <input
              type="checkbox"
              checked={settings.autostart}
              onChange={(e) => onToggleAutostart(e.target.checked)}
            />
            <span className="slider" />
          </label>
        </div>
      </section>

      <section className="section">
        <h2>단축키</h2>
        <div className="row">
          <div className="label">
            <div className="name">회의 모드 토글</div>
            <div className="desc">
              전역 단축키. 새 키 조합을 기록하려면 버튼을 눌러주세요.
            </div>
            {recordingError && (
              <div className="desc" style={{ color: "var(--danger)" }}>
                단축키를 등록하지 못했습니다: {recordingError}
              </div>
            )}
          </div>
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
            <span className="kbd">{formatShortcut(settings.shortcut)}</span>
            <button
              className={`btn ${recording ? "danger" : ""}`}
              onClick={() => {
                setRecordingError(null);
                setRecording((r) => !r);
              }}
            >
              {recording ? "취소 (Esc)" : "변경"}
            </button>
          </div>
        </div>
      </section>

      <section className="section">
        <h2>Floating 버튼</h2>
        <div className="row">
          <div className="label">
            <div className="name">항상 위 토글 버튼 표시</div>
            <div className="desc">반투명 원형 버튼. 드래그해서 위치 이동 가능.</div>
          </div>
          <label className="switch">
            <input
              type="checkbox"
              checked={settings.show_floating}
              onChange={(e) => onToggleFloating(e.target.checked)}
            />
            <span className="slider" />
          </label>
        </div>
      </section>

      <section className="section">
        <h2>자동 감지</h2>
        <div className="row">
          <div className="label">
            <div className="name">디스플레이 복제(Clone) 감지</div>
            <div className="desc">
              외부 모니터에 같은 화면이 복제되면 자동으로 회의 모드를 켭니다.
            </div>
          </div>
          <label className="switch">
            <input
              type="checkbox"
              checked={settings.auto_detect_clone}
              onChange={(e) => onToggleAutoClone(e.target.checked)}
            />
            <span className="slider" />
          </label>
        </div>
        <div className="row">
          <div className="label">
            <div className="name">PowerPoint 슬라이드쇼 감지</div>
            <div className="desc">
              PPT 슬라이드쇼 모드 진입 시 자동으로 회의 모드 켜기
            </div>
          </div>
          <label className="switch">
            <input
              type="checkbox"
              checked={settings.detect_ppt}
              onChange={(e) => onToggleDetectPpt(e.target.checked)}
            />
            <span className="slider" />
          </label>
        </div>
        <div className="row">
          <div className="label">
            <div className="name">전체 화면 앱 감지</div>
            <div className="desc">
              전체 화면 앱이 포어그라운드로 오면 자동으로 회의 모드 켜기. 영상 재생/게임 등에서 오탐할 수 있어 기본 꺼짐
            </div>
          </div>
          <label className="switch">
            <input
              type="checkbox"
              checked={settings.detect_fullscreen}
              onChange={(e) => onToggleDetectFullscreen(e.target.checked)}
            />
            <span className="slider" />
          </label>
        </div>
        <div className="row">
          <div className="label">
            <div className="name">자동 해제 타임아웃 (분)</div>
            <div className="desc">
              자동으로 켜진 회의 모드가 이 시간이 지나면 해제됩니다.
            </div>
          </div>
          <input
            type="number"
            min={5}
            max={240}
            value={settings.timeout_minutes}
            onChange={(e) => onChangeTimeout(Number(e.target.value))}
            style={{ width: 80 }}
          />
        </div>
      </section>

      <section className="section">
        <h2>메신저 연동</h2>
        <p className="desc" style={{ fontSize: 12, color: "var(--muted)", marginTop: 0 }}>
          회의 모드 ON 시 선택한 메신저를 DND(방해금지)로 바꾸고, 회의가 끝나면
          놓친 메시지 수를 토스트로 보여줍니다.
        </p>
        <p className="desc" style={{ fontSize: 12, color: "var(--muted)", marginTop: 0 }}>
          회의 모드 ON 시 상태 메시지가 설정한 내용으로 바뀌고,
          OFF 시 이전 상태로 자동 복구됩니다.
        </p>
        <div className="row">
          <div className="label">
            <div className="name">Mattermost</div>
            <div className="desc">연결 테스트 성공 후 활성화할 수 있습니다.</div>
          </div>
          <label className="switch">
            <input
              type="checkbox"
              checked={settings.mattermost_enabled}
              onChange={(e) => onToggleMattermost(e.target.checked)}
            />
            <span className="slider" />
          </label>
        </div>
        <div className="row" style={{ flexDirection: "column", alignItems: "stretch", gap: 6 }}>
          <label className="desc" style={{ fontSize: 12 }}>서버 URL</label>
          <input
            type="text"
            placeholder="https://mm.company.com"
            value={creds.mattermost_url}
            onChange={(e) => setCreds({ ...creds, mattermost_url: e.target.value })}
            style={{ padding: "6px 10px", border: "1px solid var(--border)", borderRadius: 6 }}
          />
          <label className="desc" style={{ fontSize: 12, marginTop: 4 }}>
            Personal Access Token
          </label>
          <div style={{ display: "flex", gap: 6 }}>
            <input
              type={mmTokenVisible ? "text" : "password"}
              placeholder="xxxxxxxxxxxxxxxxxxxxxxxxxx"
              value={creds.mattermost_token}
              onChange={(e) => setCreds({ ...creds, mattermost_token: e.target.value })}
              style={{
                flex: 1,
                padding: "6px 10px",
                border: "1px solid var(--border)",
                borderRadius: 6,
              }}
            />
            <button
              className="btn"
              type="button"
              onClick={() => setMmTokenVisible((v) => !v)}
            >
              {mmTokenVisible ? "숨김" : "표시"}
            </button>
          </div>
          <div style={{ display: "flex", gap: 8, alignItems: "center", marginTop: 6 }}>
            <button
              className="btn"
              type="button"
              disabled={mmTesting}
              onClick={onTestMattermost}
            >
              {mmTesting ? "테스트 중…" : "연결 테스트"}
            </button>
            <button
              className="btn primary"
              type="button"
              onClick={() => saveCreds(creds)}
            >
              저장
            </button>
            {mmTestResult && mmTestResult.ok && (
              <span style={{ color: "var(--success)", fontSize: 12 }}>
                ✓ @{mmTestResult.value}
              </span>
            )}
            {mmTestResult && !mmTestResult.ok && (
              <span style={{ color: "var(--danger)", fontSize: 12 }}>
                ✗ {mmTestResult.error}
              </span>
            )}
          </div>
          <div style={{ display: "flex", gap: 8, marginTop: 6, alignItems: "flex-end" }}>
            <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
              <label className="desc" style={{ fontSize: 12 }}>이모지 이름</label>
              <input
                type="text"
                placeholder="calendar"
                value={creds.mattermost_status_emoji}
                onChange={(e) =>
                  setCreds({ ...creds, mattermost_status_emoji: e.target.value })
                }
                style={{
                  width: 120,
                  padding: "6px 10px",
                  border: "1px solid var(--border)",
                  borderRadius: 6,
                }}
              />
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 4, flex: 1 }}>
              <label className="desc" style={{ fontSize: 12 }}>상태 문구</label>
              <input
                type="text"
                placeholder="회의 중"
                value={creds.mattermost_status_text}
                onChange={(e) =>
                  setCreds({ ...creds, mattermost_status_text: e.target.value })
                }
                style={{
                  width: 240,
                  padding: "6px 10px",
                  border: "1px solid var(--border)",
                  borderRadius: 6,
                }}
              />
            </div>
          </div>
          <div className="desc" style={{ fontSize: 11, color: "var(--muted)" }}>
            이모지 이름만 입력 (예: calendar, computer, coffee)
          </div>
        </div>
        <div className="row">
          <div className="label">
            <div className="name">Slack</div>
            <div className="desc">연결 테스트 성공 후 활성화할 수 있습니다.</div>
          </div>
          <label className="switch">
            <input
              type="checkbox"
              checked={settings.slack_enabled}
              onChange={(e) => onToggleSlack(e.target.checked)}
            />
            <span className="slider" />
          </label>
        </div>
        <div className="row" style={{ flexDirection: "column", alignItems: "stretch", gap: 6 }}>
          <label className="desc" style={{ fontSize: 12 }}>User Token (xoxp-)</label>
          <div style={{ display: "flex", gap: 6 }}>
            <input
              type={slackTokenVisible ? "text" : "password"}
              placeholder="xoxp-..."
              value={creds.slack_token}
              onChange={(e) => setCreds({ ...creds, slack_token: e.target.value })}
              style={{
                flex: 1,
                padding: "6px 10px",
                border: "1px solid var(--border)",
                borderRadius: 6,
              }}
            />
            <button
              className="btn"
              type="button"
              onClick={() => setSlackTokenVisible((v) => !v)}
            >
              {slackTokenVisible ? "숨김" : "표시"}
            </button>
          </div>
          <div style={{ display: "flex", gap: 8, alignItems: "center", marginTop: 6 }}>
            <button
              className="btn"
              type="button"
              disabled={slackTesting}
              onClick={onTestSlack}
            >
              {slackTesting ? "테스트 중…" : "연결 테스트"}
            </button>
            <button
              className="btn primary"
              type="button"
              onClick={() => saveCreds(creds)}
            >
              저장
            </button>
            {slackTestResult && slackTestResult.ok && (
              <span style={{ color: "var(--success)", fontSize: 12 }}>
                ✓ {slackTestResult.value}
              </span>
            )}
            {slackTestResult && !slackTestResult.ok && (
              <span style={{ color: "var(--danger)", fontSize: 12 }}>
                ✗ {slackTestResult.error}
              </span>
            )}
          </div>
          <div style={{ display: "flex", gap: 8, marginTop: 6, alignItems: "flex-end" }}>
            <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
              <label className="desc" style={{ fontSize: 12 }}>이모지 이름</label>
              <input
                type="text"
                placeholder="calendar"
                value={creds.slack_status_emoji}
                onChange={(e) =>
                  setCreds({ ...creds, slack_status_emoji: e.target.value })
                }
                style={{
                  width: 120,
                  padding: "6px 10px",
                  border: "1px solid var(--border)",
                  borderRadius: 6,
                }}
              />
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: 4, flex: 1 }}>
              <label className="desc" style={{ fontSize: 12 }}>상태 문구</label>
              <input
                type="text"
                placeholder="회의 중"
                value={creds.slack_status_text}
                onChange={(e) =>
                  setCreds({ ...creds, slack_status_text: e.target.value })
                }
                style={{
                  width: 240,
                  padding: "6px 10px",
                  border: "1px solid var(--border)",
                  borderRadius: 6,
                }}
              />
            </div>
          </div>
          <div className="desc" style={{ fontSize: 11, color: "var(--muted)" }}>
            이모지 이름만 입력 (예: calendar, computer, coffee)
          </div>
        </div>
      </section>

      <section className="section">
        <h2>업데이트</h2>
        <div className="row">
          <div className="label">
            <div className="name">자동 업데이트</div>
            <div className="desc">
              GitHub Releases에서 최신 빌드를 확인하고 백그라운드로 받아서
              재시작 시 적용합니다.
            </div>
            {updateStatus.kind === "available" && (
              <div className="desc" style={{ marginTop: 6 }}>
                <strong>v{updateStatus.update.version}</strong> 사용 가능
                {updateStatus.update.body && (
                  <pre
                    style={{
                      whiteSpace: "pre-wrap",
                      fontSize: 11,
                      color: "var(--muted)",
                      margin: "6px 0 0",
                      maxHeight: 120,
                      overflowY: "auto",
                    }}
                  >
                    {updateStatus.update.body}
                  </pre>
                )}
              </div>
            )}
            {updateStatus.kind === "downloading" && (
              <div className="desc" style={{ marginTop: 6 }}>
                다운로드 중…{" "}
                {updateStatus.total
                  ? `${Math.round(
                      (updateStatus.downloaded / updateStatus.total) * 100,
                    )}%`
                  : `${(updateStatus.downloaded / 1024 / 1024).toFixed(1)} MB`}
              </div>
            )}
            {updateStatus.kind === "ready" && (
              <div className="desc" style={{ marginTop: 6 }}>
                설치 준비 완료. 곧 재시작됩니다.
              </div>
            )}
            {updateStatus.kind === "latest" && (
              <div className="desc" style={{ marginTop: 6, color: "var(--success)" }}>
                ✓ 최신 버전입니다.
              </div>
            )}
            {updateStatus.kind === "error" && (
              <div className="desc" style={{ marginTop: 6, color: "var(--danger)" }}>
                ✗ {updateStatus.message}
              </div>
            )}
          </div>
          <div style={{ display: "flex", gap: 8 }}>
            {updateStatus.kind === "available" ? (
              <button className="btn primary" onClick={onInstallUpdate}>
                다운로드 및 설치
              </button>
            ) : (
              <button
                className="btn"
                onClick={onCheckUpdate}
                disabled={
                  updateStatus.kind === "checking" ||
                  updateStatus.kind === "downloading" ||
                  updateStatus.kind === "ready"
                }
              >
                {updateStatus.kind === "checking" ? "확인 중…" : "업데이트 확인"}
              </button>
            )}
          </div>
        </div>
      </section>

      <section className="section">
        <h2>정보</h2>
        <div className="row">
          <div className="label">
            <div className="name">숨김 v0.3.0</div>
            <div className="desc">Author · Minju</div>
          </div>
          <a
            href="https://github.com/"
            target="_blank"
            rel="noreferrer"
            className="btn"
          >
            GitHub
          </a>
        </div>
      </section>

      <div className="footer">
        창을 닫으면 트레이로 숨겨집니다. 완전히 종료하려면 트레이 메뉴에서
        <strong> 종료</strong>를 눌러주세요.
      </div>
    </div>
  );
}
