use crate::Node;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AgentConfig {
    pub mode: String,
    pub endpoint: String,
    pub model: String,
    pub command: Vec<String>,
    pub api_key: Option<String>,
}
impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            mode: "builtin".into(),
            endpoint: "https://api.openai.com/v1/chat/completions".into(),
            model: String::new(),
            command: vec![],
            api_key: None,
        }
    }
}
#[derive(Serialize)]
pub struct AgentStatus {
    pub mode: String,
    pub endpoint: String,
    pub model: String,
    pub command: Vec<String>,
    pub key_present: bool,
    pub configured: bool,
    pub jobs_directory: String,
}
impl AgentConfig {
    pub fn configured(&self) -> bool {
        match self.mode.as_str() {
            "builtin" => !self.endpoint.is_empty() && !self.model.is_empty(),
            "command" | "hermes" => !self.command.is_empty(),
            "inbox" => true,
            _ => false,
        }
    }
    fn validate(&self) -> Result<()> {
        ensure!(
            matches!(
                self.mode.as_str(),
                "builtin" | "command" | "hermes" | "inbox"
            ),
            "choose a supported agent mode"
        );
        if self.mode == "builtin" {
            let u = super::fetch::url(&self.endpoint)?;
            ensure!(
                u.scheme() == "https"
                    || matches!(u.host_str(), Some("127.0.0.1" | "localhost" | "[::1]")),
                "use HTTPS for a remote LLM endpoint"
            );
            ensure!(
                !self.model.trim().is_empty() && self.model.len() < 200,
                "enter an audio-capable model name"
            );
        }
        if self.mode == "command" || self.mode == "hermes" {
            ensure!(
                !self.command.is_empty()
                    && self.command.len() < 50
                    && self.command.iter().all(|a| a.len() < 4096)
                    && !self.command[0].trim().is_empty(),
                "enter the agent executable and arguments"
            );
        }
        Ok(())
    }
}
impl Node {
    pub fn agent_config(&self) -> Result<AgentConfig> {
        if let Ok(raw) = std::env::var("HOPPER_AD_AGENT")
            .or_else(|_| std::fs::read_to_string(self.0.dir.join("agent-command.json")))
        {
            return Ok(AgentConfig {
                mode: "command".into(),
                command: serde_json::from_str(&raw)
                    .context("invalid external agent command configuration")?,
                ..Default::default()
            });
        }
        let path = self.0.dir.join("agent.json");
        if !path.exists() {
            return Ok(AgentConfig::default());
        }
        Ok(serde_json::from_slice(&std::fs::read(path)?)?)
    }
    pub fn agent_status(&self) -> Result<AgentStatus> {
        let c = self.agent_config()?;
        Ok(AgentStatus {
            configured: !cfg!(target_os = "ios") && c.configured(),
            key_present: c.api_key.as_ref().is_some_and(|v| !v.is_empty())
                || std::env::var_os("HOPPER_LLM_API_KEY").is_some(),
            mode: c.mode,
            endpoint: c.endpoint,
            model: c.model,
            command: c.command,
            jobs_directory: self.0.dir.join("jobs").to_string_lossy().into(),
        })
    }
    pub fn set_agent_config(&self, mut config: AgentConfig) -> Result<AgentStatus> {
        ensure!(
            !cfg!(target_os = "ios"),
            "configure the processing agent on its desktop"
        );
        ensure!(
            std::env::var_os("HOPPER_AD_AGENT").is_none()
                && !self.0.dir.join("agent-command.json").exists(),
            "remove the desktop's legacy agent command override before changing agent settings"
        );
        config.validate()?;
        let old = self.agent_config()?;
        // Omission retains the key only for the same endpoint. An explicit empty string clears it.
        if config.api_key.is_none() && config.endpoint == old.endpoint {
            config.api_key = old.api_key;
        }
        let temp = tempfile::NamedTempFile::new_in(&self.0.dir)?;
        std::fs::write(temp.path(), serde_json::to_vec_pretty(&config)?)?;
        temp.as_file().sync_all()?;
        temp.persist(self.0.dir.join("agent.json"))?;
        self.0.podcast_wake.notify_one();
        self.agent_status()
    }
}
