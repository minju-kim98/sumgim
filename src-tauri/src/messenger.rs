use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::time::Duration;
use tokio::runtime::Runtime;

static RT: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("failed to build messenger tokio runtime")
});

fn http_client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .context("failed to build reqwest client")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessengerCreds {
    pub mattermost_url: String,
    pub mattermost_token: String,
    pub slack_token: String,
    pub mattermost_status_emoji: String,
    pub mattermost_status_text: String,
    pub slack_status_emoji: String,
    pub slack_status_text: String,
}

impl Default for MessengerCreds {
    fn default() -> Self {
        Self {
            mattermost_url: String::new(),
            mattermost_token: String::new(),
            slack_token: String::new(),
            mattermost_status_emoji: "calendar".to_string(),
            mattermost_status_text: "회의 중".to_string(),
            slack_status_emoji: "calendar".to_string(),
            slack_status_text: "회의 중".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnreadSnapshot {
    pub mattermost: Option<u64>,
    pub slack: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStatus {
    pub emoji: String,
    pub text: String,
    pub expires_at_unix: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomStatusBackup {
    pub mattermost: Option<CustomStatus>,
    pub slack: Option<CustomStatus>,
}

pub fn enable_dnd_mattermost_only(creds: &MessengerCreds) -> Result<()> {
    RT.block_on(async { set_mattermost_status_async(creds, "dnd").await })
}

pub fn disable_dnd_mattermost_only(creds: &MessengerCreds) -> Result<()> {
    RT.block_on(async { set_mattermost_status_async(creds, "online").await })
}

pub fn enable_dnd_slack_only(creds: &MessengerCreds) -> Result<()> {
    RT.block_on(async { enable_dnd_slack_async(creds).await })
}

pub fn disable_dnd_slack_only(creds: &MessengerCreds) -> Result<()> {
    RT.block_on(async { disable_dnd_slack_async(creds).await })
}

pub fn snapshot_unread(creds: &MessengerCreds) -> UnreadSnapshot {
    RT.block_on(async { snapshot_unread_async(creds).await })
}

pub fn capture_mm_status(creds: &MessengerCreds) -> Option<CustomStatus> {
    RT.block_on(async { capture_mm_status_async(creds).await })
}

pub fn capture_slack_status(creds: &MessengerCreds) -> Option<CustomStatus> {
    RT.block_on(async { capture_slack_status_async(creds).await })
}

pub fn apply_mm_status(creds: &MessengerCreds) -> Result<()> {
    RT.block_on(async { apply_mm_status_async(creds).await })
}

pub fn clear_mm_status(creds: &MessengerCreds, restore: Option<&CustomStatus>) -> Result<()> {
    RT.block_on(async { clear_mm_status_async(creds, restore).await })
}

pub fn apply_slack_status(creds: &MessengerCreds) -> Result<()> {
    RT.block_on(async { apply_slack_status_async(creds).await })
}

pub fn clear_slack_status(creds: &MessengerCreds, restore: Option<&CustomStatus>) -> Result<()> {
    RT.block_on(async { clear_slack_status_async(creds, restore).await })
}

pub fn test_mattermost(url: &str, token: &str) -> Result<String> {
    RT.block_on(async { test_mattermost_async(url, token).await })
}

pub fn test_slack(token: &str) -> Result<String> {
    RT.block_on(async { test_slack_async(token).await })
}

// ───────────────── Mattermost ─────────────────

fn trim_mattermost_url(url: &str) -> &str {
    url.trim_end_matches('/')
}

async fn get_mattermost_user_id(client: &Client, url: &str, token: &str) -> Result<String> {
    let base = trim_mattermost_url(url);
    let resp = client
        .get(format!("{}/api/v4/users/me", base))
        .bearer_auth(token)
        .send()
        .await
        .context("mattermost /users/me request failed")?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("mattermost /users/me {}: {}", status, body));
    }
    let v: Value = resp
        .json()
        .await
        .context("mattermost /users/me: invalid json")?;
    let id = v
        .get("id")
        .and_then(|x| x.as_str())
        .ok_or_else(|| anyhow!("mattermost /users/me: no id field"))?;
    Ok(id.to_string())
}

async fn set_mattermost_status_async(creds: &MessengerCreds, status: &str) -> Result<()> {
    if creds.mattermost_url.is_empty() || creds.mattermost_token.is_empty() {
        return Err(anyhow!("mattermost credentials missing"));
    }
    let client = http_client()?;
    let base = trim_mattermost_url(&creds.mattermost_url);
    let user_id = get_mattermost_user_id(&client, base, &creds.mattermost_token).await?;
    let resp = client
        .put(format!("{}/api/v4/users/{}/status", base, user_id))
        .bearer_auth(&creds.mattermost_token)
        .json(&json!({ "user_id": user_id, "status": status }))
        .send()
        .await
        .context("mattermost set status request failed")?;
    if !resp.status().is_success() {
        let st = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("mattermost set status {}: {}", st, body));
    }
    Ok(())
}

async fn capture_mm_status_async(creds: &MessengerCreds) -> Option<CustomStatus> {
    if creds.mattermost_url.is_empty() || creds.mattermost_token.is_empty() {
        return None;
    }
    let client = http_client().ok()?;
    let base = trim_mattermost_url(&creds.mattermost_url);
    let resp = client
        .get(format!("{}/api/v4/users/me", base))
        .bearer_auth(&creds.mattermost_token)
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let v: Value = resp.json().await.ok()?;
    let props = v.get("props")?;
    let raw = props.get("customStatus").and_then(|x| x.as_str())?;
    let parsed: Value = serde_json::from_str(raw).ok()?;
    let emoji = parsed
        .get("emoji")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let text = parsed
        .get("text")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    if emoji.is_empty() && text.is_empty() {
        return None;
    }
    Some(CustomStatus {
        emoji,
        text,
        expires_at_unix: None,
    })
}

async fn apply_mm_status_async(creds: &MessengerCreds) -> Result<()> {
    if creds.mattermost_url.is_empty() || creds.mattermost_token.is_empty() {
        return Err(anyhow!("mattermost credentials missing"));
    }
    let client = http_client()?;
    let base = trim_mattermost_url(&creds.mattermost_url);
    let resp = client
        .put(format!("{}/api/v4/users/me/status/custom", base))
        .bearer_auth(&creds.mattermost_token)
        .json(&json!({
            "emoji": creds.mattermost_status_emoji,
            "text": creds.mattermost_status_text,
        }))
        .send()
        .await
        .context("mattermost set custom status request failed")?;
    if !resp.status().is_success() {
        let st = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("mattermost set custom status {}: {}", st, body));
    }
    Ok(())
}

async fn clear_mm_status_async(
    creds: &MessengerCreds,
    restore: Option<&CustomStatus>,
) -> Result<()> {
    if creds.mattermost_url.is_empty() || creds.mattermost_token.is_empty() {
        return Err(anyhow!("mattermost credentials missing"));
    }
    let client = http_client()?;
    let base = trim_mattermost_url(&creds.mattermost_url);
    if let Some(r) = restore {
        let resp = client
            .put(format!("{}/api/v4/users/me/status/custom", base))
            .bearer_auth(&creds.mattermost_token)
            .json(&json!({ "emoji": r.emoji, "text": r.text }))
            .send()
            .await
            .context("mattermost restore custom status request failed")?;
        if !resp.status().is_success() {
            let st = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("mattermost restore custom status {}: {}", st, body));
        }
    } else {
        let resp = client
            .delete(format!("{}/api/v4/users/me/status/custom", base))
            .bearer_auth(&creds.mattermost_token)
            .send()
            .await
            .context("mattermost delete custom status request failed")?;
        if !resp.status().is_success() {
            let st = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("mattermost delete custom status {}: {}", st, body));
        }
    }
    Ok(())
}

