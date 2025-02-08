# OCI Auth Tauri - Architecture Documentation

## Overview

OCI Auth Tauri is a modern desktop application built using Tauri 2.0, combining a Rust backend with a React/Next.js frontend. The application is designed as a single binary that provides both GUI and CLI interfaces, making it versatile for both interactive and automated use.

## Technology Stack

### Frontend
- **Next.js**: React framework for the user interface
- **NextUI**: UI component library for modern design
- **TypeScript**: For type-safe JavaScript development
- **TailwindCSS**: For utility-first CSS styling

### Backend
- **Tauri 2.0**: Core framework for desktop application
- **Rust**: Systems programming language for the backend
- **tauri-plugin-log**: Comprehensive logging system
- **tauri-plugin-store**: Configuration management
- **tauri-plugin-dialog**: Native dialog interactions
- **tauri-plugin-cli**: Command-line interface support

## Core Architecture Components

### 1. Single Binary Design
The application is built as a single binary that can operate in two modes:
- **GUI Mode**: When launched without CLI arguments
- **CLI Mode**: When launched with command-line arguments

### 2. Configuration Management
Uses Tauri's Store plugin for persistent configuration:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub file_size_mb: u64,
    pub file_count: u32,
}

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
```

### 3. CLI Integration
Integrated CLI support using Tauri's CLI plugin:

```rust
fn handle_cli_commands(app: &tauri::App) -> Result<bool, Box<dyn std::error::Error>> {
    let cli = app.cli();
    let matches = cli.matches()?;
    
    // Handle various CLI commands (--log-level, --log-size, etc.)
    if matches.args.get("log-level").map(|v| v.occurrences > 0).unwrap_or(false) {
        // Handle log level setting
    }
    
    // Return true if CLI command was handled
    Ok(false)
}
```

### 4. Logging Architecture
Multi-target logging system using Tauri's Log plugin:

```rust
LogBuilder::new()
    .targets([
        Target::new(TargetKind::Stdout),
        Target::new(TargetKind::LogDir { file_name: Some(log_filename) }),
        Target::new(TargetKind::Webview),
    ])
    .level(LevelFilter::Debug)
    .build()
```

Features:
- Date-based log files
- Configurable file rotation
- Multiple output targets
- Synchronized frontend/backend logging

### 5. Frontend-Backend Communication
Uses Tauri's IPC system for seamless communication:

```rust
#[tauri::command]
fn get_current_config(config_state: State<ConfigState>) -> Result<AppConfig, String> {
    let config = config_state.0.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}
```

```typescript
// Frontend
const config = await invoke('get_current_config');
```

### 6. State Management
- **Backend State**: Managed through Tauri's state system using thread-safe Mutex
- **Configuration State**: Persisted using Tauri's Store plugin
- **Frontend State**: React state with hooks for UI updates

### 7. Environment Variables Management

The application uses environment variables for secure credential management:

```rust
// Environment variables required for authentication
const REQUIRED_VARS: &[&str] = &["OCI_CLIENT_ID", "OCI_CLIENT_SECRET"];

// Development mode: Load from .env file
if cfg!(debug_assertions) {
    dotenv().ok();
}

// Check for required variables in both modes
for var in REQUIRED_VARS {
    std::env::var(var)?;
}
```

#### Development Mode
- Uses the `dotenv` crate to load variables from `.env` file
- Provides immediate feedback if variables are missing
- Supports hot-reloading of environment changes

#### Production Mode
- Environment variables must be set in the system
- Launch script (`launch.sh`) provided for convenient deployment
- Variables are loaded only for the application process
- No sensitive data bundled with the application

#### Security Considerations
- Credentials are never bundled with the application
- Environment variables are process-local
- `.env` file is git-ignored and not included in builds
- Clear error messages guide proper configuration

## Data Flow

### 1. Configuration Flow
```
CLI/UI Input → Rust Backend → Store Plugin → Config File
                    ↓
              Update State
                    ↓
         Notify Frontend (if GUI mode)
```

### 2. Logging Flow
```
Frontend Log → Tauri Log Plugin → Rust Backend → Multiple Targets
     ↓                                               ↓
React Component                              - Console Output
                                            - Log Files
                                            - WebView Console
```

### 3. Command Flow
```
CLI Arguments → CLI Plugin → Command Handler → Config Update
                                  ↓
                            Exit or Launch UI
```

## File Organization

```
oci-auth-tauri/
├── app/                    # Next.js application directory
│   ├── page.tsx           # Main application page with UI components
│   ├── providers.tsx      # NextUI and theme providers setup
│   └── layout.tsx         # Root layout component
├── components/            # Reusable React components
├── public/               # Static assets
├── src-tauri/           # Rust backend code
│   ├── src/             # Rust source files
│   │   ├── main.rs      # Application entry point and window management
│   │   ├── lib.rs       # Core library functionality and exports
│   │   ├── config.rs    # Configuration management and types
│   │   └── bin/         # Binary-specific code
│   ├── Cargo.toml       # Rust dependencies
│   └── tauri.conf.json  # Tauri configuration
├── .next/               # Next.js build output
├── dist/                # Production build output
├── node_modules/        # Node.js dependencies
├── ARCHITECTURE.md      # Architecture documentation
├── README.md           # Project documentation
├── package.json        # Node.js project configuration
├── tailwind.config.ts  # Tailwind CSS configuration
└── tsconfig.json       # TypeScript configuration
```

## UI Components

### 1. Navigation
- Responsive navbar with theme toggle
- Dynamic menu items based on authentication state
- Smooth scrolling navigation

### 2. Authentication
- Login modal with username input
- Secure authentication state management
- Sign in/Sign out functionality

### 3. User Profile
- JSON data display with syntax highlighting
- Responsive card layout
- Dark/light theme support

### 4. Content Sections
- Home: Main landing section
- Features: Product capabilities
- Pricing: Service tiers
- About: Project information

## State Management

### 1. Authentication State
```typescript
const [isAuthenticated, setIsAuthenticated] = useState(false);
const [username, setUsername] = useState("");
```

### 2. UI State
```typescript
const [activeSection, setActiveSection] = useState("home");
const [isLoginOpen, setIsLoginOpen] = useState(false);
const [theme, setTheme] = useTheme();
```

## Security Considerations

1. **Configuration Storage**
   - Uses platform-specific secure storage locations
   - Configuration files have appropriate permissions

2. **Logging**
   - Sensitive data is not logged
   - Log files are stored in user-specific directories
   - Log rotation prevents disk space issues

3. **CLI Commands**
   - Input validation for all CLI arguments
   - Safe handling of file operations
   - Error handling for all operations

## Error Handling

1. **Backend**
   - Comprehensive error types
   - Error propagation to frontend
   - Graceful fallbacks to defaults

2. **Frontend**
   - Error boundaries for component failures
   - User-friendly error messages
   - Automatic error logging

## Future Considerations

1. **Extensibility**
   - Plugin system for additional features
   - Custom log targets
   - Additional CLI commands

2. **Performance**
   - Lazy loading of components
   - Efficient log rotation
   - Optimized state updates

3. **Testing**
   - Unit tests for Rust backend
   - Integration tests for CLI
   - E2E tests for UI
