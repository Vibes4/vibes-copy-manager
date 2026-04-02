const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const history = window.clipboardHistory;

const appEl         = document.getElementById('app');
const searchInput   = document.getElementById('search-input');
const clipList      = document.getElementById('clip-list');
const emptyState    = document.getElementById('empty-state');
const itemCount     = document.getElementById('item-count');
const clearBtn      = document.getElementById('clear-btn');
const closeBtn      = document.getElementById('close-btn');
const settingsBtn   = document.getElementById('settings-btn');

const settingsModal    = document.getElementById('settings-modal');
const settingsClose    = document.getElementById('settings-close');
const settingsSave     = document.getElementById('settings-save');
const settingsCancel   = document.getElementById('settings-cancel');
const shortcutInput      = document.getElementById('shortcut-input');
const shortcutRecorder   = document.getElementById('shortcut-recorder');
const shortcutRecorderWrap = document.getElementById('shortcut-recorder-wrap');
const shortcutDisplay    = document.getElementById('shortcut-display');
const shortcutClear      = document.getElementById('shortcut-clear');
const shortcutModeToggle = document.getElementById('shortcut-mode-toggle');
const shortcutTextWrap   = document.getElementById('shortcut-text-wrap');
const shortcutTextInput  = document.getElementById('shortcut-text-input');
const shortcutHint       = document.getElementById('shortcut-hint');
const modeLabel          = document.getElementById('mode-label');
const maxItemsInput      = document.getElementById('max-items-input');
const autostartToggle  = document.getElementById('autostart-toggle');
const settingsError    = document.getElementById('settings-error');
const themeBtns        = document.querySelectorAll('.theme-btn');

let selectedIndex = 0;
let filteredItems = [];
let saveTimer = null;
let searchDebounce = null;
let currentThemeSetting = 'dark';
let isDragging = false;

// ─── Theme ────────────────────────────────────────────────────────

function applyTheme(theme) {
  currentThemeSetting = theme;
  let effective = theme;

  if (theme === 'system') {
    effective = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }

  document.documentElement.classList.remove('dark', 'light');
  document.documentElement.classList.add(effective);
}

function updateThemeButtons(theme) {
  themeBtns.forEach(btn => {
    btn.classList.toggle('active', btn.dataset.theme === theme);
  });
}

window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
  if (currentThemeSetting === 'system') applyTheme('system');
});

// ─── Drag Handling ───────────────────────────────────────────────

document.getElementById('header').addEventListener('mousedown', (e) => {
  if (e.target.closest('button')) return;
  isDragging = true;
});

document.addEventListener('mouseup', () => {
  if (isDragging) {
    setTimeout(() => { isDragging = false; }, 200);
  }
});

// ─── Shortcut Input (Recorder + Manual Toggle) ─────────────────

let isRecording = false;
let shortcutMode = 'record'; // 'record' or 'text'

function formatKeyForDisplay(key) {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  const map = {
    'Control': isMac ? 'Ctrl' : 'Ctrl',
    'Meta': isMac ? 'Cmd' : 'Super',
    'Alt': 'Alt',
    'Shift': 'Shift',
  };
  return map[key] || key;
}

function formatKeyForConfig(parts) {
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  return parts.map(p => {
    if (p === 'Meta') return isMac ? 'Cmd' : 'Super';
    if (p === 'Control') return 'Ctrl';
    return p;
  }).join('+');
}

function renderShortcutDisplay(value) {
  if (!value) {
    shortcutDisplay.innerHTML = '<span class="text-txt-muted">Click to record shortcut…</span>';
    shortcutClear.classList.add('hidden');
    return;
  }
  const parts = value.split('+');
  shortcutDisplay.innerHTML = parts.map(p =>
    `<span class="shortcut-key">${p}</span>`
  ).join('<span class="text-txt-muted mx-0.5">+</span>');
  shortcutClear.classList.remove('hidden');
}

function setShortcutMode(mode) {
  shortcutMode = mode;
  if (mode === 'text') {
    shortcutRecorderWrap.classList.add('hidden');
    shortcutTextWrap.classList.remove('hidden');
    modeLabel.textContent = 'Record keys';
    shortcutHint.textContent = 'Type shortcut like Super+V, Ctrl+Shift+V, Alt+V. Leave empty to disable.';
    shortcutTextInput.value = shortcutInput.value || '';
    shortcutTextInput.focus();
  } else {
    shortcutTextWrap.classList.add('hidden');
    shortcutRecorderWrap.classList.remove('hidden');
    modeLabel.textContent = 'Type manually';
    shortcutHint.textContent = 'Click and press your desired key combination. Leave empty to disable.';
    const textVal = shortcutTextInput.value.trim();
    if (textVal) {
      shortcutInput.value = textVal;
      renderShortcutDisplay(textVal);
    }
  }
}