async fn fetch_mattermost_unread(creds: &MessengerCreds) -> Result<u64> {
    let client = http_client()?;
    let base = trim_mattermost_url(&creds.mattermost_url);
    let user_id = get_mattermost_user_id(&client, base, &creds.mattermost_token).await?;
    let resp = client
        .get(format!("{}/api/v4/users/{}/teams/unread", base, user_id))
        .bearer_auth(&creds.mattermost_token)
        .send()
        .await
        .context("mattermost unread request failed")?;
    if !resp.status().is_success() {
        let st = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("mattermost unread {}: {}", st, body));
    }
    let arr: Value = resp
        .json()
        .await
        .context("mattermost unread: invalid json")?;
    let mut total: u64 = 0;
    if let Some(items) = arr.as_array() {
        for it in items {
            if let Some(n) = it.get("msg_count").and_then(|x| x.as_u64()) {
                total = total.saturating_add(n);
            }
        }
    }
    Ok(total)
}

async fn test_mattermost_async(url: &str, token: &str) -> Result<String> {
    if url.is_empty() || token.is_empty() {
        return Err(anyhow!("url 또는 token이 비어 있습니다"));
    }
    let client = http_client()?;
    let base = trim_mattermost_url(url);
    let resp = client
        .get(format!("{}/api/v4/users/me", base))
        .bearer_auth(token)
        .send()
        .await
        .context("mattermost /users/me request failed")?;
    if !resp.status().is_success() {
        let st = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("{}: {}", st, body));
    }
    let v: Value = resp
        .json()
        .await
        .context("mattermost /users/me: invalid json")?;
    let username = v
        .get("username")
        .and_then(|x| x.as_str())
        .unwrap_or("unknown");
    Ok(username.to_string())
}

