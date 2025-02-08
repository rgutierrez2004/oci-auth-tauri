use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::AppHandle;
use tauri::Manager;
use tauri_plugin_store::StoreBuilder;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[serde(alias = "Trace")]
    Trace,
    #[serde(alias = "Debug")]
    Debug,
    #[serde(alias = "Info")]
    Info,
    #[serde(alias = "Warn")]
    Warn,
    #[serde(alias = "Error")]
    Error,
    #[serde(alias = "Off")]
    Off,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::Off => write!(f, "off"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub file_size_mb: u64,
    pub file_count: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            file_size_mb: 10,
            file_count: 5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub logging: LoggingConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
        }
    }
}

#[allow(dead_code)]
impl AppConfig {
    pub fn load(app_handle: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let app_config_dir = get_app_config_dir(app_handle)?;
        let store_path = app_config_dir.join("config.json");
        
        let store = StoreBuilder::new(app_handle, store_path).build();
        
        if let Ok(store) = store {
            if let Some(config) = store.get("config") {
                return Ok(serde_json::from_value(config)?);
            }
        }
        
        // If no config exists or there was an error, return default
        let default_config = AppConfig::default();
        Ok(default_config)
    }

    pub fn save(&self, app_handle: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
        let app_config_dir = get_app_config_dir(app_handle)?;
        let store_path = app_config_dir.join("config.json");
        
        let store = StoreBuilder::new(app_handle, store_path).build()?;
        
        let value = serde_json::to_value(self)?;
        store.set("config", value);
        store.save()?;
        
        Ok(())
    }

    pub fn set_log_level(&mut self, app_handle: &AppHandle, level: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.logging.level = match level.to_lowercase().as_str() {
            "trace" => LogLevel::Trace,
            "debug" => LogLevel::Debug,
            "info" => LogLevel::Info,
            "warn" => LogLevel::Warn,
            "error" => LogLevel::Error,
            "off" => LogLevel::Off,
            _ => return Err("Invalid log level".into()),
        };
        
        self.save(app_handle)?;
        Ok(())
    }

    pub fn set_log_file_size(&mut self, app_handle: &AppHandle, size_mb: u64) -> Result<(), Box<dyn std::error::Error>> {
        if size_mb == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Log file size must be greater than 0 MB",
            )));
        }
        self.logging.file_size_mb = size_mb;
        self.save(app_handle)
    }

    pub fn set_log_file_count(&mut self, app_handle: &AppHandle, count: u32) -> Result<(), Box<dyn std::error::Error>> {
        if count == 0 {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Log file count must be greater than 0",
            )));
        }
        self.logging.file_count = count;
        self.save(app_handle)
    }
}

#[allow(dead_code)]
fn get_app_config_dir(app_handle: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_dir = app_handle
        .path()
        .app_data_dir()?;
    
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir)
}
