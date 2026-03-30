const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const history = window.clipboardHistory;

const searchInput = document.getElementById('search-input');
const clipList    = document.getElementById('clip-list');
const emptyState  = document.getElementById('empty-state');
const itemCount   = document.getElementById('item-count');
const clearBtn    = document.getElementById('clear-btn');
const closeBtn    = document.getElementById('close-btn');

let selectedIndex = 0;
let filteredItems = [];
let saveTimer = null;

// --- Window control ---

function hideWindow() {
  invoke('hide_window').catch(e => console.error('hide_window:', e));
}

function activateWindow() {
  selectedIndex = 0;
  searchInput.value = '';
  render();

  // Aggressive focus: the native window may take 30-100ms to fully
  // activate on X11 after xdotool. We retry at staggered intervals
  // to ensure the webview + search input get focus.
  const focusInput = () => {
    window.focus();
    searchInput.focus();
  };
  focusInput();
  setTimeout(focusInput, 50);
  setTimeout(focusInput, 120);
  setTimeout(focusInput, 250);
}

// --- Rendering ---

function render(items) {
  const query = searchInput.value.trim();
  filteredItems = query ? history.search(query) : items || history.items;

  const count = filteredItems.length;
  itemCount.textContent = `${count} item${count !== 1 ? 's' : ''}${query ? ' matched' : ''}`;

  emptyState.classList.toggle('hidden', count > 0);
  clipList.classList.toggle('hidden', count === 0);

  if (selectedIndex >= count) selectedIndex = Math.max(0, count - 1);

  clipList.innerHTML = '';

  filteredItems.forEach((item, idx) => {
    const li = document.createElement('li');
    li.className = `clip-item group flex items-start gap-2.5 px-3 py-2 rounded-lg cursor-pointer transition-all duration-100 border border-transparent ${
      idx === selectedIndex
        ? 'bg-accent/10 border-accent/30'
        : 'hover:bg-bg-hover'
    }`;
    li.dataset.idx = idx;
    li.dataset.id = item.id;

    const preview = escHtml(truncate(item.text, 120));
    const time = timeAgo(item.timestamp);

    li.innerHTML = `
      <div class="flex-1 min-w-0 pt-0.5">
        <p class="text-sm leading-snug text-txt break-words line-clamp">${preview}</p>
        <p class="text-2xs text-txt-muted mt-1">${time}</p>
      </div>
      <div class="flex items-center gap-0.5 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
        <button class="pin-btn p-1 rounded hover:bg-border transition-colors ${item.pinned ? 'text-accent' : 'text-txt-muted hover:text-txt'}" title="${item.pinned ? 'Unpin' : 'Pin'}">
          <svg class="w-3 h-3" fill="${item.pinned ? 'currentColor' : 'none'}" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"/>
          </svg>
        </button>
        <button class="del-btn p-1 rounded hover:bg-border transition-colors text-txt-muted hover:text-red-400" title="Remove">
          <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>
    `;

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

    clipList.appendChild(li);
  });

  scheduleSave();
}

function truncate(str, max) {
  const single = str.replace(/\s+/g, ' ').trim();
  return single.length > max ? single.slice(0, max) + '…' : single;
}

function timeAgo(ts) {
  const diff = Math.floor((Date.now() - ts) / 1000);
  if (diff < 5) return 'just now';
  if (diff < 60) return `${diff}s ago`;
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  return `${Math.floor(diff / 86400)}d ago`;
}

function escHtml(str) {
  const d = document.createElement('div');
  d.textContent = str;
  return d.innerHTML;
}

// --- Actions ---

async function pasteItem(idx) {
  const item = filteredItems[idx];
  if (!item) return;

  try {
    await invoke('write_clipboard', { text: item.text });
  } catch (e) {
    console.error('write_clipboard failed:', e);
    return;
  }

  history.addItem(item.text);

  try {
    await invoke('paste_and_hide');
  } catch (e) {
    console.error('paste_and_hide failed:', e);
    hideWindow();
  }
}

// --- Keyboard navigation ---

document.addEventListener('keydown', (e) => {
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
  }
});

function scrollToSelected() {
  const el = clipList.querySelector(`[data-idx="${selectedIndex}"]`);
  if (el) el.scrollIntoView({ block: 'nearest' });
}

// --- Window lifecycle events ---

// Rust emits "window-shown" after show+focus — this is how we
// reliably activate the DOM (search input) after the native
// window is focused.
listen('window-shown', () => {
  activateWindow();
});

// JS-side blur as backup for click-outside on WMs where
// Rust's Focused(false) doesn't fire.
window.addEventListener('blur', () => {
  hideWindow();
});

// --- Search ---

searchInput.addEventListener('input', () => {
  selectedIndex = 0;
  render();
});

// --- Button handlers ---

clearBtn.addEventListener('click', () => {
  history.clear();
});

closeBtn.addEventListener('click', hideWindow);

// --- Persistence ---

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

// --- Init ---

history.onChange = render;

listen('clipboard-update', (event) => {
  const { text } = event.payload;
  history.addItem(text);
  selectedIndex = 0;
});

loadPersistedHistory();
render();
