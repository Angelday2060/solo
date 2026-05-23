<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import {
  diaryCreateEntry,
  diaryDeleteEntry,
  diaryGetEntry,
  diaryListEntries,
  diaryUpdateEntry,
  MOOD_OPTIONS,
  WEATHER_OPTIONS,
  type DiaryEntryDto,
  type DiaryEntryListItem,
} from '../services/diary';
import {
  daySubLabel,
  diaryEditorFullLabel,
  diaryTodayIso,
  getNewestDiaryEntry,
  groupDiaryEntriesDesc,
  parseIsoDateLocal,
} from '../utils/diaryDates';

const entries = ref<DiaryEntryListItem[]>([]);
const loading = ref(false);
const saving = ref(false);
const selectedId = ref<string | null>(null);

const openYears = ref<Set<number>>(new Set());
const openMonths = ref<Set<string>>(new Set());

const title = ref('');
const body = ref('');
const entryDate = ref('');
const weather = ref('');
const mood = ref('');
const updatedAt = ref<string | null>(null);

const dirty = ref(false);
const skipDirtyWatch = ref(false);
let saveTimer: ReturnType<typeof setTimeout> | null = null;

const tree = computed(() => groupDiaryEntriesDesc(entries.value));
const newestEntry = computed(() => getNewestDiaryEntry(entries.value));

const editorDateLabel = computed(() =>
  entryDate.value ? diaryEditorFullLabel(entryDate.value) : '—',
);

function monthKey(year: number, month: number): string {
  return `${year}-${month}`;
}

function isYearOpen(year: number): boolean {
  return openYears.value.has(year);
}

function isMonthOpen(year: number, month: number): boolean {
  return openMonths.value.has(monthKey(year, month));
}

function toggleYear(year: number) {
  const next = new Set(openYears.value);
  if (next.has(year)) next.delete(year);
  else next.add(year);
  openYears.value = next;
}

function toggleMonth(year: number, month: number) {
  const key = monthKey(year, month);
  const next = new Set(openMonths.value);
  if (next.has(key)) next.delete(key);
  else next.add(key);
  openMonths.value = next;
}

function expandToEntry(iso: string) {
  const p = parseIsoDateLocal(iso);
  if (!p) return;
  const nextYears = new Set(openYears.value);
  nextYears.add(p.y);
  openYears.value = nextYears;
  const nextMonths = new Set(openMonths.value);
  nextMonths.add(monthKey(p.y, p.m));
  openMonths.value = nextMonths;
}

function ensureTreeExpandedForNewest() {
  const e = newestEntry.value;
  if (e) expandToEntry(e.entryDate);
}

async function refreshList(selectId?: string | null) {
  loading.value = true;
  try {
    entries.value = await diaryListEntries();
    ensureTreeExpandedForNewest();
    const target =
      selectId ??
      selectedId.value ??
      newestEntry.value?.id ??
      null;
    if (target && entries.value.some((x) => x.id === target)) {
      await selectEntry(target, { skipSave: true });
    } else if (entries.value.length === 0) {
      selectedId.value = null;
      clearEditor();
    } else if (newestEntry.value) {
      await selectEntry(newestEntry.value.id, { skipSave: true });
    }
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}

function clearEditor() {
  skipDirtyWatch.value = true;
  title.value = '';
  body.value = '';
  entryDate.value = '';
  weather.value = '';
  mood.value = '';
  updatedAt.value = null;
  dirty.value = false;
  skipDirtyWatch.value = false;
}

function applyEntryToForm(entry: DiaryEntryDto) {
  skipDirtyWatch.value = true;
  title.value = entry.title ?? '';
  body.value = entry.body ?? '';
  entryDate.value = entry.entryDate;
  weather.value = entry.weather ?? '';
  mood.value = entry.mood ?? '';
  updatedAt.value = entry.updatedAt;
  dirty.value = false;
  skipDirtyWatch.value = false;
  expandToEntry(entry.entryDate);
}

async function flushSave(): Promise<boolean> {
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
  }
  if (!dirty.value || !selectedId.value || saving.value) {
    return true;
  }
  const savingId = selectedId.value;
  saving.value = true;
  try {
    const updated = await diaryUpdateEntry({
      id: savingId,
      entryDate: entryDate.value,
      title: title.value.trim() || null,
      body: body.value,
      weather: weather.value || null,
      mood: mood.value || null,
    });
    applyEntryToForm(updated);
    entries.value = await diaryListEntries();
    expandToEntry(updated.entryDate);
    return true;
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
    return false;
  } finally {
    saving.value = false;
  }
}