shortcutModeToggle.addEventListener('click', (e) => {
  e.preventDefault();
  if (shortcutMode === 'record') {
    const textVal = shortcutTextInput.value.trim();
    if (!textVal && shortcutInput.value) {
      shortcutTextInput.value = shortcutInput.value;
    }
    setShortcutMode('text');
  } else {
    setShortcutMode('record');
  }
});

shortcutTextInput.addEventListener('input', () => {
  shortcutInput.value = shortcutTextInput.value.trim();
});

function startRecording() {
  isRecording = true;
  shortcutRecorder.classList.add('recording');
  shortcutDisplay.innerHTML = '<span class="text-txt-muted animate-pulse">Press keys…</span>';
}

function stopRecording() {
  isRecording = false;
  shortcutRecorder.classList.remove('recording');
}

shortcutRecorder.addEventListener('focus', () => {
  startRecording();
});

shortcutRecorder.addEventListener('blur', () => {
  stopRecording();
  if (!shortcutInput.value) {
    renderShortcutDisplay(null);
  }
});

shortcutRecorder.addEventListener('keydown', (e) => {
  if (!isRecording) return;
  e.preventDefault();
  e.stopPropagation();

  if (e.key === 'Escape') {
    stopRecording();
    shortcutRecorder.blur();
    return;
  }

  if (e.key === 'Backspace' || e.key === 'Delete') {
    shortcutInput.value = '';
    renderShortcutDisplay(null);
    stopRecording();
    shortcutRecorder.blur();
    return;
  }

  const modifiers = [];
  if (e.ctrlKey) modifiers.push('Control');
  if (e.altKey) modifiers.push('Alt');
  if (e.shiftKey) modifiers.push('Shift');
  if (e.metaKey) modifiers.push('Meta');

  const isModifierOnly = ['Control', 'Alt', 'Shift', 'Meta'].includes(e.key);
  if (isModifierOnly) {
    const display = modifiers.map(formatKeyForDisplay).join(' + ') + ' + …';
    shortcutDisplay.innerHTML = `<span class="text-txt-muted">${display}</span>`;
    return;
  }

  if (modifiers.length === 0) return;

  let key = e.key.length === 1 ? e.key.toUpperCase() : e.key;
  const configParts = [...modifiers, key];
  const configStr = formatKeyForConfig(configParts);
  const displayParts = [...modifiers.map(formatKeyForDisplay), key];
  const displayStr = displayParts.join('+');

  shortcutInput.value = configStr;
  renderShortcutDisplay(displayStr);
  stopRecording();
  shortcutRecorder.blur();
});

shortcutClear.addEventListener('click', (e) => {
  e.stopPropagation();
  shortcutInput.value = '';
  shortcutTextInput.value = '';
  renderShortcutDisplay(null);
});

// ─── Window Control ──────────────────────────────────────────────

function hideWindow() {
  appEl.classList.remove('app-visible');
  appEl.classList.add('app-hiding');
  setTimeout(() => {
    invoke('hide_window').catch(e => console.error('hide_window:', e));
  }, 100);
}

function activateWindow() {
  selectedIndex = 0;
  searchInput.value = '';
  render();

  appEl.classList.remove('app-visible', 'app-hiding');
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      appEl.classList.add('app-visible');
    });
  });

  const focusInput = () => {
    window.focus();
    searchInput.focus();
  };
  focusInput();
  setTimeout(focusInput, 50);
  setTimeout(focusInput, 120);
  setTimeout(focusInput, 250);
}

// ─── Rendering ───────────────────────────────────────────────────

let lastRenderedIds = '';

function render(items) {
  const query = searchInput.value.trim();
  filteredItems = query ? history.search(query) : (items || history.items);

  const count = filteredItems.length;
  itemCount.textContent = `${count} item${count !== 1 ? 's' : ''}${query ? ' matched' : ''}`;

  emptyState.classList.toggle('hidden', count > 0);
  clipList.classList.toggle('hidden', count === 0);

  if (selectedIndex >= count) selectedIndex = Math.max(0, count - 1);

  const currentIds = filteredItems.map(i => i.id + (i.pinned ? 'p' : '')).join(',');
  const needsFullRender = currentIds !== lastRenderedIds;
  lastRenderedIds = currentIds;

  if (needsFullRender) {
    clipList.innerHTML = '';
    filteredItems.forEach((item, idx) => {
      clipList.appendChild(createItemEl(item, idx));
    });
  } else {
    const children = clipList.children;
    for (let i = 0; i < children.length; i++) {
      children[i].className = itemClass(i === selectedIndex);
    }
  }

  scheduleSave();
}

