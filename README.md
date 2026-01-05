# pip_gui - Python Package Manager GUI

A modern, production-grade desktop application for managing Python packages across virtual environments. Built with Tauri (Rust), Vanilla JS, and Tailwind CSS.

## ⚠️ NOTICE

This project is still under **development**. It will **NOT WORK** now and will **give compilation errors**. As a humble request to anyone who sees this, **please fork this repo and help me in dev by creating PRs**. I really value your help and appreciate it.

## Why pip_gui?

- **No IDE required** - Manage packages without opening an IDE
- **venv-aware** - Auto-detects and switches between virtual environments
- **No embedded Python** - Works with your system Python
- **Cross-platform** - Windows, macOS, Linux
- **Fast & native** - Rust backend, minimal overhead
- **Intentional design** - Every feature is production-ready

## Quick Start

### Install

Download pre-built installers from [Releases](https://github.com/pro-grammer-SD/pip_gui/releases):

- **Windows**: `pip_gui-installer.msi`
- **macOS**: `pip_gui.dmg`
- **Linux**: `pip_gui.AppImage` or `.deb`

### First Run

1. App launches with Python selection dialog
2. Auto-detects system Python installations
3. Or manually browse to select a Python executable
4. Confirms with `python --version` and `python -m pip --version`
5. Selection persisted - no need to select again

### Main Interface

> **Sidebar**

- Selected Python version
- Detected virtual environments (quick-switch)
- Navigation tabs

> **Main Tabs**

- **Installed Packages**: Manage installed packages (upgrade, uninstall)
- **Search PyPI**: Find and install packages
- **Project Dependencies**: View `requirements.txt` and `pyproject.toml`

## Features

### Python Management

- Auto-detect system Python installations
- Custom folder browser (no OS dialogs)
- Virtual environment detection and quick-switch
- Validation with version checks
- Path persistence across sessions

### Package Operations

- List installed packages with versions
- Search PyPI for new packages
- Install specific versions
- Upgrade to latest
- Downgrade to previous version
- Uninstall with confirmation
- Real-time command output logging

### Project Support

- Detect `pyproject.toml`, `requirements.txt`, `requirements-dev.txt`
- Parse PEP 621 dependencies
- Show dependency status: installed ✓, missing ✗, version mismatch ⚠
- Version specifier support: `==`, `>=`, `<=`, `>`, `<`

### Virtual Environments

- Auto-detect `.venv`, `venv`, `env`, and custom venvs
- Parse `pyvenv.cfg` for venv identification
- Quick-switch between venvs in sidebar
- Clearly labeled venv Python selections

## Technical Stack

> **Frontend**

- Vanilla JavaScript (no frameworks)
- Tauri IPC for backend communication
- Tailwind CSS for styling
- Vite for build

> **Backend**

- Rust with Tauri 2.0
- Async command handlers
- Native subprocess execution
- PyPI JSON API integration

> **Cross-platform**

- Works with system Python only
- Executes pip via `python -m pip`
- Safe path validation
- Platform-aware file operations

## Architecture

```bash
pip_gui Application
├── Frontend (JavaScript)
│   ├── Python Selection UI
│   ├── Folder Browser
│   ├── Package Management UI
│   └── Project Dependencies View
│
└── Backend (Rust)
    ├── Python Detection & Validation
    ├── Virtual Environment Detection
    ├── Directory Listing
    ├── pip Subprocess Execution
    ├── PyPI Metadata Fetching
    └── Project File Parsing
```

## File Structure

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
│   └── tauri.conf.json
│
├── vite.config.js               # Frontend build
├── tailwind.config.js           # Styling
├── package.json                 # Node dependencies
└── README.md                    # This file
```

## Development

### Prerequisites

- Rust 1.70+
- Node.js 18+
- Tauri CLI

### Setup

```bash
npm install
npm run tauri-dev
```

### Build

```bash
npm run tauri-build
```

See [SETUP.md](./SETUP.md) for detailed instructions.

## CLI Usage

While pip_gui is a GUI app, all operations can be done from terminal with:

```bash
# Direct pip usage (why use GUI then?)
python -m pip install requests

# venv management
python -m venv myenv
source myenv/bin/activate  # or myenv\Scripts\activate on Windows
```

pip_gui just makes this faster and more visual!

## Security

- All paths validated before filesystem access
- pip executed without shell (no injection vectors)
- No remote code execution possibilities
- Config stored in user home directory
- Source code auditable and open

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Write tests for new commands
4. Submit a pull request

## License

MIT

## Support

Issues and feature requests: [GitHub Issues](https://github.com/pro-grammer-SD/pip_gui/issues)

---

**Built with** ❤️ using Tauri, Rust, and Python