// ───────────────── Slack ─────────────────

async fn enable_dnd_slack_async(creds: &MessengerCreds) -> Result<()> {
    if creds.slack_token.is_empty() {
        return Err(anyhow!("slack token missing"));
    }
    let client = http_client()?;
    let resp = client
        .post("https://slack.com/api/dnd.setSnooze")
        .bearer_auth(&creds.slack_token)
        .form(&[("num_minutes", "60")])
        .send()
        .await
        .context("slack dnd.setSnooze request failed")?;
    let v: Value = resp.json().await.context("slack dnd.setSnooze: invalid json")?;
    if !v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
        let err = v
            .get("error")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown_error");
        return Err(anyhow!("slack dnd.setSnooze: {}", err));
    }
    // Best-effort presence away
    let _ = client
        .post("https://slack.com/api/users.setPresence")
        .bearer_auth(&creds.slack_token)
        .form(&[("presence", "away")])
        .send()
        .await;
    Ok(())
}

async fn disable_dnd_slack_async(creds: &MessengerCreds) -> Result<()> {
    if creds.slack_token.is_empty() {
        return Err(anyhow!("slack token missing"));
    }
    let client = http_client()?;
    let resp = client
        .post("https://slack.com/api/dnd.endSnooze")
        .bearer_auth(&creds.slack_token)
        .send()
        .await
        .context("slack dnd.endSnooze request failed")?;
    let v: Value = resp
        .json()
        .await
        .context("slack dnd.endSnooze: invalid json")?;
    if !v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
        let err = v
            .get("error")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown_error");
        return Err(anyhow!("slack dnd.endSnooze: {}", err));
    }
    let _ = client
        .post("https://slack.com/api/users.setPresence")
        .bearer_auth(&creds.slack_token)
        .form(&[("presence", "auto")])
        .send()
        .await;
    Ok(())
}

fn wrap_slack_emoji(name: &str) -> String {
    let trimmed = name.trim().trim_matches(':');
    if trimmed.is_empty() {
        String::new()
    } else {
        format!(":{}:", trimmed)
    }
}

async fn capture_slack_status_async(creds: &MessengerCreds) -> Option<CustomStatus> {
    if creds.slack_token.is_empty() {
        return None;
    }
    let client = http_client().ok()?;
    let resp = client
        .get("https://slack.com/api/users.profile.get")
        .bearer_auth(&creds.slack_token)
        .send()
        .await
        .ok()?;
    let v: Value = resp.json().await.ok()?;
    if !v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
        return None;
    }
    let profile = v.get("profile")?;
    let emoji = profile
        .get("status_emoji")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let text = profile
        .get("status_text")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let expires_at_unix = profile
        .get("status_expiration")
        .and_then(|x| x.as_i64());
    if emoji.is_empty() && text.is_empty() {
        return None;
    }
    Some(CustomStatus {
        emoji,
        text,
        expires_at_unix,
    })
}

