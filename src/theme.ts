import { ref, watch } from 'vue';

export type Appearance = 'light' | 'dark';

const STORAGE_KEY = 'solo-appearance';

function readStored(): Appearance | null {
  try {
    const v = localStorage.getItem(STORAGE_KEY);
    if (v === 'light' || v === 'dark') return v;
  } catch {
    /* ignore */
  }
  return null;
}

function prefersDark(): boolean {
  return window.matchMedia?.('(prefers-color-scheme: dark)')?.matches ?? false;
}

export const appearance = ref<Appearance>(
  readStored() ?? (prefersDark() ? 'dark' : 'light'),
);

function sync(): void {
  document.documentElement.dataset.theme = appearance.value;
  try {
    localStorage.setItem(STORAGE_KEY, appearance.value);
  } catch {
    /* ignore */
  }
}

sync();
watch(appearance, sync);
