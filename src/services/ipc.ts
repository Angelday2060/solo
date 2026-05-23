declare global {
  interface Window {
    __TAURI_INTERNALS__?: {
      invoke?: (
        cmd: string,
        args?: Record<string, unknown>,
        options?: unknown,
      ) => Promise<unknown>;
    };
  }
}

const TAURI_HINT =
  '未在 Tauri 桌面窗口内运行：请使用「npm run tauri dev」并通过弹出的应用窗口操作；不要只在浏览器里打开开发服务器地址（此时无法调用 Rust / SQLite）。';

/**
 * 安全封装：非 Tauri WebView 时给出明确错误，避免 `undefined.invoke` 难读堆栈。
 */
export function invoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (typeof window === 'undefined') {
    return Promise.reject(new Error(TAURI_HINT));
  }
  const inv = window.__TAURI_INTERNALS__?.invoke;
  if (typeof inv !== 'function') {
    return Promise.reject(new Error(TAURI_HINT));
  }
  return inv(cmd, args ?? {}) as Promise<T>;
}
