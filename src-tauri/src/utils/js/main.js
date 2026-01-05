import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

let appState = {
    selectedPython: null,
    selectedPythonForBrowser: null,
    currentTab: 'installed',
};

// Initialize on load
window.addEventListener('DOMContentLoaded', async () => {
    await initializeApp();
});

async function initializeApp() {
    try {
        // Check for existing Python selection
        const selectedPython = await invoke('get_selected_python');

        if (selectedPython) {
            appState.selectedPython = selectedPython;
            showMainApp();
        } else {
            // Show Python selector
            await showPythonSelector();
        }
    } catch (err) {
        console.error('Init error:', err);
        showToast('Initialization failed: ' + err, 'error');
    }
}

async function showPythonSelector() {
    const selector = document.getElementById('pythonSelector');
    selector.classList.remove('hidden');

    try {
        // Auto-detect Python
        const detections = await invoke('detect_python_installations');
        renderDetectedPython(detections);

        // Initialize browser at home
        const home = await invoke('get_home_directory');
        await browsePath(home);
    } catch (err) {
        showToast('Failed to detect Python: ' + err, 'error');
    }

    // Event listeners
    document.getElementById('confirmPythonBtn').addEventListener('click', confirmPythonSelection);

    // Tab navigation
    document.querySelectorAll('.nav-item').forEach(btn => {
        btn.addEventListener('click', switchTab);
    });

    // Change Python button
    document.getElementById('changePythonBtn').addEventListener('click', () => {
        document.getElementById('mainApp').classList.add('hidden');
        document.getElementById('pythonSelector').classList.remove('hidden');
    });

    // Refresh installed packages
    document.getElementById('refreshInstalledBtn').addEventListener('click', loadInstalledPackages);

    // Search
    document.getElementById('searchBtn').addEventListener('click', searchPackages);
    document.getElementById('searchInput').addEventListener('keypress', (e) => {
        if (e.key === 'Enter') searchPackages();
    });

    // Close pip log modal
    document.getElementById('closePipLogBtn').addEventListener('click', () => {
        document.getElementById('pipLogModal').classList.add('hidden');
    });

    // Listen to pip logs
    await listen('pip-log', (event) => {
        const logContent = document.getElementById('pipLogContent');
        const line = document.createElement('div');
        line.textContent = event.payload;
        logContent.appendChild(line);
        logContent.scrollTop = logContent.scrollHeight;
    });
}

function renderDetectedPython(detections) {
    const list = document.getElementById('detectedList');
    list.innerHTML = '';

    for (const python of detections) {
        const div = document.createElement('div');
        div.className = 'p-3 bg-slate-800 border border-slate-700 rounded-lg cursor-pointer hover:bg-slate-700/50 transition-colors';
        div.innerHTML = `
            <div class="flex items-center justify-between">
                <div>
                    <p class="font-mono text-sm text-slate-200">${python.path}</p>
                    <p class="text-xs text-slate-400 mt-1">Python ${python.version} • ${python.is_venv ? '(venv)' : '(system)'}</p>
                </div>
                <input type="radio" name="pythonSelection" value="${python.path}" class="w-4 h-4">
            </div>
        `;
        div.addEventListener('click', () => {
            document.querySelector(`input[value="${python.path}"]`).checked = true;
            appState.selectedPythonForBrowser = python.path;
            document.getElementById('confirmPythonBtn').disabled = false;
        });
        list.appendChild(div);
    }
}

async function browsePath(path) {
    try {
        const entries = await invoke('list_directory', { path });
        renderBrowserEntries(entries, path);
    } catch (err) {
        showToast('Cannot browse: ' + err, 'error');
    }
}

