import { useEffect, useState } from "react";
import logo from "../assets/logo.png";
import {
  AppSettings,
  MessengerCreds,
  completeOnboarding,
  getMessengerCreds,
  getSettings,
  testMattermost,
  testSlack,
  updateMessengerCreds,
  updateSettings,
} from "../lib/api";

interface Props {
  onDone: (settings: AppSettings) => void;
}

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

export default function Onboarding({ onDone }: Props) {
  const [step, setStep] = useState<1 | 2 | 3>(1);
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [creds, setCreds] = useState<MessengerCreds>({
    mattermost_url: "",
    mattermost_token: "",
    slack_token: "",
    mattermost_status_emoji: "calendar",
    mattermost_status_text: "회의 중",
    slack_status_emoji: "calendar",
    slack_status_text: "회의 중",
  });
  const [mmTesting, setMmTesting] = useState(false);
  const [mmResult, setMmResult] = useState<string | null>(null);
  const [mmError, setMmError] = useState<string | null>(null);
  const [slackTesting, setSlackTesting] = useState(false);
  const [slackResult, setSlackResult] = useState<string | null>(null);
  const [slackError, setSlackError] = useState<string | null>(null);
  const [finishing, setFinishing] = useState(false);
  const [finishError, setFinishError] = useState<string | null>(null);

  useEffect(() => {
    (async () => {
      const s = await getSettings();
      setSettings(s);
      const c = await getMessengerCreds();
      setCreds(c);
    })();
  }, []);

  if (!settings) {
    return (
      <div className="settings">
        <p>불러오는 중…</p>
      </div>
    );
  }

  const updateTrigger = async (patch: Partial<AppSettings>) => {
    const next = await updateSettings(patch);
    setSettings(next);
  };

  const onTestMm = async () => {
    setMmTesting(true);
    setMmError(null);
    setMmResult(null);
    try {
      await updateMessengerCreds(creds);
      const name = await testMattermost(creds.mattermost_url, creds.mattermost_token);
      setMmResult(name);
      const next = await updateSettings({ mattermost_enabled: true });
      setSettings(next);
    } catch (e) {
      setMmError(String(e));
    } finally {
      setMmTesting(false);
    }
  };

  const onTestSlack = async () => {
    setSlackTesting(true);
    setSlackError(null);
    setSlackResult(null);
    try {
      await updateMessengerCreds(creds);
      const id = await testSlack(creds.slack_token);
      setSlackResult(id);
      const next = await updateSettings({ slack_enabled: true });
      setSettings(next);
    } catch (e) {
      setSlackError(String(e));
    } finally {
      setSlackTesting(false);
    }
  };

  const onFinish = async () => {
    setFinishing(true);
    setFinishError(null);
    try {
      const next = await completeOnboarding();
      onDone(next);
    } catch (e) {
      setFinishError(String(e));
    } finally {
      setFinishing(false);
    }
  };

  return (
    <div className="settings">
      <header>
        <img src={logo} alt="Sumgim logo" />
        <div>
          <h1>숨김 · Sumgim</h1>
          <p className="sub">3단계 초기 설정</p>
        </div>
      </header>

      <div className="row" style={{ gap: 8 }}>
        {[1, 2, 3].map((n) => (
          <div
            key={n}
            style={{
              flex: 1,
              height: 4,
              borderRadius: 2,
              background:
                n <= step ? "var(--accent, #6366f1)" : "var(--border, #e5e7eb)",
            }}
          />
        ))}
      </div>

      {step === 1 && (
        <section className="section">
          <h2>1. 언제 회의 모드를 자동으로 켤까요?</h2>
          <p className="desc">나중에 설정 화면에서 다시 바꿀 수 있습니다.</p>
          <div className="row">
            <div className="label">
              <div className="name">디스플레이 복제(Clone) 감지</div>
              <div className="desc">
                외부 모니터에 같은 화면이 복제되면 회의 모드 자동 ON. 가장 정확.
              </div>
            </div>
            <label className="switch">
              <input
                type="checkbox"
                checked={settings.auto_detect_clone}
                onChange={(e) =>
                  updateTrigger({ auto_detect_clone: e.target.checked })
                }
              />
              <span className="slider" />
            </label>
          </div>
          <div className="row">
            <div className="label">
              <div className="name">PowerPoint 슬라이드쇼 감지</div>
              <div className="desc">PPT 발표 모드 진입 시 자동 ON.</div>
            </div>
            <label className="switch">
              <input
                type="checkbox"
                checked={settings.detect_ppt}
                onChange={(e) =>
                  updateTrigger({ detect_ppt: e.target.checked })
                }
              />
              <span className="slider" />
            </label>
          </div>
          <div className="row">
            <div className="label">
              <div className="name">전체 화면 앱 감지</div>
              <div className="desc">
                전체화면이면 자동 ON. 영상/게임에서도 켜질 수 있어 기본 꺼짐.
              </div>
            </div>
            <label className="switch">
              <input
                type="checkbox"
                checked={settings.detect_fullscreen}
                onChange={(e) =>
                  updateTrigger({ detect_fullscreen: e.target.checked })
                }
              />
              <span className="slider" />
            </label>
          </div>
          <div style={{ display: "flex", justifyContent: "flex-end", marginTop: 12 }}>
            <button className="btn primary" onClick={() => setStep(2)}>
              다음 →
            </button>
          </div>
        </section>
      )}

      {step === 2 && (
        <section className="section">
          <h2>2. 메신저 연동 (선택)</h2>
          <p className="desc">
            회의 모드 ON 시 자동으로 DND + 상태 메시지를 바꾸고, 종료 시 놓친
            메시지 수를 알려줍니다. 지금 안 해도 됩니다 — 나중에 설정에서.
          </p>

          <div className="row" style={{ flexDirection: "column", alignItems: "stretch", gap: 6 }}>
            <label className="desc" style={{ fontSize: 12 }}>Mattermost 서버 URL</label>
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
            <input
              type="password"
              placeholder="xxxxxxxxxxxxxxxxxxxxxxxxxx"
              value={creds.mattermost_token}
              onChange={(e) => setCreds({ ...creds, mattermost_token: e.target.value })}
              style={{ padding: "6px 10px", border: "1px solid var(--border)", borderRadius: 6 }}
            />
            <div style={{ display: "flex", gap: 8, alignItems: "center", marginTop: 6 }}>
              <button className="btn" onClick={onTestMm} disabled={mmTesting}>
                {mmTesting ? "테스트 중…" : "Mattermost 연결"}
              </button>
              {mmResult && (
                <span style={{ color: "var(--success)", fontSize: 12 }}>
                  ✓ @{mmResult}
                </span>
              )}
              {mmError && (
                <span style={{ color: "var(--danger)", fontSize: 12 }}>✗ {mmError}</span>
              )}
            </div>
          </div>

          <div className="row" style={{ flexDirection: "column", alignItems: "stretch", gap: 6 }}>
            <label className="desc" style={{ fontSize: 12 }}>Slack User Token (xoxp-)</label>
            <input
              type="password"
              placeholder="xoxp-..."
              value={creds.slack_token}
              onChange={(e) => setCreds({ ...creds, slack_token: e.target.value })}
              style={{ padding: "6px 10px", border: "1px solid var(--border)", borderRadius: 6 }}
            />
            <div style={{ display: "flex", gap: 8, alignItems: "center", marginTop: 6 }}>
              <button className="btn" onClick={onTestSlack} disabled={slackTesting}>
                {slackTesting ? "테스트 중…" : "Slack 연결"}
              </button>
              {slackResult && (
                <span style={{ color: "var(--success)", fontSize: 12 }}>
                  ✓ {slackResult}
                </span>
              )}
              {slackError && (
                <span style={{ color: "var(--danger)", fontSize: 12 }}>✗ {slackError}</span>
              )}
            </div>
          </div>

          <div style={{ display: "flex", justifyContent: "space-between", marginTop: 12 }}>
            <button className="btn" onClick={() => setStep(1)}>
              ← 이전
            </button>
            <button className="btn primary" onClick={() => setStep(3)}>
              다음 →
            </button>
          </div>
        </section>
      )}

      {step === 3 && (
        <section className="section">
          <h2>3. 단축키 확인</h2>
          <p className="desc">언제든 이 단축키로 회의 모드를 즉시 켜고 끌 수 있습니다.</p>
          <div className="row">
            <div className="label">
              <div className="name">회의 모드 토글</div>
              <div className="desc">설정 화면에서 변경할 수 있습니다.</div>
            </div>
            <span className="kbd">{formatShortcut(settings.shortcut)}</span>
          </div>

          <div className="row">
            <div className="label">
              <div className="name">준비 완료</div>
              <div className="desc">
                마치면 트레이로 들어갑니다. 좌클릭 / 트레이 클릭 / 단축키로
                회의 모드를 토글하세요.
              </div>
            </div>
          </div>

          {finishError && (
            <div className="desc" style={{ color: "var(--danger)" }}>
              {finishError}
            </div>
          )}

          <div style={{ display: "flex", justifyContent: "space-between", marginTop: 12 }}>
            <button className="btn" onClick={() => setStep(2)} disabled={finishing}>
              ← 이전
            </button>
            <button className="btn primary" onClick={onFinish} disabled={finishing}>
              {finishing ? "마치는 중…" : "시작하기"}
            </button>
          </div>
        </section>
      )}
    </div>
  );
}
