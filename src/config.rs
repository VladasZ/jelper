use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const KEYRING_SERVICE: &str = "jelper";
const KEYRING_KEY: &str = "api_token";

pub struct Config {
    pub jira_url: String,
    pub jira_email: String,
    pub token: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jira_url: String::new(),
            jira_email: String::new(),
            token: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct FileConfig {
    jira_url: String,
    jira_email: String,
}

fn config_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("jelper")
        .join("config.json")
}

pub fn load(url: Option<&str>, email: Option<&str>, token: Option<&str>) -> Config {
    let mut cfg = Config::default();

    let path = config_path();
    if path.exists() {
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(fc) = serde_json::from_str::<FileConfig>(&text) {
                cfg.jira_url = fc.jira_url;
                cfg.jira_email = fc.jira_email;
            }
        }
    }

    if let Some(u) = url {
        cfg.jira_url = u.to_string();
    }
    if let Some(e) = email {
        cfg.jira_email = e.to_string();
    }

    if let Some(t) = token {
        cfg.token = t.to_string();
    } else {
        cfg.token = keyring::Entry::new(KEYRING_SERVICE, KEYRING_KEY)
            .ok()
            .and_then(|e| e.get_password().ok())
            .unwrap_or_default();
    }

    cfg
}

fn save_to_disk(url: &str, email: &str, token: &str) -> Result<()> {
    let path = config_path();
    std::fs::create_dir_all(path.parent().unwrap())?;
    let content = serde_json::to_string_pretty(&FileConfig {
        jira_url: url.to_string(),
        jira_email: email.to_string(),
    })?;
    std::fs::write(&path, content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    }

    keyring::Entry::new(KEYRING_SERVICE, KEYRING_KEY)
        .context("failed to create keyring entry")?
        .set_password(token)
        .context("failed to save API token to keyring")?;

    Ok(())
}

pub async fn setup() -> Result<Config> {
    use dialoguer::{Input, Password};

    let existing = load(None, None, None);

    println!("\nJelper setup — configure your Jira credentials\n");

    let url: String = Input::new()
        .with_prompt("Jira URL")
        .default(if existing.jira_url.is_empty() {
            "https://your-org.atlassian.net".to_string()
        } else {
            existing.jira_url.clone()
        })
        .interact_text()?;

    let email: String = Input::new()
        .with_prompt("Jira email")
        .default(existing.jira_email.clone())
        .allow_empty(true)
        .interact_text()?;

    println!("\nOpening browser to create an API token...");
    let _ = open::that("https://id.atlassian.com/manage-profile/security/api-tokens");

    let token: String = Password::new()
        .with_prompt("Paste your API token")
        .interact()?;

    print!("\nVerifying credentials... ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let client = crate::jira::Client::new(&url, &email, &token);
    let user = client.get_current_user().await.map_err(|e| {
        eprintln!("Failed ({e})");
        e
    })?;

    println!("OK — logged in as {}", user.display_name);

    save_to_disk(&url, &email, &token)?;
    println!("Configuration saved.\n");

    Ok(Config { jira_url: url, jira_email: email, token })
}
