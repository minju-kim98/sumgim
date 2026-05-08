use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::messenger::{self, MessengerCreds, UnreadSnapshot};

pub const DEFAULT_SHORTCUT: &str = "Ctrl+Alt+M";
pub const DEFAULT_TIMEOUT_MINUTES: u32 = 60;
pub const STORE_FILE: &str = "settings.json";

// Keys stored inside the tauri-plugin-store file.
pub const KEY_SHORTCUT: &str = "shortcut";
pub const KEY_SHOW_FLOATING: &str = "show_floating";
pub const KEY_AUTO_DETECT_CLONE: &str = "auto_detect_clone";
pub const KEY_TIMEOUT_MINUTES: &str = "timeout_minutes";
pub const KEY_TOAST_ENABLED_BACKUP: &str = "toast_enabled_backup";
pub const KEY_MEETING_ACTIVE: &str = "meeting_active";
pub const KEY_SUSPEND_KAKAOTALK: &str = "suspend_kakaotalk";
pub const KEY_DETECT_PPT: &str = "detect_ppt";
pub const KEY_DETECT_FULLSCREEN: &str = "detect_fullscreen";
pub const KEY_MATTERMOST_ENABLED: &str = "mattermost_enabled";
pub const KEY_SLACK_ENABLED: &str = "slack_enabled";
pub const KEY_MATTERMOST_URL: &str = "mattermost_url";
pub const KEY_MATTERMOST_TOKEN: &str = "mattermost_token";
pub const KEY_SLACK_TOKEN: &str = "slack_token";
pub const KEY_MM_STATUS_EMOJI: &str = "mattermost_status_emoji";
pub const KEY_MM_STATUS_TEXT: &str = "mattermost_status_text";
pub const KEY_SLACK_STATUS_EMOJI: &str = "slack_status_emoji";
pub const KEY_SLACK_STATUS_TEXT: &str = "slack_status_text";
pub const KEY_AUTOSTART: &str = "autostart";
pub const KEY_ONBOARDING_DONE: &str = "onboarding_done";

/// User-visible settings. `meeting_active` mirrors runtime state but is persisted for
/// crash recovery — if we crash while ON, the next launch should release DND.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub shortcut: String,
    pub show_floating: bool,
    pub auto_detect_clone: bool,
    pub timeout_minutes: u32,
    pub meeting_active: bool,
    pub suspend_kakaotalk: bool,
    pub detect_ppt: bool,
    pub detect_fullscreen: bool,
    pub mattermost_enabled: bool,
    pub slack_enabled: bool,
    pub autostart: bool,
    pub onboarding_done: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            shortcut: DEFAULT_SHORTCUT.to_string(),
            show_floating: false,
            auto_detect_clone: true,
            timeout_minutes: DEFAULT_TIMEOUT_MINUTES,
            meeting_active: false,
            suspend_kakaotalk: true,
            detect_ppt: true,
            detect_fullscreen: false,
            mattermost_enabled: false,
            slack_enabled: false,
            autostart: false,
            onboarding_done: false,
        }
    }
}

/// Why meeting mode was turned on. Used to decide whether a display-clone release
/// or timeout should auto-disable (it should only auto-disable auto-enables).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MeetingSource {
    Manual,
    Auto,
}

#[derive(Debug, Clone, Serialize)]
pub struct MeetingState {
    pub active: bool,
    pub source: Option<MeetingSource>,
    pub since_epoch_secs: Option<u64>,
}

impl Default for MeetingState {
    fn default() -> Self {
        Self {
            active: false,
            source: None,
            since_epoch_secs: None,
        }
    }
}

impl MeetingState {
    pub fn activate(&mut self, source: MeetingSource) {
        self.active = true;
        self.source = Some(source);
        self.since_epoch_secs = Some(now_secs());
    }

    pub fn deactivate(&mut self) {
        self.active = false;
        self.source = None;
        self.since_epoch_secs = None;
    }
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Runtime state shared across Tauri command handlers, tray menu, shortcut
/// callbacks, and the display-monitor thread. Must be `Send + Sync`.
pub struct AppState {
    pub settings: RwLock<AppSettings>,
    pub meeting: RwLock<MeetingState>,
    /// Backup of the user's original `ToastEnabled` registry value before we
    /// flipped it. Stored so we restore instead of blindly setting to 1.
    pub toast_enabled_backup: RwLock<Option<u32>>,
    /// Currently registered global shortcut string (lets us unregister when the
    /// user changes it from the settings UI).
    pub active_shortcut: RwLock<Option<String>>,
    pub unread_baseline: RwLock<Option<UnreadSnapshot>>,
    pub messenger_creds: RwLock<MessengerCreds>,
    pub status_backup: RwLock<messenger::CustomStatusBackup>,
}

impl AppState {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            settings: RwLock::new(AppSettings::default()),
            meeting: RwLock::new(MeetingState::default()),
            toast_enabled_backup: RwLock::new(None),
            active_shortcut: RwLock::new(None),
            unread_baseline: RwLock::new(None),
            messenger_creds: RwLock::new(MessengerCreds::default()),
            status_backup: RwLock::new(messenger::CustomStatusBackup::default()),
        })
    }
}
