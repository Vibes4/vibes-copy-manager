/**
 * ClipboardHistory — manages the in-memory clipboard history.
 * Handles dedup, ordering, search, pinning, and persistence bridge.
 */

class ClipboardHistory {
  constructor(maxItems = 50) {
    this.items = [];
    this.maxItems = maxItems;
    this.onChange = null;
  }

  _notify() {
    if (this.onChange) this.onChange(this.items);
  }

  addItem(text) {
    if (!text || !text.trim()) return;

    const existing = this.items.findIndex(i => i.text === text);
    if (existing !== -1) {
      const [item] = this.items.splice(existing, 1);
      item.timestamp = Date.now();
      this.items.unshift(item);
    } else {
      this.items.unshift({
        id: Date.now().toString(36) + Math.random().toString(36).slice(2, 7),
        text,
        timestamp: Date.now(),
        pinned: false,
      });
    }

    if (this.items.length > this.maxItems) {
      const unpinned = [...this.items].reverse().findIndex(i => !i.pinned);
      if (unpinned !== -1) {
        this.items.splice(this.items.length - 1 - unpinned, 1);
      }
    }

    this._notify();
  }

  togglePin(id) {
    const item = this.items.find(i => i.id === id);
    if (item) {
      item.pinned = !item.pinned;
      this._notify();
    }
  }

  removeItem(id) {
    this.items = this.items.filter(i => i.id !== id);
    this._notify();
  }

  clear() {
    this.items = this.items.filter(i => i.pinned);
    this._notify();
  }

  search(query) {
    if (!query || !query.trim()) return this.items;
    const q = query.toLowerCase();
    return this.items.filter(i => i.text.toLowerCase().includes(q));
  }

  /** Restore from persistence data */
  load(entries) {
    if (!Array.isArray(entries)) return;
    this.items = entries.map(e => ({
      id: e.id || Date.now().toString(36) + Math.random().toString(36).slice(2, 7),
      text: e.text,
      timestamp: e.timestamp || Date.now(),
      pinned: !!e.pinned,
    }));
    this._notify();
  }

  /** Serialize for persistence */
  serialize() {
    return this.items.map(i => ({
      text: i.text,
      timestamp: i.timestamp,
      pinned: i.pinned,
    }));
  }
}

window.clipboardHistory = new ClipboardHistory(50);
