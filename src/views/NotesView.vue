<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import {
  formatNoteUpdatedAt,
  noteAddAttachment,
  noteCreateNote,
  noteCreateNotebook,
  noteCreateTag,
  noteDeleteAttachment,
  noteDeleteNote,
  noteDeleteNotebook,
  noteDisplayTitle,
  noteGetNote,
  noteListNotebooks,
  noteListNotes,
  noteListTags,
  noteUpdateNote,
  noteUpdateNotebook,
  type NoteAttachmentDto,
  type NoteListItem,
  type NoteNotebookDto,
  type NoteTagDto,
} from '../services/notes';
import {
  buildNotebookTree,
  flattenNotebookTree,
} from '../utils/noteNotebooks';

type NotebookFilter = 'all' | 'uncategorized' | string;

const notebooks = ref<NoteNotebookDto[]>([]);
const noteList = ref<NoteListItem[]>([]);
const allTags = ref<NoteTagDto[]>([]);

const loading = ref(false);
const saving = ref(false);
const listLoading = ref(false);

const selectedNotebookFilter = ref<NotebookFilter>('all');
const selectedNotebookId = ref<string | null>(null);
const selectedNoteId = ref<string | null>(null);
const searchQuery = ref('');
const openNotebookIds = ref<Set<string>>(new Set());

const title = ref('');
const body = ref('');
const notebookId = ref<string | null>(null);
const isPinned = ref(false);
const tagIds = ref<string[]>([]);
const attachments = ref<NoteAttachmentDto[]>([]);
const updatedAt = ref<string | null>(null);

const dirty = ref(false);
const skipDirtyWatch = ref(false);
const newTagInput = ref('');
let saveTimer: ReturnType<typeof setTimeout> | null = null;

const notebookTree = computed(() => buildNotebookTree(notebooks.value));
const flatNotebookRows = computed(() =>
  flattenNotebookTree(notebookTree.value, openNotebookIds.value),
);

const selectedTags = computed(() =>
  allTags.value.filter((t) => tagIds.value.includes(t.id)),
);

const filterLabel = computed(() => {
  if (selectedNotebookFilter.value === 'all') return '全部笔记';
  if (selectedNotebookFilter.value === 'uncategorized') return '未分类';
  const nb = notebooks.value.find((n) => n.id === selectedNotebookFilter.value);
  return nb?.name ?? '笔记本';
});

function isNotebookOpen(id: string): boolean {
  return openNotebookIds.value.has(id);
}

function toggleNotebookOpen(id: string) {
  const next = new Set(openNotebookIds.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  openNotebookIds.value = next;
}

function selectNotebookFilter(filter: NotebookFilter) {
  selectedNotebookFilter.value = filter;
  if (filter !== 'all' && filter !== 'uncategorized') {
    selectedNotebookId.value = filter;
  } else {
    selectedNotebookId.value = null;
  }
  void refreshNoteList();
}

async function refreshNotebooks() {
  notebooks.value = await noteListNotebooks();
  if (openNotebookIds.value.size === 0) {
    const next = new Set<string>();
    for (const nb of notebooks.value) {
      if (!nb.parentId) next.add(nb.id);
    }
    openNotebookIds.value = next;
  }
}

async function refreshNoteList(selectId?: string | null) {
  listLoading.value = true;
  try {
    const filter: Parameters<typeof noteListNotes>[0] = {};
    if (selectedNotebookFilter.value === 'uncategorized') {
      filter.notebookId = null;
    } else if (selectedNotebookFilter.value !== 'all') {
      filter.notebookId = selectedNotebookFilter.value;
    }
    const q = searchQuery.value.trim();
    if (q) filter.search = q;
    noteList.value = await noteListNotes(filter);

    const target =
      selectId ??
      selectedNoteId.value ??
      noteList.value[0]?.id ??
      null;
    if (target && noteList.value.some((n) => n.id === target)) {
      await selectNote(target, { skipSave: true });
    } else if (noteList.value.length === 0) {
      selectedNoteId.value = null;
      clearEditor();
    } else if (noteList.value[0]) {
      await selectNote(noteList.value[0].id, { skipSave: true });
    }
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  } finally {
    listLoading.value = false;
  }
}

async function refreshTags() {
  allTags.value = await noteListTags();
}

async function refreshAll(selectNoteId?: string | null) {
  loading.value = true;
  try {
    await Promise.all([refreshNotebooks(), refreshTags()]);
    await refreshNoteList(selectNoteId);
  } finally {
    loading.value = false;
  }
}

function clearEditor() {
  skipDirtyWatch.value = true;
  title.value = '';
  body.value = '';
  notebookId.value = null;
  isPinned.value = false;
  tagIds.value = [];
  attachments.value = [];
  updatedAt.value = null;
  newTagInput.value = '';
  dirty.value = false;
  skipDirtyWatch.value = false;
}

function applyNoteToForm(
  note: NonNullable<Awaited<ReturnType<typeof noteGetNote>>>,
) {
  skipDirtyWatch.value = true;
  title.value = note.title ?? '';
  body.value = note.body ?? '';
  notebookId.value = note.notebookId;
  isPinned.value = note.isPinned;
  tagIds.value = note.tags.map((t) => t.id);
  attachments.value = [...note.attachments];
  updatedAt.value = note.updatedAt;
  dirty.value = false;
  skipDirtyWatch.value = false;
}

async function flushSave(): Promise<boolean> {
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
  }
  if (!dirty.value || !selectedNoteId.value || saving.value) {
    return true;
  }
  const savingId = selectedNoteId.value;
  saving.value = true;
  try {
    const updated = await noteUpdateNote({
      id: savingId,
      notebookId: notebookId.value,
      title: title.value.trim() || null,
      body: body.value,
      isPinned: isPinned.value,
      tagIds: tagIds.value,
    });
    applyNoteToForm(updated);
    await refreshNoteList(savingId);
    return true;
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
    return false;
  } finally {
    saving.value = false;
  }
}

