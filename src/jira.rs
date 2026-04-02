use anyhow::Result;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

pub struct Client {
    http: HttpClient,
    base_url: String,
    email: String,
    token: String,
}

pub struct JiraUser {
    pub account_id: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    pub key: String,
    pub summary: String,
    pub status: String,
    pub seconds: i64,
    pub started: String,
    pub url: String,
    pub description: String,
}

#[derive(Deserialize)]
struct ApiUser {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "displayName")]
    display_name: String,
}

#[derive(Deserialize)]
struct SearchResponse {
    issues: Vec<Issue>,
    #[serde(rename = "isLast")]
    is_last: Option<bool>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(Deserialize, Clone)]
struct Issue {
    key: String,
    fields: IssueFields,
}

#[derive(Deserialize, Clone)]
struct IssueFields {
    summary: String,
    status: StatusField,
    worklog: Option<WorklogContainer>,
}

#[derive(Deserialize, Clone)]
struct StatusField {
    name: String,
}

#[derive(Deserialize, Clone)]
struct WorklogContainer {
    total: Option<u32>,
    worklogs: Vec<Worklog>,
}

#[derive(Deserialize, Clone)]
struct Worklog {
    author: WorklogAuthor,
    started: String,
    #[serde(rename = "timeSpentSeconds")]
    time_spent_seconds: i64,
    comment: Option<serde_json::Value>,
}

#[derive(Deserialize, Clone)]
struct WorklogAuthor {
    #[serde(rename = "accountId")]
    account_id: String,
}

#[derive(Deserialize)]
struct WorklogResponse {
    worklogs: Vec<Worklog>,
}

impl Client {
    pub fn new(base_url: &str, email: &str, token: &str) -> Self {
        Self {
            http: HttpClient::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            email: email.to_string(),
            token: token.to_string(),
        }
    }

    fn request(&self, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}/rest/api/3{}", self.base_url, path);
        self.http
            .get(url)
            .basic_auth(&self.email, Some(&self.token))
            .header("Accept", "application/json")
            .timeout(std::time::Duration::from_secs(15))
    }

    pub async fn get_current_user(&self) -> Result<JiraUser> {
        let user: ApiUser = self.request("/myself").send().await?.error_for_status()?.json().await?;
        Ok(JiraUser { account_id: user.account_id, display_name: user.display_name })
    }

    async fn get_issues(&self, account_id: &str) -> Result<Vec<Issue>> {
        let jql = format!(
            r#"worklogAuthor = "{account_id}" AND worklogDate >= startOfMonth(-1) ORDER BY updated DESC"#
        );
        let mut issues = Vec::new();
        let mut next_page_token: Option<String> = None;

        loop {
            let mut req = self
                .request("/search/jql")
                .query(&[
                    ("jql", jql.as_str()),
                    ("fields", "summary,status,timespent,worklog,project"),
                    ("maxResults", "100"),
                ]);

            if let Some(ref t) = next_page_token {
                req = req.query(&[("nextPageToken", t.as_str())]);
            }

            let data: SearchResponse = req.send().await?.error_for_status()?.json().await?;
            let is_last = data.is_last.unwrap_or(true);
            let token = data.next_page_token.clone();
            issues.extend(data.issues);

            if is_last || token.is_none() {
                break;
            }
            next_page_token = token;
        }

        Ok(issues)
    }

    pub async fn get_entries(&self, account_id: &str) -> Result<Vec<Entry>> {
        let issues = self.get_issues(account_id).await?;
        let mut entries = Vec::new();

        for issue in &issues {
            let worklogs = match &issue.fields.worklog {
                Some(wl) if wl.total.unwrap_or(0) > 20 => self.get_all_worklogs(&issue.key).await?,
                Some(wl) => wl.worklogs.clone(),
                None => continue,
            };

            for wl in &worklogs {
                if wl.author.account_id != account_id {
                    continue;
                }
                entries.push(Entry {
                    key: issue.key.clone(),
                    summary: issue.fields.summary.clone(),
                    status: issue.fields.status.name.clone(),
                    seconds: wl.time_spent_seconds,
                    started: wl.started[..10].to_string(),
                    url: format!("{}/browse/{}", self.base_url, issue.key),
                    description: wl.comment.as_ref().map(extract_adf_text).unwrap_or_default(),
                });
            }
        }

        Ok(entries)
    }

    async fn get_all_worklogs(&self, key: &str) -> Result<Vec<Worklog>> {
        let data: WorklogResponse = self
            .request(&format!("/issue/{key}/worklog"))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(data.worklogs)
    }
}


fn extract_adf_text(value: &serde_json::Value) -> String {
    let obj = match value.as_object() {
        Some(o) => o,
        None => return String::new(),
    };

    if obj.get("type").and_then(|t| t.as_str()) == Some("text") {
        return obj
            .get("text")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
    }

    obj.get("content")
        .and_then(|c| c.as_array())
        .map(|nodes| {
            nodes
                .iter()
                .map(extract_adf_text)
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default()
        .trim()
        .to_string()
}
