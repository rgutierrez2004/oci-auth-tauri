use tauri::Manager;
use tauri::menu::{Menu, MenuItem, Submenu, MenuId};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};
use tauri_plugin_log::{Target, TargetKind, Builder as LogBuilder};
use log::{debug, info, warn, LevelFilter, error};
use tauri::State;
use tauri_plugin_cli::CliExt;
use tauri_plugin_store::Builder as StoreBuilder;
use chrono::Local;
use std::sync::Mutex;
use oci_auth_tauri::config::{AppConfig, LogLevel};
use dotenvy::dotenv;
mod config;
mod auth;

use auth::{complete_auth, initiate_auth};

#[derive(Default)]
pub struct ConfigState(Mutex<AppConfig>);

#[tauri::command]
fn update_log_level(app_handle: tauri::AppHandle, state: tauri::State<ConfigState>, new_level: String) -> Result<(), String> {
    let mut config = state.0.lock().map_err(|e| e.to_string())?;
    config.set_log_level(&app_handle, &new_level).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_log_level(state: tauri::State<ConfigState>) -> Result<String, String> {
    let config = state.0.lock().map_err(|e| e.to_string())?;
    Ok(config.logging.level.to_string())
}

#[tauri::command]
fn get_current_config(config_state: State<ConfigState>) -> Result<AppConfig, String> {
    let config = config_state.0.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

// Handle CLI commands and return Ok(true) if a command was handled
fn handle_cli_commands(app: &tauri::App) -> Result<bool, Box<dyn std::error::Error>> {
    let cli = app.cli();
    
    // Get the matches from CLI
    let matches = cli.matches()?;

    // Special handling for help - it might have a pre-filled value
    if matches.args.contains_key("help") {
        println!("{}", HELP_TEXT);
        return Ok(true);
    }

    // Check if any of our specific arguments were actually provided (occurrences > 0)
    let our_args = matches.args.iter().any(|(k, v)| {
        let is_ours = matches!(k.as_str(), 
            "get-config" | "log-level" | "log-size" | 
            "log-count" | "clear-config" | "help");
        let was_provided = v.occurrences > 0;
        //println!("  Checking arg '{}': is_ours = {}, was_provided = {}", k, is_ours, was_provided);
        is_ours && was_provided
    });

    // If none of our arguments were provided, don't handle as CLI command
    if !our_args {
        return Ok(false);
    }

    // Found provided arguments, handling CLI command
    let app_handle = app.handle();
    let mut config = AppConfig::load(&app_handle)?;

    // Handle each CLI command
    if matches.args.get("get-config").map(|v| v.occurrences > 0).unwrap_or(false) {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let log_dir = app.path().app_log_dir().unwrap_or_default();
        let log_path = log_dir.join(format!("oci-auth-{}", today));
        
        println!("Current configuration:");
        println!("Log filename: {}", log_path.display());
        println!("Store plugin config: {}/config.json", app.path().app_data_dir().unwrap().display());
        println!("Log level: {}", config.logging.level);
        println!("Max log file size: {}MB", config.logging.file_size_mb);
        println!("Number of log files: {}", config.logging.file_count);
        return Ok(true);
    }

    if let Some(level) = matches.args.get("log-level") {
        if level.occurrences > 0 {
            if let Some(value) = level.value.as_str() {
                config.set_log_level(&app_handle, value)?;
                println!("Log level set to: {}", value);
                return Ok(true);
            }
        }
    }

    if let Some(size) = matches.args.get("log-size") {
        if size.occurrences > 0 {
            if let Some(value) = size.value.as_str() {
                if let Ok(size_mb) = value.parse::<u64>() {
                    if size_mb >= 1 {
                        config.logging.file_size_mb = size_mb;
                        config.save(&app_handle)?;
                        println!("Log file size set to: {}MB", size_mb);
                        return Ok(true);
                    }
                }
                println!("Invalid log size value. Must be a number >= 1");
                return Ok(true);
            }
        }
    }

    if let Some(count) = matches.args.get("log-count") {
        if count.occurrences > 0 {
            if let Some(value) = count.value.as_str() {
                if let Ok(file_count) = value.parse::<u32>() {
                    if file_count >= 1 {
                        config.logging.file_count = file_count;
                        config.save(&app_handle)?;
                        println!("Number of log files set to: {}", file_count);
                        return Ok(true);
                    }
                }
                println!("Invalid log count value. Must be a number >= 1");
                return Ok(true);
            }
        }
    }

    if matches.args.get("clear-config").map(|v| v.occurrences > 0).unwrap_or(false) {
        config = AppConfig::default();
        config.save(&app_handle)?;
        println!("Configuration reset to default values");
        return Ok(true);
    }

    Ok(false)
}

fn main() {
    // Load .env file only in development mode
    if cfg!(debug_assertions) {
        match dotenv() {
            Ok(_) => println!("Development mode: Loaded .env file"),
            Err(e) => println!("Warning: Could not load .env file: {}", e),
        }
    }

    // Get current date for log filename
    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_filename = format!("oci-auth-{}", today);

    // Set environment variable to suppress Mesa/OpenGL warnings
    std::env::set_var("LIBGL_ALWAYS_SOFTWARE", "1");

    // Check if required environment variables are set
    let required_vars = ["OCI_CLIENT_ID", "OCI_CLIENT_SECRET"];
    for var in required_vars.iter() {
        if std::env::var(var).is_err() {
            eprintln!("Error: Required environment variable {} is not set", var);
            if cfg!(debug_assertions) {
                eprintln!("In development mode, make sure these are set in your .env file");
            } else {
                eprintln!("In release mode, make sure to set these environment variables in your system");
            }
            std::process::exit(1);
        }
    }

    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_cli::init())
        .plugin(StoreBuilder::default().build())
        .plugin(
            LogBuilder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: Some(log_filename.clone()) }),
                    Target::new(TargetKind::Webview),
                ])
                .level(LevelFilter::Debug)  // Start with Debug level, will be updated in setup
                .build(),
        )
        .setup(move |app| {
            // Handle CLI commands first
            let cli_result = handle_cli_commands(app);
            match cli_result {
                Ok(true) => {
                    // CLI command was handled, exit immediately
                    std::process::exit(0);
                }
                Ok(false) => {
                    // No CLI command, continue with UI setup
                    info!("Starting in UI mode");
                    // Only print these messages in UI mode
                    println!("Log filename: {}", log_filename);
                    println!("Store plugin config will be created at: {}/config.json", app.path().app_data_dir().unwrap().display());
                }
                Err(e) => {
                    error!("CLI command error: {}", e);
                    std::process::exit(1);
                }
            }

            // Load or create config
            let config = AppConfig::load(&app.handle()).unwrap_or_else(|e| {
                eprintln!("Failed to load config: {}", e);
                AppConfig::default()
            });

            // Store the config in app state
            app.manage(ConfigState(Mutex::new(config.clone())));

            // Convert the log level from the config
            let log_level = match config.logging.level {
                LogLevel::Trace => LevelFilter::Trace,
                LogLevel::Debug => LevelFilter::Debug,
                LogLevel::Info => LevelFilter::Info,
                LogLevel::Warn => LevelFilter::Warn,
                LogLevel::Error => LevelFilter::Error,
                LogLevel::Off => LevelFilter::Off,
            };

            log::set_max_level(log_level);

            if let Some(window) = app.get_webview_window("main") {
                let handle_for_menu = app.handle().clone();
                let quit_item = MenuItem::with_id(&handle_for_menu, MenuId::from("quit"), "Quit", true, None::<&str>)?;
                let about_item = MenuItem::with_id(&handle_for_menu, MenuId::from("about"), "About", true, None::<&str>)?;

                // Create submenus
                let file = Submenu::with_items(
                    &handle_for_menu,
                    "File",
                    true,
                    &[&quit_item]
                )?;

                let help = Submenu::with_items(
                    &handle_for_menu,
                    "Help",
                    true,
                    &[&about_item]
                )?;

                // Create the menu
                let menu = Menu::with_items(
                    &handle_for_menu,
                    &[&file, &help]
                )?;

                window.set_menu(menu)?;

                // Handle menu events
                let app_handle_clone = app.handle().clone();
                window.on_menu_event(move |_window, event| {
                    debug!("Menu event received: {}", event.id().0);
                    
                    match event.id().0.as_str() {
                        "quit" => {
                            debug!("Processing quit menu action");
                            info!("Application exit requested via menu");
                            app_handle_clone.exit(0);
                        }
                        "about" => {
                            debug!("Processing about menu action");
                            info!("About dialog opened");
                            
                            let window = app_handle_clone.get_webview_window("main").unwrap();
                            window.dialog()
                                .message("OCI Auth Tauri\nVersion 1.0.0\n\nA Tauri authentication app for Oracle Cloud Infrastructure.\n\n 2025 OCI Auth Team")
                                .title("About OCI Auth Tauri")
                                .buttons(MessageDialogButtons::Ok)
                                .show(|_| {
                                    debug!("About dialog shown to user");
                                });
                        }
                        _ => {
                            debug!("Received unknown menu action: {}", event.id().0);
                            warn!("Unknown menu item clicked: {}", event.id().0);
                        }
                    }
                });
            } else {
                app.handle().exit(1);
                return Ok(());
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            update_log_level,
            get_log_level,
            get_current_config,
            initiate_auth,
            complete_auth
        ]);

    builder.run(tauri::generate_context!())
        .expect("error while running tauri application");
}

const HELP_TEXT: &str = "\
OCI Auth Tauri

USAGE:
    oci-auth-tauri [OPTIONS]

OPTIONS:
    -h, --help                  Print help information
    --get-config               Display current configuration
    --log-level <LEVEL>        Set log level (trace, debug, info, warn, error, off)
    --log-size <SIZE>          Set maximum log file size in MB (minimum 1)
    --log-count <COUNT>        Set number of log files to keep (minimum 1)
    --clear-config            Reset configuration to default values

EXAMPLES:
    # Show current configuration
    oci-auth-tauri --get-config

    # Set log level to debug
    oci-auth-tauri --log-level debug

    # Set maximum log file size to 10MB
    oci-auth-tauri --log-size 10

    # Set number of log files to keep to 5
    oci-auth-tauri --log-count 5

    # Reset configuration to defaults
    oci-auth-tauri --clear-config
";
