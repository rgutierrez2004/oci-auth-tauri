# OCI Auth Tauri

A cross-platform desktop application for Oracle Cloud Infrastructure authentication built with Tauri 2.0, Rust, React, Next.js, and NextUI.

## Features

- **Single Binary**: One executable that includes both CLI and GUI functionality
- **Modern UI**: Built with NextUI and React
- **Configurable Logging**: Multiple log levels and file rotation support
- **Cross-Platform**: Runs on Windows, macOS, and Linux
- **CLI Support**: Configure the application via command line
- **User Authentication**: Secure login system with profile management
- **Theme Support**: Light and dark mode with system preference detection
- **Responsive Design**: Adapts to different screen sizes and orientations

## Project Structure

```
oci-auth-tauri/
├── app/                    # Next.js application directory
│   ├── page.tsx           # Main application page
│   ├── providers.tsx      # UI providers configuration
│   └── layout.tsx         # Root layout
├── components/            # Reusable React components
├── public/               # Static assets
├── src-tauri/           # Rust backend code
│   └── src/             # Rust source files
│       ├── main.rs      # Application entry point
│       ├── lib.rs       # Core library code
│       └── config.rs    # Configuration management
├── ARCHITECTURE.md      # Detailed architecture documentation
└── README.md           # Project documentation
```

For a detailed explanation of the architecture and components, see [ARCHITECTURE.md](ARCHITECTURE.md).

## Prerequisites

### Core Requirements
- Node.js 18.x or later
- Rust 1.77.2 or later
- Cargo (latest stable version)

### System Dependencies

#### Linux

For Debian/Ubuntu-based systems:
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

For Arch Linux:
```bash
sudo pacman -Syu
sudo pacman -S --needed \
  webkit2gtk-4.1 \
  base-devel \
  curl \
  wget \
  file \
  openssl \
  appmenu-gtk-module \
  libappindicator-gtk3 \
  librsvg
```

For Fedora:
```bash
sudo dnf check-update
sudo dnf install webkit2gtk4.1-devel \
  openssl-devel \
  curl \
  wget \
  file \
  libappindicator-gtk3-devel \
  librsvg2-devel
sudo dnf group install "c-development"
```

#### macOS
1. Install Xcode from:
   - [Mac App Store](https://apps.apple.com/gb/app/xcode/id497799835?mt=12) or
   - [Apple Developer website](https://developer.apple.com/xcode/resources/)
2. Launch Xcode to complete setup
3. Install Command Line Tools:
```bash
xcode-select --install
```

#### Windows
- Windows 7 or later
- [Microsoft Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
- [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (comes pre-installed on Windows 11)

For more details on system requirements, see [Tauri Prerequisites](https://v2.tauri.app/start/).

### Troubleshooting System Dependencies

If you encounter build errors:
1. Ensure all system dependencies are installed correctly for your specific OS version
2. For Linux users: some distributions might require slightly different package names
3. For macOS users: make sure Xcode Command Line Tools are properly installed
4. For Windows users: verify WebView2 runtime is installed if you're on Windows 10 or earlier

Check the [Tauri GitHub issues](https://github.com/tauri-apps/tauri/issues) or [Discussions](https://github.com/tauri-apps/tauri/discussions) for known solutions to common problems.

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/oci-auth-tauri.git
cd oci-auth-tauri
```

2. Install dependencies:
```bash
npm install
```

## Usage

### GUI Mode
Start the application in GUI mode:
```bash
npx tauri dev
```

### CLI Mode
The application supports various CLI commands:

```bash
# Get current configuration
npx tauri dev -- -- --get-config

# Set log level (trace, debug, info, warn, error, off)
npx tauri dev -- -- --log-level debug

# Set maximum log file size in MB
npx tauri dev -- -- --log-size 20

# Set number of log files to keep
npx tauri dev -- -- --log-count 5

# Reset configuration to defaults
npx tauri dev -- -- --clear-config

# Show help
npx tauri dev -- -- --help
```

## Configuration

The application uses Tauri's Store plugin for configuration management. The config file is stored in the platform-specific app config directory:
- Windows: `%APPDATA%\com.oci-auth.dev\config.json`
- macOS: `~/Library/Application Support/com.oci-auth.dev/config.json`
- Linux: `~/.local/share/com.oci-auth.dev/config.json`

### Configuration Options

```json
{
  "config": {
    "logging": {
      "file_count": 5,    // Number of log files to keep
      "file_size_mb": 10, // Maximum size of each log file
      "level": "debug"    // trace, debug, info, warn, error, off
    }
  }
}
```

## Logging

The application uses a comprehensive logging system that writes to:
- Standard output (console)
- Rotating log files (with date-based naming)
- WebView (for frontend logs)

Log files are stored in the platform-specific app log directory with the format `oci-auth-YYYY-MM-DD.log`.

### Log Levels

1. **Trace**: Detailed debugging information
2. **Debug**: Debugging information useful during development
3. **Info**: General information about application state
4. **Warn**: Warning messages for potentially problematic situations
5. **Error**: Error messages for serious problems
6. **Off**: Disable logging

## Environment Variables

The application requires the following environment variables for authentication:

```env
OCI_CLIENT_ID=your_client_id
OCI_CLIENT_SECRET=your_client_secret
```

### Development Mode
In development mode, create a `.env` file in the folder `src-tauri` with the above variables. The application will automatically load them when running:
```bash
npx tauri dev
```

### Production Mode
For production builds, you have two options:

1. Using the launch script (recommended):
   ```bash
   # Build the application
   npx tauri build
   
   # Create a `.env` file in the root folder

   # Run using the launch script (which will use your .env file)
   ./launch.sh
   ```

2. Setting environment variables manually:
   ```bash
   # Set environment variables
   export OCI_CLIENT_ID=your_client_id
   export OCI_CLIENT_SECRET=your_client_secret
   
   # Run the application
   ./src-tauri/target/release/oci-auth-tauri
   ```

> **Security Note**: The `.env` file is not bundled with the application in production builds to protect sensitive credentials.

## Development

1. Start in development mode:
```bash
npx tauri dev
```

2. Build for production:
```bash
npx tauri build
```

## Building and Running the Application

### Building
To create a production build of the application:
```bash
npx tauri build
```

This will create platform-specific binaries in the following location:
- Windows: `src-tauri/target/release/oci-auth-tauri.exe`
- macOS: `src-tauri/target/release/oci-auth-tauri.app`
- Linux: `src-tauri/target/release/oci-auth-tauri`

### Running the Built Application

#### GUI Mode
Simply double-click the binary file or run it from the terminal:
```bash
# Linux/macOS
./src-tauri/target/release/oci-auth-tauri

# Windows
.\src-tauri\target\release\oci-auth-tauri.exe
```

#### CLI Mode
The built binary supports the same CLI commands as the development version:
```bash
# Linux/macOS
./src-tauri/target/release/oci-auth-tauri --get-config
./src-tauri/target/release/oci-auth-tauri --log-level debug

# Windows
.\src-tauri\target\release\oci-auth-tauri.exe --get-config
.\src-tauri\target\release\oci-auth-tauri.exe --log-level debug
```

## Architecture

The application follows a modern architecture:

- **Frontend**: React with Next.js and NextUI Pro
- **Backend**: Rust with Tauri 2.0
- **State Management**: Tauri Store plugin
- **Configuration**: JSON-based config with Serde serialization
- **Logging**: Multi-target logging with file rotation
- **CLI**: Integrated CLI support in the main binary

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.