function renderBrowserEntries(entries, currentPath) {
    const content = document.getElementById('browserContent');
    content.innerHTML = '';

    // Add parent directory
    const parentPath = currentPath.substring(0, currentPath.lastIndexOf('/'));
    if (parentPath !== currentPath) {
        const parentDiv = document.createElement('div');
        parentDiv.className = 'p-2 hover:bg-slate-700 rounded cursor-pointer text-slate-300 text-sm';
        parentDiv.textContent = '📁 ..';
        parentDiv.addEventListener('click', () => browsePath(parentPath));
        content.appendChild(parentDiv);
    }

    for (const entry of entries) {
        const div = document.createElement('div');
        div.className = 'p-2 hover:bg-slate-700 rounded cursor-pointer transition-colors';

        if (entry.is_python) {
            div.className += ' bg-green-900/20 border border-green-700/30';
            div.innerHTML = `
                <div class="flex items-center justify-between">
                    <span class="text-sm text-green-400">🐍 ${entry.name}</span>
                    <input type="radio" name="pythonSelection" value="${entry.path}" class="w-4 h-4">
                </div>
            `;
            div.querySelector('input').addEventListener('change', () => {
                appState.selectedPythonForBrowser = entry.path;
                document.getElementById('confirmPythonBtn').disabled = false;
            });
        } else if (entry.is_dir) {
            div.className += ' text-slate-300 text-sm';
            div.innerHTML = `<span>${entry.is_venv ? '⚙️ ' : '📁 '}${entry.name}</span>`;
            div.addEventListener('click', () => browsePath(entry.path));
        } else {
            return; // Skip non-directories
        }

        content.appendChild(div);
    }
}

async function confirmPythonSelection() {
    if (!appState.selectedPythonForBrowser) {
        showToast('No Python selected', 'error');
        return;
    }

    try {
        const validated = await invoke('validate_python_path', { path: appState.selectedPythonForBrowser });
        const selected = await invoke('select_python', { path: appState.selectedPythonForBrowser });

        appState.selectedPython = selected;
        document.getElementById('pythonSelector').classList.add('hidden');
        showMainApp();
    } catch (err) {
        showToast('Invalid Python: ' + err, 'error');
    }
}

function showMainApp() {
    document.getElementById('mainApp').classList.remove('hidden');
    document.getElementById('pythonSelector').classList.add('hidden');
    updatePythonLabel();
    switchTab({ target: { dataset: { tab: 'installed' } } });
    loadInstalledPackages();
    detectProjectDependencies();
    detectVenvs();
}

function updatePythonLabel() {
    const label = document.getElementById('selectedPythonLabel');
    if (appState.selectedPython) {
        label.textContent = `${appState.selectedPython.version}`;
    }
}

function switchTab(event) {
    const tab = event.target.dataset.tab;
    appState.currentTab = tab;

    // Update UI
    document.querySelectorAll('.tab-content').forEach(el => el.classList.add('hidden'));
    document.getElementById(tab + 'Tab').classList.remove('hidden');

    document.querySelectorAll('.nav-item').forEach(btn => {
        btn.classList.remove('bg-blue-600/20', 'text-blue-400', 'border', 'border-blue-500/30');
        btn.classList.add('text-slate-300', 'hover:bg-slate-800');
    });
    event.target.classList.add('bg-blue-600/20', 'text-blue-400', 'border', 'border-blue-500/30');
    event.target.classList.remove('text-slate-300', 'hover:bg-slate-800');

    if (tab === 'installed') {
        loadInstalledPackages();
    }
}

async function loadInstalledPackages() {
    if (!appState.selectedPython) return;

    const list = document.getElementById('installedList');
    list.innerHTML = '<div class="text-center py-8"><p class="text-slate-400">Loading packages...</p></div>';

    try {
        const packages = await invoke('list_installed_packages', { pythonPath: appState.selectedPython.path });

        list.innerHTML = '';
        if (packages.length === 0) {
            list.innerHTML = '<p class="text-slate-400">No packages installed</p>';
            return;
        }

        for (const pkg of packages) {
            list.appendChild(createPackageCard(pkg, false));
        }
    } catch (err) {
        list.innerHTML = '<p class="text-red-400">Error loading packages: ' + err + '</p>';
    }
}

