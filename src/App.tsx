import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Settings from "./pages/Settings";
import Onboarding from "./pages/Onboarding";
import FloatingToggle from "./pages/FloatingToggle";
import { AppSettings, getSettings } from "./lib/api";

function App() {
  const [label, setLabel] = useState<string | null>(null);
  const [settings, setSettings] = useState<AppSettings | null>(null);

  useEffect(() => {
    const w = getCurrentWindow();
    setLabel(w.label);
    if (w.label !== "floating") {
      getSettings().then(setSettings);
    }
  }, []);

  if (label === null) {
    return null;
  }

  if (label === "floating") {
    return <FloatingToggle />;
  }

  if (settings === null) {
    return null;
  }

  if (!settings.onboarding_done) {
    return <Onboarding onDone={(s) => setSettings(s)} />;
  }

  return <Settings />;
}

export default App;