function itemClass(selected) {
  return `clip-item group flex items-start gap-2.5 px-3 py-2 rounded-lg cursor-pointer transition-all duration-100 border border-transparent ${
    selected ? 'bg-accent/10 border-accent/30' : 'hover:bg-bg-hover'
  }`;
}

function createItemEl(item, idx) {
  const li = document.createElement('li');
  li.className = itemClass(idx === selectedIndex);
  li.dataset.idx = idx;
  li.dataset.id = item.id;

  if (item.type === 'image') {
    li.innerHTML = `
      <div class="flex-1 min-w-0 pt-0.5">
        <img src="data:image/png;base64,${item.content}" class="max-h-16 rounded border border-border object-contain" alt="clipboard image" />
        <p class="text-2xs text-txt-muted mt-1">${timeAgo(item.createdAt)}</p>
      </div>
      ${itemActions(item)}
    `;
  } else {
    const preview = escHtml(truncate(item.content, 120));
    li.innerHTML = `
      <div class="flex-1 min-w-0 pt-0.5">
        <p class="text-sm leading-snug text-txt break-words line-clamp">${preview}</p>
        <p class="text-2xs text-txt-muted mt-1">${timeAgo(item.createdAt)}</p>
      </div>
      ${itemActions(item)}
    `;
  }

  li.addEventListener('click', (e) => {
    if (e.target.closest('.pin-btn')) {
      history.togglePin(item.id);
      return;
    }
    if (e.target.closest('.del-btn')) {
      history.removeItem(item.id);
      return;
    }
    pasteItem(idx);
  });

  return li;
}

function itemActions(item) {
  return `
    <div class="flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity duration-150">
      <button class="pin-btn p-1 rounded hover:bg-border transition-colors duration-150 ${item.pinned ? 'text-accent' : 'text-txt-muted hover:text-txt'}" title="${item.pinned ? 'Unpin' : 'Pin'}">
        <svg class="w-3 h-3" fill="${item.pinned ? 'currentColor' : 'none'}" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"/>
        </svg>
      </button>
      <button class="del-btn p-1 rounded hover:bg-border transition-colors duration-150 text-txt-muted hover:text-red-400" title="Remove">
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
        </svg>
      </button>
    </div>
  `;
}

function truncate(str, max) {
  const single = str.replace(/\s+/g, ' ').trim();
  return single.length > max ? single.slice(0, max) + '…' : single;
}

function timeAgo(ts) {
  const diff = Math.floor((Date.now() - ts) / 1000);
  if (diff < 5)     return 'just now';
  if (diff < 60)    return `${diff}s ago`;
  if (diff < 3600)  return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

function escHtml(str) {
  const d = document.createElement('div');
  d.textContent = str;
  return d.innerHTML;
}

// ─── Actions ─────────────────────────────────────────────────────

async function pasteItem(idx) {
  const item = filteredItems[idx];
  if (!item) return;

  try {
    if (item.type === 'image') {
      await invoke('write_image_clipboard', { base64Png: item.content });
    } else {
      await invoke('write_clipboard', { text: item.content });
      history.addItem('text', item.content);
    }
  } catch (e) {
    console.error('write clipboard failed:', e);
    return;
  }

  try {
    await invoke('paste_and_hide');
  } catch (e) {
    console.error('paste_and_hide failed:', e);
    hideWindow();
  }
}

// ─── Keyboard Navigation ─────────────────────────────────────────

document.addEventListener('keydown', (e) => {
  if (isRecording) return;

  if (!settingsModal.classList.contains('hidden')) {
    if (e.key === 'Escape') closeSettings();
    return;
  }

  const count = filteredItems.length;

  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault();
      if (count > 0) {
        selectedIndex = Math.min(selectedIndex + 1, count - 1);
        render();
        scrollToSelected();
      }
      break;

    case 'ArrowUp':
      e.preventDefault();
      if (count > 0) {
        selectedIndex = Math.max(selectedIndex - 1, 0);
        render();
        scrollToSelected();
      }
      break;

    case 'Enter':
      e.preventDefault();
      if (count > 0) pasteItem(selectedIndex);
      break;

    case 'Escape':
      hideWindow();
      break;

    case 'Delete':
      if (count > 0 && filteredItems[selectedIndex]) {
        history.removeItem(filteredItems[selectedIndex].id);
      }
      break;

    case 'p':
    case 'P':
      if ((e.ctrlKey || e.metaKey) && count > 0 && filteredItems[selectedIndex]) {
        e.preventDefault();
        history.togglePin(filteredItems[selectedIndex].id);
      }
      break;
  }
});

