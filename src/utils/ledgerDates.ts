/** 本地日历日 YYYY-MM-DD */
export function localYmd(d: Date): string {
  const y = d.getFullYear();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  const day = String(d.getDate()).padStart(2, '0');
  return `${y}-${m}-${day}`;
}

/** 周一起始，与后端 chrono 周一致（周一至周日） */
export function startOfWeekMonday(d: Date): Date {
  const day = d.getDay();
  const diff = day === 0 ? -6 : 1 - day;
  const n = new Date(d.getFullYear(), d.getMonth(), d.getDate());
  n.setDate(n.getDate() + diff);
  return n;
}

export function endOfWeekSunday(startMonday: Date): Date {
  const e = new Date(startMonday);
  e.setDate(startMonday.getDate() + 6);
  return e;
}

export function startOfMonth(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth(), 1);
}

export function endOfMonth(d: Date): Date {
  return new Date(d.getFullYear(), d.getMonth() + 1, 0);
}

export function startOfYear(d: Date): Date {
  return new Date(d.getFullYear(), 0, 1);
}

export function endOfYear(d: Date): Date {
  return new Date(d.getFullYear(), 11, 31);
}

export type StatPreset = 'week' | 'month' | 'year' | 'custom';

export function rangeForPreset(
  preset: StatPreset,
  customFrom: string,
  customTo: string,
): { from: string; to: string } {
  const now = new Date();
  if (preset === 'custom') {
    return { from: customFrom, to: customTo };
  }
  if (preset === 'week') {
    const mon = startOfWeekMonday(now);
    const sun = endOfWeekSunday(mon);
    return { from: localYmd(mon), to: localYmd(sun) };
  }
  if (preset === 'month') {
    const s = startOfMonth(now);
    const e = endOfMonth(now);
    return { from: localYmd(s), to: localYmd(e) };
  }
  const s = startOfYear(now);
  const e = endOfYear(now);
  return { from: localYmd(s), to: localYmd(e) };
}