function scheduleSave() {
  if (skipDirtyWatch.value || !selectedNoteId.value) return;
  dirty.value = true;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    void flushSave();
  }, 700);
}

async function selectNote(
  id: string,
  opts: { skipSave?: boolean } = {},
) {
  if (!opts.skipSave && selectedNoteId.value && selectedNoteId.value !== id) {
    const ok = await flushSave();
    if (!ok) return;
  }
  loading.value = true;
  try {
    const note = await noteGetNote(id);
    if (!note) {
      window.alert('笔记不存在');
      await refreshNoteList();
      return;
    }
    selectedNoteId.value = id;
    applyNoteToForm(note);
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}

async function startNewNote() {
  if (selectedNoteId.value) {
    const ok = await flushSave();
    if (!ok) return;
  }
  try {
    const notebookId =
      selectedNotebookFilter.value !== 'all' &&
      selectedNotebookFilter.value !== 'uncategorized'
        ? selectedNotebookFilter.value
        : null;
    const created = await noteCreateNote({ notebookId });
    await refreshNoteList(created.id);
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function removeCurrentNote() {
  const id = selectedNoteId.value;
  if (!id) return;
  if (!window.confirm('确定删除这篇笔记？')) return;
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
  }
  dirty.value = false;
  try {
    await noteDeleteNote(id);
    selectedNoteId.value = null;
    clearEditor();
    await refreshNoteList();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function createNotebook(parentId?: string | null) {
  const name = window.prompt('笔记本名称');
  if (name === null) return;
  const trimmed = name.trim();
  if (!trimmed) {
    window.alert('名称不能为空');
    return;
  }
  try {
    const created = await noteCreateNotebook({
      name: trimmed,
      parentId: parentId ?? null,
    });
    await refreshNotebooks();
    const nextOpen = new Set(openNotebookIds.value);
    nextOpen.add(created.id);
    if (parentId) nextOpen.add(parentId);
    openNotebookIds.value = nextOpen;
    selectNotebookFilter(created.id);
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function renameSelectedNotebook() {
  const id = selectedNotebookId.value;
  if (!id) return;
  const nb = notebooks.value.find((n) => n.id === id);
  if (!nb) return;
  const name = window.prompt('重命名笔记本', nb.name);
  if (name === null) return;
  const trimmed = name.trim();
  if (!trimmed) {
    window.alert('名称不能为空');
    return;
  }
  try {
    await noteUpdateNotebook({
      id,
      name: trimmed,
      parentId: nb.parentId,
      isPinned: nb.isPinned,
    });
    await refreshNotebooks();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function toggleNotebookPin() {
  const id = selectedNotebookId.value;
  if (!id) return;
  const nb = notebooks.value.find((n) => n.id === id);
  if (!nb) return;
  try {
    await noteUpdateNotebook({
      id,
      name: nb.name,
      parentId: nb.parentId,
      isPinned: !nb.isPinned,
    });
    await refreshNotebooks();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function deleteSelectedNotebook() {
  const id = selectedNotebookId.value;
  if (!id) return;
  const nb = notebooks.value.find((n) => n.id === id);
  if (!nb) return;
  if (
    !window.confirm(
      `确定删除笔记本「${nb.name}」？所含笔记将移至「未分类」。`,
    )
  ) {
    return;
  }
  try {
    await noteDeleteNotebook(id);
    if (selectedNotebookFilter.value === id) {
      selectNotebookFilter('all');
    }
    selectedNotebookId.value = null;
    await refreshAll(selectedNoteId.value);
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function addTagFromInput() {
  const name = newTagInput.value.trim();
  if (!name || !selectedNoteId.value) return;
  try {
    const tag = await noteCreateTag(name);
    if (!allTags.value.some((t) => t.id === tag.id)) {
      allTags.value = [...allTags.value, tag].sort((a, b) =>
        a.name.localeCompare(b.name, 'zh-CN'),
      );
    }
    if (!tagIds.value.includes(tag.id)) {
      tagIds.value = [...tagIds.value, tag.id];
      scheduleSave();
    }
    newTagInput.value = '';
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

function removeTag(tagId: string) {
  tagIds.value = tagIds.value.filter((id) => id !== tagId);
  scheduleSave();
}

async function addAttachmentPath() {
  if (!selectedNoteId.value) return;
  const path = window.prompt('附件本地路径');
  if (path === null) return;
  const trimmed = path.trim();
  if (!trimmed) {
    window.alert('路径不能为空');
    return;
  }
  try {
    const att = await noteAddAttachment({
      noteId: selectedNoteId.value,
      filePath: trimmed,
    });
    attachments.value = [...attachments.value, att];
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function removeAttachment(id: string) {
  if (!window.confirm('移除该附件记录？（不删除磁盘文件）')) return;
  try {
    await noteDeleteAttachment(id);
    attachments.value = attachments.value.filter((a) => a.id !== id);
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

function onSearchInput() {
  void refreshNoteList(selectedNoteId.value);
}

watch([title, body, isPinned], () => {
  if (skipDirtyWatch.value) return;
  scheduleSave();
});

onMounted(() => {
  void refreshAll();
});
</script>

<template>
  <div class="view-root">
    <div class="layout-notes">
      <div class="panel tree">
        <div class="panel-h">文件夹 / 笔记本</div>
        <div class="panel-body note-tree-panel-body">
          <div class="note-tree-toolbar">
            <button
              type="button"
              class="btn btn-sm"
              :disabled="loading"
              @click="createNotebook(null)"
            >
              新建
            </button>
            <button
              type="button"
              class="btn btn-sm"
              :disabled="loading || !selectedNotebookId"
              title="在当前笔记本下创建子级"
              @click="createNotebook(selectedNotebookId)"
            >
              子笔记本
            </button>
          </div>
          <div class="note-tree-filters">
            <button
              type="button"
              class="note-tree-filter"
              :class="{ 'is-selected': selectedNotebookFilter === 'all' }"
              @click="selectNotebookFilter('all')"
            >
              全部笔记
            </button>
            <button
              type="button"
              class="note-tree-filter"
              :class="{ 'is-selected': selectedNotebookFilter === 'uncategorized' }"
              @click="selectNotebookFilter('uncategorized')"
            >
              未分类
            </button>
          </div>
          <p v-if="loading && notebooks.length === 0" class="note-tree-empty">
            加载中…
          </p>
          <div
            v-else
            class="note-tree"
            role="tree"
            aria-label="笔记本树"
          >
            <div
              v-for="row in flatNotebookRows"
              :key="row.notebook.id"
              class="note-tree-row"
              :class="{
                'is-selected': selectedNotebookFilter === row.notebook.id,
              }"
              role="treeitem"
              :style="{ paddingLeft: `${8 + row.depth * 14}px` }"
            >
              <button
                v-if="row.hasChildren"
                type="button"
                class="note-tree-chevron-btn"
                :aria-expanded="isNotebookOpen(row.notebook.id)"
                :aria-label="isNotebookOpen(row.notebook.id) ? '折叠' : '展开'"
                @click.stop="toggleNotebookOpen(row.notebook.id)"
              >
                <span
                  class="note-tree-chevron"
                  :class="{ open: isNotebookOpen(row.notebook.id) }"
                  aria-hidden="true"
                />
              </button>
              <span v-else class="note-tree-chevron-spacer" aria-hidden="true" />
              <button
                type="button"
                class="note-tree-name"
                @click="selectNotebookFilter(row.notebook.id)"
              >
                <span
                  v-if="row.notebook.isPinned"
                  class="note-pin-mark"
                  title="已置顶"
                >📌</span>
                {{ row.notebook.name }}
              </button>
            </div>
          </div>
          <div
            v-if="selectedNotebookId"
            class="note-tree-actions"
          >
            <button
              type="button"
              class="btn btn-sm"
              :disabled="loading"
              @click="renameSelectedNotebook"
            >
              重命名
            </button>
            <button
              type="button"
              class="btn btn-sm"
              :disabled="loading"
              @click="toggleNotebookPin"
            >
              置顶
            </button>
            <button
              type="button"
              class="btn btn-sm btn-danger"
              :disabled="loading"
              @click="deleteSelectedNotebook"
            >
              删除
            </button>
          </div>
        </div>
      </div>

      <div class="panel note-list">
        <div class="panel-h note-list-header">
          <span>笔记列表 · {{ filterLabel }}</span>
        </div>
        <div class="panel-body note-list-panel-body">
          <div class="note-list-toolbar">
            <input
              v-model="searchQuery"
              type="search"
              class="field-control note-search-input"
              placeholder="搜索标题 / 正文"
              aria-label="搜索笔记"
              @keydown.enter="onSearchInput"
            />
            <button
              type="button"
              class="btn btn-sm"
              :disabled="loading || listLoading"
              @click="startNewNote"
            >
              新建笔记
            </button>
          </div>
          <p v-if="listLoading && noteList.length === 0" class="note-list-empty">
            加载中…
          </p>
          <p v-else-if="noteList.length === 0" class="note-list-empty">
            暂无笔记，点击「新建笔记」开始记录。
          </p>
          <ul v-else class="note-list-items" aria-label="笔记列表">
            <li v-for="item in noteList" :key="item.id">
              <button
                type="button"
                class="note-list-item"
                :class="{ 'is-selected': selectedNoteId === item.id }"
                @click="selectNote(item.id)"
              >
                <span class="note-list-item-title">
                  <span
                    v-if="item.isPinned"
                    class="note-pin-mark"
                    title="置顶"
                  >📌</span>
                  {{ noteDisplayTitle(item.title) }}
                </span>
                <span v-if="item.preview" class="note-list-item-preview">
                  {{ item.preview }}
                </span>
                <span v-if="item.updatedAt" class="note-list-item-time">
                  {{ formatNoteUpdatedAt(item.updatedAt) }}
                </span>
              </button>
            </li>
          </ul>
        </div>
      </div>

      <div class="panel note-editor">
        <div class="panel-h note-editor-header">
          <span>编辑器</span>
          <span v-if="selectedNoteId" class="note-editor-actions">
            <label class="note-pin-toggle">
              <input
                v-model="isPinned"
                type="checkbox"
              />
              置顶
            </label>
            <span v-if="saving" class="note-save-hint">保存中…</span>
            <span v-else-if="dirty" class="note-save-hint">未保存</span>
            <span v-else-if="updatedAt" class="note-save-hint">
              已保存 · {{ formatNoteUpdatedAt(updatedAt) }}
            </span>
            <button
              type="button"
              class="btn btn-sm btn-danger"
              :disabled="loading || saving"
              @click="removeCurrentNote"
            >
              删除
            </button>
          </span>
        </div>
        <div class="panel-body note-editor-body">
          <template v-if="selectedNoteId">
            <input
              v-model="title"
              type="text"
              class="field-control note-title-input"
              placeholder="标题（留空显示为「无标题」）"
              aria-label="笔记标题"
            />
            <textarea
              v-model="body"
              class="field-control note-body-input"
              placeholder="正文…"
              aria-label="笔记正文"
            />

            <section class="note-meta-section" aria-label="标签">
              <div class="note-meta-head">标签</div>
              <div class="note-tag-row">
                <span
                  v-for="tag in selectedTags"
                  :key="tag.id"
                  class="note-tag-chip"
                >
                  {{ tag.name }}
                  <button
                    type="button"
                    class="note-tag-remove"
                    :aria-label="`移除标签 ${tag.name}`"
                    @click="removeTag(tag.id)"
                  >
                    ×
                  </button>
                </span>
                <input
                  v-model="newTagInput"
                  type="text"
                  class="field-control note-tag-input"
                  placeholder="输入标签名回车添加"
                  maxlength="32"
                  @keydown.enter.prevent="addTagFromInput"
                />
              </div>
            </section>

            <section class="note-meta-section" aria-label="附件">
              <div class="note-meta-head">
                <span>附件</span>
                <button
                  type="button"
                  class="btn btn-sm"
                  @click="addAttachmentPath"
                >
                  添加路径
                </button>
              </div>
              <p v-if="attachments.length === 0" class="note-attach-empty">
                暂无附件 · 首期仅存本地文件路径
              </p>
              <ul v-else class="note-attach-list">
                <li
                  v-for="att in attachments"
                  :key="att.id"
                  class="note-attach-item"
                >
                  <span
                    class="note-attach-name"
                    :title="att.filePath"
                  >
                    {{ att.displayName ?? att.filePath }}
                  </span>
                  <button
                    type="button"
                    class="ledger-link-btn danger"
                    @click="removeAttachment(att.id)"
                  >
                    移除
                  </button>
                </li>
              </ul>
            </section>
          </template>
          <div v-else class="placeholder note-editor-empty">
            选择左侧笔记或新建后开始编辑 · 自动记录创建/修改时间
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