function scrollToSelected() {
  const el = clipList.querySelector(`[data-idx="${selectedIndex}"]`);
  if (el) el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
}

// ─── Settings Modal ──────────────────────────────────────────────

let pendingTheme = 'dark';

function openSettings() {
  Promise.all([
    invoke('get_config'),
    invoke('get_autostart'),
  ]).then(([cfg, autoEnabled]) => {
    shortcutInput.value = cfg.shortcut || '';
    shortcutTextInput.value = cfg.shortcut || '';
    renderShortcutDisplay(cfg.shortcut || null);
    setShortcutMode('record');
    maxItemsInput.value = cfg.maxItems || 50;
    autostartToggle.checked = autoEnabled;
    pendingTheme = cfg.theme || 'dark';
    updateThemeButtons(pendingTheme);
    settingsError.classList.add('hidden');
    settingsModal.classList.remove('hidden');
  }).catch(e => console.error('get_config:', e));
}

function closeSettings() {
  settingsModal.classList.add('hidden');
  searchInput.focus();
}

async function saveSettings() {
  const shortcut = shortcutInput.value.trim() || null;
  const maxItems = parseInt(maxItemsInput.value, 10);

  if (isNaN(maxItems) || maxItems < 10) {
    settingsError.textContent = 'Max items must be at least 10';
    settingsError.classList.remove('hidden');
    return;
  }

  try {
    const autoStart = autostartToggle.checked;
    const theme = pendingTheme;
    await invoke('set_config', { cfg: { shortcut, maxItems, autoStart, theme } });
    history.maxItems = maxItems;
    applyTheme(theme);
    closeSettings();
  } catch (e) {
    settingsError.textContent = String(e);
    settingsError.classList.remove('hidden');
  }
}

settingsBtn.addEventListener('click', openSettings);
settingsClose.addEventListener('click', closeSettings);
settingsCancel.addEventListener('click', closeSettings);
settingsSave.addEventListener('click', saveSettings);

themeBtns.forEach(btn => {
  btn.addEventListener('click', () => {
    pendingTheme = btn.dataset.theme;
    updateThemeButtons(pendingTheme);
    applyTheme(pendingTheme);
  });
});

// ─── Window Lifecycle ────────────────────────────────────────────

listen('window-shown', () => {
  activateWindow();
});

listen('open-settings', () => {
  setTimeout(openSettings, 200);
});

window.addEventListener('blur', () => {
  if (isDragging) return;
  if (!settingsModal.classList.contains('hidden')) return;
  hideWindow();
});

// ─── Search (debounced 50ms) ─────────────────────────────────────

searchInput.addEventListener('input', () => {
  clearTimeout(searchDebounce);
  searchDebounce = setTimeout(() => {
    selectedIndex = 0;
    render();
  }, 50);
});

// ─── Button Handlers ─────────────────────────────────────────────

clearBtn.addEventListener('click', () => {
  history.clear();
});

closeBtn.addEventListener('click', hideWindow);

// ─── Persistence ─────────────────────────────────────────────────

function scheduleSave() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    invoke('save_history', { entries: history.serialize() }).catch(() => {});
  }, 1000);
}

async function loadPersistedHistory() {
  try {
    const entries = await invoke('load_history');
    if (entries && entries.length) {
      history.load(entries);
    }
  } catch (e) {
    console.error('load_history failed:', e);
  }
}

// ─── Config-driven init ──────────────────────────────────────────

async function loadConfig() {
  try {
    const cfg = await invoke('get_config');
    if (cfg) {
      if (cfg.maxItems) history.maxItems = cfg.maxItems;
      applyTheme(cfg.theme || 'dark');
    }
  } catch (_) {}
}

// ─── Init ────────────────────────────────────────────────────────

history.onChange = render;

listen('clipboard-update', (event) => {
  const payload = event.payload;
  history.addItem(payload.type, payload.content);
  selectedIndex = 0;
});

loadConfig();
loadPersistedHistory();
render();
