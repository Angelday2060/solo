import { invoke } from './ipc';

export interface DbHealth {
  ok: boolean;
  sqliteVersion: string;
  dbPath: string;
}

/** 校验 SQLite 已连接（需在 Tauri WebView 内调用）。 */
export function fetchDbHealth(): Promise<DbHealth> {
  return invoke<DbHealth>('db_health');
}