async fn apply_slack_status_async(creds: &MessengerCreds) -> Result<()> {
    if creds.slack_token.is_empty() {
        return Err(anyhow!("slack token missing"));
    }
    let client = http_client()?;
    let emoji = wrap_slack_emoji(&creds.slack_status_emoji);
    let resp = client
        .post("https://slack.com/api/users.profile.set")
        .bearer_auth(&creds.slack_token)
        .json(&json!({
            "profile": {
                "status_text": creds.slack_status_text,
                "status_emoji": emoji,
                "status_expiration": 0,
            }
        }))
        .send()
        .await
        .context("slack users.profile.set request failed")?;
    let v: Value = resp
        .json()
        .await
        .context("slack users.profile.set: invalid json")?;
    if !v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
        let err = v
            .get("error")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown_error");
        return Err(anyhow!("slack users.profile.set: {}", err));
    }
    Ok(())
}

async fn clear_slack_status_async(
    creds: &MessengerCreds,
    restore: Option<&CustomStatus>,
) -> Result<()> {
    if creds.slack_token.is_empty() {
        return Err(anyhow!("slack token missing"));
    }
    let client = http_client()?;
    let (text, emoji, expiration) = match restore {
        Some(r) => (
            r.text.clone(),
            r.emoji.clone(),
            r.expires_at_unix.unwrap_or(0),
        ),
        None => (String::new(), String::new(), 0),
    };
    let resp = client
        .post("https://slack.com/api/users.profile.set")
        .bearer_auth(&creds.slack_token)
        .json(&json!({
            "profile": {
                "status_text": text,
                "status_emoji": emoji,
                "status_expiration": expiration,
            }
        }))
        .send()
        .await
        .context("slack users.profile.set (clear) request failed")?;
    let v: Value = resp
        .json()
        .await
        .context("slack users.profile.set (clear): invalid json")?;
    if !v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
        let err = v
            .get("error")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown_error");
        return Err(anyhow!("slack users.profile.set (clear): {}", err));
    }
    Ok(())
}

async fn fetch_slack_unread(creds: &MessengerCreds) -> Option<u64> {
    let client = http_client().ok()?;
    let resp = client
        .get("https://slack.com/api/users.counts")
        .bearer_auth(&creds.slack_token)
        .send()
        .await
        .ok()?;
    let v: Value = resp.json().await.ok()?;
    if !v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
        return None;
    }
    let mut total: u64 = 0;
    for key in ["channels", "groups", "ims", "mpims"] {
        if let Some(arr) = v.get(key).and_then(|x| x.as_array()) {
            for item in arr {
                if let Some(n) = item.get("mention_count").and_then(|x| x.as_u64()) {
                    total = total.saturating_add(n);
                } else if let Some(n) = item.get("unread_count").and_then(|x| x.as_u64()) {
                    total = total.saturating_add(n);
                }
            }
        }
    }
    Some(total)
}

async fn test_slack_async(token: &str) -> Result<String> {
    if token.is_empty() {
        return Err(anyhow!("token이 비어 있습니다"));
    }
    let client = http_client()?;
    let resp = client
        .get("https://slack.com/api/auth.test")
        .bearer_auth(token)
        .send()
        .await
        .context("slack auth.test request failed")?;
    let v: Value = resp.json().await.context("slack auth.test: invalid json")?;
    if !v.get("ok").and_then(|x| x.as_bool()).unwrap_or(false) {
        let err = v
            .get("error")
            .and_then(|x| x.as_str())
            .unwrap_or("unknown_error");
        return Err(anyhow!("{}", err));
    }
    let user = v.get("user").and_then(|x| x.as_str()).unwrap_or("unknown");
    let team = v.get("team").and_then(|x| x.as_str()).unwrap_or("unknown");
    Ok(format!("{}@{}", user, team))
}

// ───────────────── Combined ─────────────────

async fn snapshot_unread_async(creds: &MessengerCreds) -> UnreadSnapshot {
    let mm = if !creds.mattermost_url.is_empty() && !creds.mattermost_token.is_empty() {
        fetch_mattermost_unread(creds).await.ok()
    } else {
        None
    };
    let sl = if !creds.slack_token.is_empty() {
        fetch_slack_unread(creds).await
    } else {
        None
    };
    UnreadSnapshot {
        mattermost: mm,
        slack: sl,
    }
}
