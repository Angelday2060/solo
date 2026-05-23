import type { NoteNotebookDto } from '../services/notes';

export interface NotebookTreeNode {
  notebook: NoteNotebookDto;
  children: NotebookTreeNode[];
}

function compareNotebooks(a: NoteNotebookDto, b: NoteNotebookDto): number {
  if (a.isPinned !== b.isPinned) return a.isPinned ? -1 : 1;
  if (a.sortOrder !== b.sortOrder) return a.sortOrder - b.sortOrder;
  return a.name.localeCompare(b.name, 'zh-CN');
}

export function buildNotebookTree(
  notebooks: NoteNotebookDto[],
): NotebookTreeNode[] {
  const map = new Map<string, NotebookTreeNode>();
  for (const nb of notebooks) {
    map.set(nb.id, { notebook: nb, children: [] });
  }
  const roots: NotebookTreeNode[] = [];
  for (const nb of notebooks) {
    const node = map.get(nb.id)!;
    if (nb.parentId && map.has(nb.parentId)) {
      map.get(nb.parentId)!.children.push(node);
    } else {
      roots.push(node);
    }
  }
  const sortNodes = (nodes: NotebookTreeNode[]) => {
    nodes.sort((a, b) => compareNotebooks(a.notebook, b.notebook));
    for (const n of nodes) sortNodes(n.children);
  };
  sortNodes(roots);
  return roots;
}

export interface FlatNotebookRow {
  notebook: NoteNotebookDto;
  depth: number;
  hasChildren: boolean;
}

export function flattenNotebookTree(
  nodes: NotebookTreeNode[],
  openIds: Set<string>,
  depth = 0,
): FlatNotebookRow[] {
  const out: FlatNotebookRow[] = [];
  for (const node of nodes) {
    out.push({
      notebook: node.notebook,
      depth,
      hasChildren: node.children.length > 0,
    });
    if (openIds.has(node.notebook.id) && node.children.length > 0) {
      out.push(...flattenNotebookTree(node.children, openIds, depth + 1));
    }
  }
  return out;
}

export function notebookById(
  notebooks: NoteNotebookDto[],
  id: string,
): NoteNotebookDto | undefined {
  return notebooks.find((n) => n.id === id);
}
