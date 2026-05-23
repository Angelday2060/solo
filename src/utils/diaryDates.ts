import type { DiaryEntryListItem } from '../services/diary';
import { localYmd } from './ledgerDates';

const WEEKDAY_ZH = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];

export interface ParsedIsoDate {
  y: number;
  m: number;
  d: number;
}

export function parseIsoDateLocal(iso: string): ParsedIsoDate | null {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso || '');
  if (!m) return null;
  const y = Number(m[1]);
  const mo = Number(m[2]);
  const d = Number(m[3]);
  if (!y || mo < 1 || mo > 12 || d < 1 || d > 31) return null;
  return { y, m: mo, d };
}

export function diaryTodayIso(): string {
  return localYmd(new Date());
}

export function diaryWeekdayZh(iso: string): string {
  const p = parseIsoDateLocal(iso);
  if (!p) return '';
  const dt = new Date(p.y, p.m - 1, p.d);
  return WEEKDAY_ZH[dt.getDay()] || '';
}

export function diaryEditorFullLabel(iso: string): string {
  const p = parseIsoDateLocal(iso);
  if (!p) return '—';
  const wd = diaryWeekdayZh(iso);
  return `${p.y}年${p.m}月${p.d}日${wd ? ` ${wd}` : ''}`;
}

export interface DiaryTreeYear {
  year: number;
  months: DiaryTreeMonth[];
}

export interface DiaryTreeMonth {
  month: number;
  days: DiaryEntryListItem[];
}

export function groupDiaryEntriesDesc(
  samples: DiaryEntryListItem[],
): DiaryTreeYear[] {
  const valid = samples.filter((x) => parseIsoDateLocal(x.entryDate));
  valid.sort((a, b) => {
    if (a.entryDate !== b.entryDate) {
      return a.entryDate < b.entryDate ? 1 : -1;
    }
    if (a.sortOrder !== b.sortOrder) {
      return a.sortOrder - b.sortOrder;
    }
    return 0;
  });

  const byYear = new Map<number, Map<number, DiaryEntryListItem[]>>();
  valid.forEach((row) => {
    const p = parseIsoDateLocal(row.entryDate);
    if (!p) return;
    if (!byYear.has(p.y)) byYear.set(p.y, new Map());
    const byMonth = byYear.get(p.y)!;
    if (!byMonth.has(p.m)) byMonth.set(p.m, []);
    byMonth.get(p.m)!.push(row);
  });

  const years = Array.from(byYear.keys()).sort((a, b) => b - a);
  return years.map((y) => ({
    year: y,
    months: Array.from(byYear.get(y)!.keys())
      .sort((a, b) => b - a)
      .map((mo) => ({
        month: mo,
        days: byYear.get(y)!.get(mo)!,
      })),
  }));
}

export function getNewestDiaryEntry(
  samples: DiaryEntryListItem[],
): DiaryEntryListItem | null {
  const valid = samples.filter((x) => parseIsoDateLocal(x.entryDate));
  valid.sort((a, b) => {
    if (a.entryDate !== b.entryDate) {
      return a.entryDate < b.entryDate ? 1 : -1;
    }
    if (a.sortOrder !== b.sortOrder) {
      return a.sortOrder - b.sortOrder;
    }
    return 0;
  });
  return valid.length ? valid[0] : null;
}

export function daySubLabel(row: DiaryEntryListItem): string {
  const wd = diaryWeekdayZh(row.entryDate);
  const pv = (row.preview || '').trim();
  if (wd) return pv ? `${wd} · ${pv}` : wd;
  return pv || '（无摘录）';
}