async function searchPackages() {
    const query = document.getElementById('searchInput').value.trim();
    if (!query) {
        showToast('Enter a package name', 'error');
        return;
    }

    const results = document.getElementById('searchResults');
    results.innerHTML = '<div class="text-center py-8"><p class="text-slate-400">Searching...</p></div>';

    try {
        const packages = await invoke('search_pypi', { query });
        const resultsDiv = document.getElementById('searchResults');
        resultsDiv.innerHTML = '';

        if (packages.length === 0) {
            resultsDiv.innerHTML = '<p class="text-slate-400">No packages found</p>';
            return;
        }

        for (const pkg of packages) {
            const card = createSearchResultCard(pkg);
            resultsDiv.appendChild(card);
        }
    } catch (err) {
        results.innerHTML = '<p class="text-red-400">Search failed: ' + err + '</p>';
    }
}

async function detectProjectDependencies() {
    try {
        const home = await invoke('get_home_directory');
        document.getElementById('projectPath').textContent = home;

        const deps = await invoke('parse_requirements', { projectPath: home, pythonPath: appState.selectedPython.path });
        const list = document.getElementById('dependenciesList');

        if (deps.length === 0) {
            list.innerHTML = '<p class="text-slate-400">No project files detected</p>';
            return;
        }

        list.innerHTML = '';
        for (const dep of deps) {
            list.appendChild(createDependencyCard(dep));
        }
    } catch (err) {
        console.error('Error loading dependencies:', err);
    }
}

async function detectVenvs() {
    try {
        const home = await invoke('get_home_directory');
        const venvs = await invoke('detect_venvs', { projectPath: home });
        const list = document.getElementById('venvList');

        if (venvs.length === 0) {
            list.innerHTML = '<p class="text-xs text-slate-500">None detected</p>';
            return;
        }

        list.innerHTML = '';
        for (const venv of venvs) {
            const div = document.createElement('div');
            div.className = 'p-2 bg-slate-800 rounded border border-slate-700 cursor-pointer hover:bg-slate-700 transition-colors text-slate-300 text-xs';
            div.innerHTML = `
                <p class="font-mono truncate">${venv.path.split('/').pop()}</p>
                <p class="text-slate-500 text-xs">Use this venv</p>
            `;
            div.addEventListener('click', async () => {
                try {
                    const selected = await invoke('select_python', { path: venv.python_path });
                    appState.selectedPython = selected;
                    updatePythonLabel();
                    loadInstalledPackages();
                    showToast('Switched to: ' + venv.path, 'success');
                } catch (err) {
                    showToast('Failed to switch: ' + err, 'error');
                }
            });
            list.appendChild(div);
        }
    } catch (err) {
        console.error('Error detecting venvs:', err);
    }
}

function createPackageCard(pkg, isSearch = false) {
    const card = document.createElement('div');
    card.className = 'p-4 bg-slate-800 border border-slate-700 rounded-lg hover:border-slate-600 transition-colors';
    card.innerHTML = `
        <div class="flex justify-between items-start">
            <div class="flex-1">
                <h3 class="font-semibold text-slate-200">${pkg.name}</h3>
                <p class="text-xs text-slate-400 mt-1">v${pkg.version}</p>
                ${pkg.summary ? `<p class="text-xs text-slate-400 mt-2">${pkg.summary}</p>` : ''}
            </div>
            <div class="flex gap-2">
                ${!isSearch ? `
                    <button class="px-2 py-1 text-xs bg-yellow-600 hover:bg-yellow-700 text-white rounded transition-colors upgrade-btn" data-package="${pkg.name}">
                        ↑
                    </button>
                    <button class="px-2 py-1 text-xs bg-red-600 hover:bg-red-700 text-white rounded transition-colors uninstall-btn" data-package="${pkg.name}">
                        ✕
                    </button>
                ` : `
                    <button class="px-2 py-1 text-xs bg-green-600 hover:bg-green-700 text-white rounded transition-colors install-btn" data-package="${pkg.name}">
                        +
                    </button>
                `}
            </div>
        </div>
    `;

    if (!isSearch) {
        card.querySelector('.upgrade-btn')?.addEventListener('click', () => upgradePackage(pkg.name));
        card.querySelector('.uninstall-btn')?.addEventListener('click', () => uninstallPackage(pkg.name));
    } else {
        card.querySelector('.install-btn')?.addEventListener('click', () => installPackage(pkg.name));
    }

    return card;
}

