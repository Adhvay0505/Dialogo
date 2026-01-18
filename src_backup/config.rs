use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use dirs::config_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub use_tls: bool,
    pub accept_invalid_certs: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5222,
            use_tls: true,
            accept_invalid_certs: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountConfig {
    pub jid: String,
    pub password: String,
    pub resource: String,
    pub server: ServerConfig,
    pub auto_connect: bool,
    pub save_password: bool,
}

impl Default for AccountConfig {
    fn default() -> Self {
        Self {
            jid: String::new(),
            password: String::new(),
            resource: "xmpp-client".to_string(),
            server: ServerConfig::default(),
            auto_connect: false,
            save_password: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub accounts: Vec<AccountConfig>,
    pub default_account: Option<String>,
    pub log_level: String,
    pub theme: String,
    pub notification_enabled: bool,
    pub file_transfer_dir: PathBuf,
    pub max_file_size: u64,
    pub message_history_limit: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            default_account: None,
            log_level: "info".to_string(),
            theme: "default".to_string(),
            notification_enabled: true,
            file_transfer_dir: dirs::download_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap().join("Downloads")),
            max_file_size: 100 * 1024 * 1024, // 100MB
            message_history_limit: 1000,
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> crate::error::Result<Self> {
        let config_dir = config_dir()
            .ok_or_else(|| crate::error::XmppError::ConfigError(
                config::ConfigError::Message("Could not find config directory".to_string())
            ))?;
        
        let config_path = config_dir.join("xmpp-client");
        std::fs::create_dir_all(&config_path)?;
        
        Ok(Self {
            config_path: config_path.join("config.toml"),
        })
    }

    pub fn load_config(&self) -> crate::error::Result<AppConfig> {
        if self.config_path.exists() {
            let config_str = std::fs::read_to_string(&self.config_path)?;
            let config: AppConfig = toml::from_str(&config_str)
                .map_err(|e| crate::error::XmppError::ConfigError(
                    config::ConfigError::Foreign(Box::new(e))
                ))?;
            Ok(config)
        } else {
            Ok(AppConfig::default())
        }
    }

    pub fn save_config(&self, config: &AppConfig) -> crate::error::Result<()> {
        let config_str = toml::to_string_pretty(config)
            .map_err(|e| crate::error::XmppError::ConfigError(
                config::ConfigError::Foreign(Box::new(e))
            ))?;
        
        std::fs::write(&self.config_path, config_str)?;
        Ok(())
    }
}