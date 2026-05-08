import { useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import logo from "../assets/logo.png";
import { getMeetingState, onMeetingChanged, toggleMeeting } from "../lib/api";

export default function FloatingToggle() {
  const [active, setActive] = useState(false);
  const draggingRef = useRef(false);

  useEffect(() => {
    document.body.classList.add("floating-body");
    document.documentElement.style.background = "transparent";
    let unlisten: (() => void) | undefined;
    (async () => {
      const m = await getMeetingState();
      setActive(m.active);
      unlisten = await onMeetingChanged((state) => setActive(state.active));
    })();
    return () => {
      unlisten?.();
      document.body.classList.remove("floating-body");
    };
  }, []);

  async function onPointerDown(e: React.PointerEvent<HTMLDivElement>) {
    if (e.button !== 0) return;
    draggingRef.current = false;
    const startX = e.clientX;
    const startY = e.clientY;
    const win = getCurrentWindow();
    const onMove = (ev: PointerEvent) => {
      if (
        Math.abs(ev.clientX - startX) > 4 ||
        Math.abs(ev.clientY - startY) > 4
      ) {
        draggingRef.current = true;
        win.startDragging().catch(() => {});
        window.removeEventListener("pointermove", onMove);
      }
    };
    window.addEventListener("pointermove", onMove, { once: false });
    const cleanup = () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", cleanup);
    };
    window.addEventListener("pointerup", cleanup, { once: true });
  }

  async function onClick() {
    if (draggingRef.current) {
      draggingRef.current = false;
      return;
    }
    const next = await toggleMeeting("manual");
    setActive(next.active);
  }

  return (
    <div
      className={`floating ${active ? "on" : ""}`}
      onPointerDown={onPointerDown}
      onClick={onClick}
      title={active ? "회의 모드 ON (클릭 = 끄기)" : "회의 모드 OFF (클릭 = 켜기)"}
    >
      <img src={logo} alt="toggle" />
    </div>
  );
}
