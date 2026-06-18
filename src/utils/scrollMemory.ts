const SCROLL_STORAGE_PREFIX = "readnotes:scroll:";

export function saveScrollPosition(key: string, scrollTop: number) {
  try {
    window.sessionStorage.setItem(`${SCROLL_STORAGE_PREFIX}${key}`, String(scrollTop));
  } catch {
    // Scroll restoration is optional when session storage is unavailable.
  }
}

export function restoreScrollPosition(key: string, element: HTMLElement | null) {
  if (!element) {
    return;
  }

  try {
    const storedValue = window.sessionStorage.getItem(`${SCROLL_STORAGE_PREFIX}${key}`);
    const scrollTop = storedValue === null ? 0 : Number.parseFloat(storedValue);
    element.scrollTop = Number.isFinite(scrollTop) ? scrollTop : 0;
  } catch {
    element.scrollTop = 0;
  }
}
