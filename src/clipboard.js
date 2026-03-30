/**
 * ClipboardHistory — in-memory clipboard history with pinning,
 * image support, and efficient search over 1000+ items.
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

  _genId() {
    return Date.now().toString(36) + Math.random().toString(36).slice(2, 7);
  }

  addItem(type, content) {
    if (!content || (type === 'text' && !content.trim())) return;

    // Dedup: only for text items
    if (type === 'text') {
      const idx = this.items.findIndex(i => i.type === 'text' && i.content === content);
      if (idx !== -1) {
        const [item] = this.items.splice(idx, 1);
        item.createdAt = Date.now();
        this._insertSorted(item);
        this._notify();
        return;
      }
    }

    const item = {
      id: this._genId(),
      type,
      content,
      pinned: false,
      createdAt: Date.now(),
    };

    this._insertSorted(item);
    this._trim();
    this._notify();
  }

  _insertSorted(item) {
    if (item.pinned) {
      const firstUnpinned = this.items.findIndex(i => !i.pinned);
      if (firstUnpinned === -1) {
        this.items.push(item);
      } else {
        this.items.splice(firstUnpinned, 0, item);
      }
    } else {
      const firstUnpinned = this.items.findIndex(i => !i.pinned);
      if (firstUnpinned === -1) {
        this.items.push(item);
      } else {
        this.items.splice(firstUnpinned, 0, item);
      }
    }
  }

  _trim() {
    while (this.items.length > this.maxItems) {
      const idx = this._lastUnpinnedIndex();
      if (idx === -1) break;
      this.items.splice(idx, 1);
    }
  }

  _lastUnpinnedIndex() {
    for (let i = this.items.length - 1; i >= 0; i--) {
      if (!this.items[i].pinned) return i;
    }
    return -1;
  }

  togglePin(id) {
    const item = this.items.find(i => i.id === id);
    if (!item) return;
    item.pinned = !item.pinned;
    this.items = this.items.filter(i => i.id !== id);
    this._insertSorted(item);
    this._notify();
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
    return this.items.filter(
      i => i.type === 'text' && i.content.toLowerCase().includes(q)
    );
  }

  load(entries) {
    if (!Array.isArray(entries)) return;
    this.items = entries.map(e => ({
      id: e.id || this._genId(),
      type: e.type || 'text',
      content: e.content || e.text || '',
      pinned: !!e.pinned,
      createdAt: e.createdAt || e.timestamp || Date.now(),
    }));
    this._notify();
  }

  serialize() {
    return this.items.map(i => ({
      type: i.type,
      content: i.content,
      pinned: i.pinned,
      createdAt: i.createdAt,
    }));
  }
}

window.clipboardHistory = new ClipboardHistory(50);
