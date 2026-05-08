import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import Settings from "./pages/Settings";
import FloatingToggle from "./pages/FloatingToggle";

// Route by window label: the floating toggle window has its own label.
function App() {
  const [label, setLabel] = useState<string | null>(null);

  useEffect(() => {
    const w = getCurrentWindow();
    setLabel(w.label);
  }, []);

  if (label === null) {
    return null;
  }

  if (label === "floating") {
    return <FloatingToggle />;
  }

  return <Settings />;
}

export default App;
