# pip_gui - Setup & Installation Guide

## Prerequisites

- **Rust 1.70+** - [Install](https://rustup.rs/)
- **Node.js 18+** - [Install](https://nodejs.org/)
- **Tauri CLI** - `npm install -g @tauri-apps/cli@latest`

## Project Setup

### 1. Clone and Install Dependencies

```bash
# Clone project
git clone https://github.com/pro-grammer-SD/pip_gui.git
cd pip_gui/src-tauri

# Frontend dependencies
npm install

# Rust backend handled by Tauri CLI
```

### 2. Verify File Structure

Ensure all files are in place:

```bash
pip_gui/
├── src/                          # Frontend
│   ├── index.html               # Main HTML
│   ├── css/input.css            # Tailwind input
│   └── js/main.js               # App logic
│
├── src-tauri/                    # Tauri/Rust backend
│   ├── src/
│   │   ├── main.rs              # App entry
│   │   ├── state.rs             # App state
│   │   ├── models/              # Data structures
│   │   ├── commands/            # Tauri commands
│   │   └── utils/               # Helpers
│   ├── Cargo.toml
│   ├── tailwind.config.js        # Styling
│   └── tauri.conf.json
│
├── vite.config.js               # Frontend build
├── package.json                 # Node dependencies
├── SETUP.md                     # This file
└── README.md                    # README
```

### 3. Build Tailwind CSS

```bash
npx tailwindcss -i ./src-tauri/src/utils/css/input.css -o ./src-tauri/src/utils/css/output.css
```

### 4. Development Mode

### Setup

```bash
cargo install tauri-cli --version "^2.0.0" --locked
```

```bash
npm install
cargo tauri dev
```

### Build

```bash
cargo tauri build
```

This starts:

- Vite dev server (frontend hot reload)
- Tauri app with HMR
- Rust backend compiler

Generates installers for:

- Windows (MSI + NSIS)
- macOS (DMG)
- Linux (AppImage + deb)

## Platform-Specific Notes

### Windows

- Requires Microsoft Build Tools or Visual Studio Community
- Python detection checks: `C:\Python*`, `Program Files\Python*`, AppData
- Uses `.exe` for executables

### macOS

- Requires Xcode Command Line Tools
- Homebrew Python at `/opt/homebrew/bin/python3`
- System Python at `/usr/bin/python3`

### Linux

- Uses system Python at `/usr/bin/python3` or `/usr/local/bin/python3`
- Ensure `python3-dev` or `python3-devel` installed
- May need `libssl-dev`

## Features Implemented

### ✅ Python Path Selection

- Auto-detection of system Python
- Custom folder browser (no OS dialogs)
- venv detection via `pyvenv.cfg`
- Path persistence across sessions
- Explicit validation with `python --version` and `python -m pip --version`

### ✅ Virtual Environment Handling

- Auto-detects `.venv`, `venv`, `env` directories
- Scans for `pyvenv.cfg` for venv discovery
- Shows venv detection in sidebar
- Quick-switch between venvs

### ✅ Package Management

- List installed packages (via `pip list --format=json`)
- Install (latest or specific version)
- Uninstall with confirmation
- Upgrade to latest
- Downgrade to specific version
- Real-time pip output streaming via event listeners

### ✅ Project File Detection

- Detects `pyproject.toml`, `requirements.txt`, `requirements-dev.txt`
- Parses PEP 621 and classic pip requirements
- Shows dependency status: installed, missing, version mismatch
- Supports version specifiers: `==`, `>=`, `<=`, `>`, `<`

### ✅ PyPI Integration

- Searches packages via PyPI JSON API
- Fetches package metadata and release history
- Displays author, license, description, version history
- No deprecated `pip search` - uses modern API

### ✅ UI/UX

- Clean, dark Tailwind design
- Responsive layout with sidebar and main content area
- Real-time pip command output modal
- Toast notifications
- Status badges for dependencies
- Smooth tab navigation

## Architecture

### Frontend (Vanilla JS + Tailwind)

- No frameworks - pure JavaScript with Tauri IPC
- Component-based UI rendering
- Event-driven architecture
- State management via `appState` object

### Backend (Rust)

- Tauri commands as async functions
- Cross-platform file system operations
- Direct pip subprocess execution
- Safe path validation and venv detection
- PyPI API client with caching ready

### Data Flow

```bash
User Action → JavaScript Handler
    ↓
Tauri Command (IPC) → Rust Backend
    ↓
OS Operation (file, subprocess, network)
    ↓
JSON Response → Frontend State Update
    ↓
UI Re-render via DOM manipulation
```

## Troubleshooting

### "No Python installations found"

- Ensure Python is in standard locations (see Platform-Specific Notes)
- Or manually select Python using the folder browser
- Check Python executable has execution permissions (Unix)

### "pip is not available"

- Verify: `python -m pip --version` works in terminal
- May need to reinstall pip: `python -m ensurepip --upgrade`

### Venv not detected

- Ensure `pyvenv.cfg` exists in venv root
- Standard venvs created with `python -m venv` include this file
- If missing, venv is corrupted - recreate it

### Tauri build fails

- Clear `src-tauri/target`: `cargo clean`
- Ensure Rust is up to date: `rustup update`
- Check Node modules: `npm install`

## Performance Tuning

### Caching

- PyPI results cached in memory (ready to extend to disk)
- Installed packages list cached with TTL
- Project files detected once per session

### Concurrency

- All Tauri commands are async (`#[tauri::command]`)
- Multiple pip operations can run in parallel
- Network requests use tokio runtime

## Security Considerations

- All paths validated before execution
- pip runs without shell (subprocess::Command direct execution)
- No shell evaluation of user input
- Configuration stored in user config directory
- No remote code execution vectors

## Future Enhancements

- Poetry support (parsing `poetry.lock`)
- Conda environment detection
- Dependency graph visualization
- Package update notifications
- Virtual environment creation UI
- Local package index support
- Requirements freeze/export
- Dependency conflict resolution
