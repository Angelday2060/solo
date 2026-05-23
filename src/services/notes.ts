import { invoke } from './ipc';

export interface NoteNotebookDto {
  id: string;
  name: string;
  parentId: string | null;
  sortOrder: number;
  isPinned: boolean;
  createdAt: string | null;
  updatedAt: string | null;
}

export interface NoteTagDto {
  id: string;
  name: string;
  createdAt: string | null;
}

export interface NoteAttachmentDto {
  id: string;
  noteId: string;
  filePath: string;
  displayName: string | null;
  sortOrder: number;
  createdAt: string | null;
}

export interface NoteListItem {
  id: string;
  notebookId: string | null;
  title: string | null;
  preview: string;
  isPinned: boolean;
  updatedAt: string | null;
}

export interface NoteDto {
  id: string;
  notebookId: string | null;
  title: string | null;
  body: string;
  isPinned: boolean;
  sortOrder: number;
  tags: NoteTagDto[];
  attachments: NoteAttachmentDto[];
  createdAt: string | null;
  updatedAt: string | null;
}

export interface NoteListFilter {
  /** 省略 = 全部；`null` = 未分类；字符串 = 指定笔记本 */
  notebookId?: string | null;
  search?: string;
}

export interface NoteCreateNotebook {
  name: string;
  parentId?: string | null;
}

export interface NoteUpdateNotebook {
  id: string;
  name: string;
  parentId?: string | null;
  isPinned?: boolean;
}

export interface NoteCreateNote {
  notebookId?: string | null;
  title?: string | null;
  body?: string;
}

export interface NoteUpdateNote {
  id: string;
  notebookId?: string | null;
  title?: string | null;
  body: string;
  isPinned?: boolean;
  tagIds?: string[];
}

export function noteListNotebooks(): Promise<NoteNotebookDto[]> {
  return invoke('note_list_notebooks');
}

export function noteCreateNotebook(
  payload: NoteCreateNotebook,
): Promise<NoteNotebookDto> {
  return invoke('note_create_notebook', { payload });
}

export function noteUpdateNotebook(
  payload: NoteUpdateNotebook,
): Promise<NoteNotebookDto> {
  return invoke('note_update_notebook', { payload });
}

export function noteDeleteNotebook(id: string): Promise<void> {
  return invoke('note_delete_notebook', { id });
}

export function noteListNotes(
  filter: NoteListFilter = {},
): Promise<NoteListItem[]> {
  const args: Record<string, unknown> = { filter: {} as Record<string, unknown> };
  const f = args.filter as Record<string, unknown>;
  if ('notebookId' in filter) {
    f.notebookId = filter.notebookId;
  }
  if (filter.search) {
    f.search = filter.search;
  }
  return invoke('note_list_notes', args);
}

export function noteGetNote(id: string): Promise<NoteDto | null> {
  return invoke('note_get_note', { id });
}

export function noteCreateNote(
  payload: NoteCreateNote = {},
): Promise<NoteDto> {
  return invoke('note_create_note', { payload });
}

export function noteUpdateNote(payload: NoteUpdateNote): Promise<NoteDto> {
  return invoke('note_update_note', { payload });
}

export function noteDeleteNote(id: string): Promise<void> {
  return invoke('note_delete_note', { id });
}

export function noteListTags(): Promise<NoteTagDto[]> {
  return invoke('note_list_tags');
}

export function noteCreateTag(name: string): Promise<NoteTagDto> {
  return invoke('note_create_tag', { payload: { name } });
}

export function noteAddAttachment(payload: {
  noteId: string;
  filePath: string;
  displayName?: string | null;
}): Promise<NoteAttachmentDto> {
  return invoke('note_add_attachment', { payload });
}

export function noteDeleteAttachment(id: string): Promise<void> {
  return invoke('note_delete_attachment', { id });
}

export function noteDisplayTitle(title: string | null | undefined): string {
  const t = title?.trim();
  return t ? t : '无标题';
}

export function formatNoteUpdatedAt(iso: string | null | undefined): string {
  if (!iso) return '';
  return iso.slice(0, 16).replace('T', ' ');
}