function scheduleSave() {
  if (skipDirtyWatch.value || !selectedId.value) return;
  dirty.value = true;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => {
    void flushSave();
  }, 700);
}

async function selectEntry(
  id: string,
  opts: { skipSave?: boolean } = {},
) {
  if (!opts.skipSave && selectedId.value && selectedId.value !== id) {
    const ok = await flushSave();
    if (!ok) return;
  }
  loading.value = true;
  try {
    const entry = await diaryGetEntry(id);
    if (!entry) {
      window.alert('日记不存在');
      await refreshList();
      return;
    }
    selectedId.value = id;
    applyEntryToForm(entry);
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}

async function startNewDiary() {
  if (selectedId.value) {
    const ok = await flushSave();
    if (!ok) return;
  }
  try {
    const created = await diaryCreateEntry({ entryDate: diaryTodayIso() });
    await refreshList(created.id);
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function removeCurrent() {
  const id = selectedId.value;
  if (!id) return;
  if (!window.confirm('确定删除这篇日记？')) return;
  if (saveTimer) {
    clearTimeout(saveTimer);
    saveTimer = null;
  }
  dirty.value = false;
  try {
    await diaryDeleteEntry(id);
    selectedId.value = null;
    clearEditor();
    await refreshList();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

watch([title, body, entryDate, weather, mood], () => {
  if (skipDirtyWatch.value) return;
  scheduleSave();
});

onMounted(() => {
  refreshList();
});
</script>

<template>
  <div class="view-root">
    <div class="layout-diary">
      <div class="panel date-list">
        <div class="panel-h">时间轴 · 年 / 月 / 日</div>
        <div class="panel-body diary-tree-panel-body">
          <div class="diary-tree-toolbar">
            <button
              type="button"
              class="btn btn-sm"
              :disabled="loading || saving"
              @click="startNewDiary"
            >
              新建日记
            </button>
            <span
              class="diary-toolbar-hint"
              title="写入 DiaryEntry · 可与已有条目同日多篇"
            >
              今日草稿 · 可与已有条目同日多篇
            </span>
          </div>
          <p v-if="loading && entries.length === 0" class="diary-tree-empty">
            加载中…
          </p>
          <p v-else-if="entries.length === 0" class="diary-tree-empty">
            暂无日记，点击「新建日记」开始记录。
          </p>
          <div
            v-else
            class="diary-tree"
            role="tree"
            aria-label="日记日期树"
          >
            <div
              v-for="yearNode in tree"
              :key="yearNode.year"
              class="diary-tree-year diary-tree-node"
              :class="{ open: isYearOpen(yearNode.year) }"
              role="treeitem"
              :aria-expanded="isYearOpen(yearNode.year)"
            >
              <button
                type="button"
                class="diary-tree-toggle"
                :aria-expanded="isYearOpen(yearNode.year)"
                @click="toggleYear(yearNode.year)"
              >
                <span class="diary-tree-chevron" aria-hidden="true" />
                <span class="diary-tree-label">{{ yearNode.year }}年</span>
              </button>
              <div class="diary-tree-children" role="group">
                <div
                  v-for="monthNode in yearNode.months"
                  :key="monthKey(yearNode.year, monthNode.month)"
                  class="diary-tree-month diary-tree-node"
                  :class="{ open: isMonthOpen(yearNode.year, monthNode.month) }"
                  role="treeitem"
                  :aria-expanded="isMonthOpen(yearNode.year, monthNode.month)"
                >
                  <button
                    type="button"
                    class="diary-tree-toggle"
                    :aria-expanded="isMonthOpen(yearNode.year, monthNode.month)"
                    @click="toggleMonth(yearNode.year, monthNode.month)"
                  >
                    <span class="diary-tree-chevron" aria-hidden="true" />
                    <span class="diary-tree-label">{{ monthNode.month }}月</span>
                  </button>
                  <div class="diary-tree-children" role="group">
                    <div class="diary-tree-days">
                      <button
                        v-for="row in monthNode.days"
                        :key="row.id"
                        type="button"
                        class="diary-tree-day"
                        :class="{ 'is-selected': selectedId === row.id }"
                        role="treeitem"
                        @click="selectEntry(row.id)"
                      >
                        <span class="diary-tree-day-main">
                          {{
                            parseIsoDateLocal(row.entryDate)?.d ?? row.entryDate
                          }}日
                        </span>
                        <span class="diary-tree-day-sub">{{ daySubLabel(row) }}</span>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="panel editor-main">
        <div class="panel-h diary-editor-header">
          <span>
            正文区 ·
            <span class="diary-editor-date-label">{{ editorDateLabel }}</span>
          </span>
          <span v-if="selectedId" class="diary-editor-actions">
            <span v-if="saving" class="diary-save-hint">保存中…</span>
            <span v-else-if="dirty" class="diary-save-hint">未保存</span>
            <span v-else-if="updatedAt" class="diary-save-hint">
              已保存 · {{ updatedAt.slice(0, 16).replace('T', ' ') }}
            </span>
            <button
              type="button"
              class="btn btn-sm btn-danger"
              :disabled="loading || saving"
              @click="removeCurrent"
            >
              删除
            </button>
          </span>
        </div>
        <div class="panel-body diary-editor-body">
          <template v-if="selectedId">
            <input
              v-model="title"
              type="text"
              class="field-control diary-title-input"
              placeholder="标题（留空显示为「无标题」）"
              aria-label="日记标题"
            />
            <textarea
              v-model="body"
              class="field-control diary-body-input"
              placeholder="写下今天的事…"
              aria-label="日记正文"
            />
          </template>
          <div v-else class="placeholder diary-editor-empty">
            选择左侧条目或新建日记后开始编辑 · 自动记录创建/修改时间
          </div>
        </div>
      </div>

      <div class="meta-col">
        <div class="panel">
          <div class="panel-h">元数据 · 日期 / 天气 / 心情</div>
          <div class="panel-body diary-meta-body">
            <p class="diary-meta-hint">
              是否在编辑区展示以上字段，可在「设置」中逐项开关（原型仅示意）。
            </p>
            <div class="diary-meta-field">
              <label class="diary-meta-label" for="diary-meta-date">日期</label>
              <input
                id="diary-meta-date"
                v-model="entryDate"
                class="diary-meta-control field-control"
                type="date"
                aria-label="日记日期"
                :disabled="!selectedId"
              />
            </div>
            <div class="diary-meta-field">
              <label class="diary-meta-label" for="diary-meta-weather">天气</label>
              <select
                id="diary-meta-weather"
                v-model="weather"
                class="diary-meta-control field-control"
                aria-label="天气"
                :disabled="!selectedId"
              >
                <option
                  v-for="opt in WEATHER_OPTIONS"
                  :key="opt.value || 'empty'"
                  :value="opt.value"
                >
                  {{ opt.label }}
                </option>
              </select>
            </div>
            <div class="diary-meta-field">
              <label class="diary-meta-label" for="diary-meta-mood">心情</label>
              <select
                id="diary-meta-mood"
                v-model="mood"
                class="diary-meta-control field-control"
                aria-label="心情"
                :disabled="!selectedId"
              >
                <option
                  v-for="opt in MOOD_OPTIONS"
                  :key="opt.value || 'empty'"
                  :value="opt.value"
                >
                  {{ opt.label }}
                </option>
              </select>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