function createSearchResultCard(pkg) {
    const card = document.createElement('div');
    card.className = 'p-4 bg-slate-800 border border-slate-700 rounded-lg hover:border-blue-500 transition-colors';
    card.innerHTML = `
        <div class="flex justify-between items-start">
            <div class="flex-1">
                <h3 class="font-semibold text-slate-200">${pkg.name}</h3>
                <p class="text-xs text-slate-400 mt-1">Latest: v${pkg.version}</p>
                ${pkg.summary ? `<p class="text-xs text-slate-400 mt-2">${pkg.summary}</p>` : ''}
                ${pkg.author ? `<p class="text-xs text-slate-500 mt-2">by ${pkg.author}</p>` : ''}
            </div>
            <button class="px-3 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded font-medium transition-colors install-btn text-xs" data-package="${pkg.name}">
                Install
            </button>
        </div>
    `;

    card.querySelector('.install-btn').addEventListener('click', () => installPackage(pkg.name));
    return card;
}

function createDependencyCard(dep) {
    const statusColors = {
        'Installed': 'bg-green-600/20 border-green-700/30 text-green-400',
        'Missing': 'bg-red-600/20 border-red-700/30 text-red-400',
        'VersionMismatch': 'bg-yellow-600/20 border-yellow-700/30 text-yellow-400',
    };

    const card = document.createElement('div');
    card.className = 'p-4 bg-slate-800 border border-slate-700 rounded-lg';
    card.innerHTML = `
        <div class="flex justify-between items-start">
            <div class="flex-1">
                <h3 class="font-semibold text-slate-200">${dep.name}</h3>
                <p class="text-xs text-slate-400 mt-1">Spec: ${dep.version_spec}</p>
                ${dep.installed_version ? `<p class="text-xs text-slate-400">Installed: v${dep.installed_version}</p>` : ''}
            </div>
            <span class="px-3 py-1 rounded border text-xs font-medium ${statusColors[dep.status] || ''}">
                ${dep.status}
            </span>
        </div>
    `;

    return card;
}

async function installPackage(name, version = null) {
    if (!appState.selectedPython) return;

    showPipLog();
    try {
        await invoke('install_package', {
            pythonPath: appState.selectedPython.path,
            package: name,
            version: version,
        });
        showToast(`Installed ${name}`, 'success');
        setTimeout(() => loadInstalledPackages(), 1000);
    } catch (err) {
        showToast('Installation failed: ' + err, 'error');
    }
}

async function uninstallPackage(name) {
    if (!confirm(`Uninstall ${name}?`)) return;

    showPipLog();
    try {
        await invoke('uninstall_package', {
            pythonPath: appState.selectedPython.path,
            package: name,
        });
        showToast(`Uninstalled ${name}`, 'success');
        setTimeout(() => loadInstalledPackages(), 1000);
    } catch (err) {
        showToast('Uninstall failed: ' + err, 'error');
    }
}

async function upgradePackage(name) {
    showPipLog();
    try {
        await invoke('upgrade_package', {
            pythonPath: appState.selectedPython.path,
            package: name,
        });
        showToast(`Upgraded ${name}`, 'success');
        setTimeout(() => loadInstalledPackages(), 1000);
    } catch (err) {
        showToast('Upgrade failed: ' + err, 'error');
    }
}

function showPipLog() {
    document.getElementById('pipLogContent').innerHTML = '';
    document.getElementById('pipLogModal').classList.remove('hidden');
}

function showToast(message, type = 'info') {
    const container = document.getElementById('toastContainer');
    const toast = document.createElement('div');

    const colors = {
        success: 'bg-green-600 text-white',
        error: 'bg-red-600 text-white',
        info: 'bg-blue-600 text-white',
    };

    toast.className = `px-4 py-3 rounded-lg shadow-lg font-medium ${colors[type]} animate-in`;
    toast.textContent = message;
    container.appendChild(toast);

    setTimeout(() => toast.remove(), 4000);
}
