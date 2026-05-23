import { invoke } from './ipc';

export interface DiaryEntryListItem {
  id: string;
  entryDate: string;
  title: string | null;
  preview: string;
  sortOrder: number;
}

export interface DiaryEntryDto {
  id: string;
  entryDate: string;
  title: string | null;
  body: string;
  weather: string | null;
  mood: string | null;
  sortOrder: number;
  createdAt: string | null;
  updatedAt: string | null;
}

export interface DiaryListFilter {
  dateFrom?: string;
  dateTo?: string;
  search?: string;
}

export interface DiaryCreateEntry {
  entryDate?: string;
  title?: string | null;
  body?: string;
  weather?: string | null;
  mood?: string | null;
}

export interface DiaryUpdateEntry {
  id: string;
  entryDate: string;
  title?: string | null;
  body: string;
  weather?: string | null;
  mood?: string | null;
}

export const WEATHER_OPTIONS = [
  { value: '', label: '未填' },
  { value: 'sunny', label: '晴' },
  { value: 'cloudy', label: '多云' },
  { value: 'overcast', label: '阴' },
  { value: 'rain', label: '雨' },
  { value: 'snow', label: '雪' },
  { value: 'fog', label: '雾' },
  { value: 'wind', label: '大风' },
] as const;

export const MOOD_OPTIONS = [
  { value: '', label: '未填' },
  { value: 'calm', label: '平和' },
  { value: 'happy', label: '愉快' },
  { value: 'low', label: '低落' },
  { value: 'anxious', label: '焦虑' },
  { value: 'excited', label: '兴奋' },
  { value: 'tired', label: '疲惫' },
] as const;

export function diaryListEntries(
  filter: DiaryListFilter = {},
): Promise<DiaryEntryListItem[]> {
  return invoke('diary_list_entries', { filter });
}

export function diaryGetEntry(id: string): Promise<DiaryEntryDto | null> {
  return invoke('diary_get_entry', { id });
}

export function diaryCreateEntry(
  payload: DiaryCreateEntry = {},
): Promise<DiaryEntryDto> {
  return invoke('diary_create_entry', { payload });
}

export function diaryUpdateEntry(
  payload: DiaryUpdateEntry,
): Promise<DiaryEntryDto> {
  return invoke('diary_update_entry', { payload });
}

export function diaryDeleteEntry(id: string): Promise<void> {
  return invoke('diary_delete_entry', { id });
}
