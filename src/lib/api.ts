import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

export interface AppSettings {
  shortcut: string;
  show_floating: boolean;
  auto_detect_clone: boolean;
  timeout_minutes: number;
  meeting_active: boolean;
  suspend_kakaotalk: boolean;
  detect_ppt: boolean;
  detect_fullscreen: boolean;
  mattermost_enabled: boolean;
  slack_enabled: boolean;
  autostart: boolean;
  onboarding_done: boolean;
}

export interface MessengerCreds {
  mattermost_url: string;
  mattermost_token: string;
  slack_token: string;
  mattermost_status_emoji: string;
  mattermost_status_text: string;
  slack_status_emoji: string;
  slack_status_text: string;
}

export interface MeetingState {
  active: boolean;
  source: "manual" | "auto" | null;
  since_epoch_secs: number | null;
}

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function updateSettings(
  patch: Partial<AppSettings>,
): Promise<AppSettings> {
  return invoke<AppSettings>("update_settings", { patch });
}

export async function getMeetingState(): Promise<MeetingState> {
  return invoke<MeetingState>("get_meeting_state");
}

export async function toggleMeeting(source: "manual" | "auto" = "manual"): Promise<MeetingState> {
  return invoke<MeetingState>("toggle_meeting", { source });
}

export async function setMeeting(
  active: boolean,
  source: "manual" | "auto" = "manual",
): Promise<MeetingState> {
  return invoke<MeetingState>("set_meeting", { active, source });
}

export async function setShortcut(shortcut: string): Promise<void> {
  return invoke("set_shortcut", { shortcut });
}

export async function setFloatingVisible(visible: boolean): Promise<void> {
  return invoke("set_floating_visible", { visible });
}

export async function setAutostart(enabled: boolean): Promise<void> {
  await invoke("set_autostart", { enabled });
}

export async function quitApp(): Promise<void> {
  return invoke("quit_app");
}

export async function completeOnboarding(): Promise<AppSettings> {
  return invoke<AppSettings>("complete_onboarding");
}

export async function getMessengerCreds(): Promise<MessengerCreds> {
  return invoke<MessengerCreds>("get_messenger_creds");
}

export async function updateMessengerCreds(creds: MessengerCreds): Promise<void> {
  return invoke("update_messenger_creds", { creds });
}

export async function testMattermost(url: string, token: string): Promise<string> {
  return invoke<string>("test_mattermost_connection", { url, token });
}

export async function testSlack(token: string): Promise<string> {
  return invoke<string>("test_slack_connection", { token });
}

export function onMeetingChanged(
  cb: (state: MeetingState) => void,
): Promise<UnlistenFn> {
  return listen<MeetingState>("meeting-changed", (event) => cb(event.payload));
}

export function onUpdateCheckTriggered(cb: () => void): Promise<UnlistenFn> {
  return listen<null>("trigger-update-check", () => cb());
}

export interface MissedAlertItem {
  name: string;
  count: number;
}

export function onMissedAlerts(
  cb: (items: MissedAlertItem[]) => void,
): Promise<UnlistenFn> {
  return listen<MissedAlertItem[]>("missed-alerts", (event) => cb(event.payload));
}
