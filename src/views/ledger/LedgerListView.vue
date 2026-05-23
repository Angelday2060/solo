<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import {
  ledgerDeleteTransactions,
  ledgerListCategories,
  ledgerListTransactions,
  type LedgerCategoryDto,
  type LedgerListFilter,
  type LedgerTransactionDto,
} from '../../services/ledger';

const router = useRouter();

const loading = ref(false);
const rows = ref<LedgerTransactionDto[]>([]);
const categories = ref<LedgerCategoryDto[]>([]);

const searchText = ref('');
const filterDateFrom = ref('');
const filterDateTo = ref('');
const filterDirection = ref<'all' | 'in' | 'out'>('all');
const filterCategoryId = ref('all');
const filterCurrency = ref<'all' | 'cny' | 'jpy'>('all');

const selectedIds = ref<Set<string>>(new Set());

const allSelected = computed(() => {
  if (rows.value.length === 0) return false;
  return rows.value.every((r) => selectedIds.value.has(r.id));
});

function buildFilter(): LedgerListFilter {
  const f: LedgerListFilter = {
    direction: filterDirection.value,
    currency: filterCurrency.value,
    categoryId: filterCategoryId.value,
    search: searchText.value.trim() || undefined,
  };
  if (filterDateFrom.value) f.dateFrom = filterDateFrom.value;
  if (filterDateTo.value) f.dateTo = filterDateTo.value;
  return f;
}

async function loadCategories() {
  try {
    categories.value = await ledgerListCategories();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function loadTransactions() {
  loading.value = true;
  try {
    rows.value = await ledgerListTransactions(buildFilter());
    selectedIds.value = new Set();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}

function toggleSelectAll(ev: Event) {
  const ck = ev.target as HTMLInputElement;
  if (ck.checked) {
    selectedIds.value = new Set(rows.value.map((r) => r.id));
  } else {
    selectedIds.value = new Set();
  }
}

function toggleRow(id: string, ev: Event) {
  const ck = ev.target as HTMLInputElement;
  const next = new Set(selectedIds.value);
  if (ck.checked) next.add(id);
  else next.delete(id);
  selectedIds.value = next;
}

function rowChecked(id: string) {
  return selectedIds.value.has(id);
}

function dirLabel(d: string) {
  return d === 'in' ? '收入' : '支出';
}

function curLabel(c: string) {
  return c.toLowerCase() === 'jpy' ? 'JPY' : 'CNY';
}

function goNew() {
  router.push({ name: 'ledger-detail', query: {} });
}

function goEdit(id: string) {
  router.push({ name: 'ledger-detail', query: { id } });
}

async function deleteOne(id: string) {
  if (!window.confirm('确定删除这条流水？')) return;
  try {
    await ledgerDeleteTransactions([id]);
    await loadTransactions();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function batchDelete() {
  const ids = [...selectedIds.value];
  if (ids.length === 0) return;
  if (!window.confirm(`确定删除选中的 ${ids.length} 条流水？`)) return;
  try {
    await ledgerDeleteTransactions(ids);
    await loadTransactions();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

onMounted(async () => {
  await loadCategories();
  await loadTransactions();
});

watch(
  [filterDirection, filterCategoryId, filterCurrency],
  () => {
    loadTransactions();
  },
);
</script>

<template>
  <div class="view-root">
    <div class="layout-ledger">
      <div class="ledger-table-root panel ledger-table">
        <div class="panel-h">
          流水列表 · 币种 CNY/JPY · 检索与筛选下方 · 新建在「新增账单」页
        </div>
        <div class="panel-body">
          <div class="ledger-list-filters">
            <input
              v-model="searchText"
              type="search"
              class="field-control ledger-filter-search"
              placeholder="备注或金额关键字"
              aria-label="搜索"
              @keydown.enter.prevent="loadTransactions"
            />
            <input
              v-model="filterDateFrom"
              type="date"
              class="field-control ledger-filter-date"
              aria-label="起始日"
            />
            <span class="ledger-filter-sep">—</span>
            <input
              v-model="filterDateTo"
              type="date"
              class="field-control ledger-filter-date"
              aria-label="结束日"
            />
            <select
              v-model="filterDirection"
              class="field-control ledger-filter-select"
              aria-label="方向"
            >
              <option value="all">全部方向</option>
              <option value="out">支出</option>
              <option value="in">收入</option>
            </select>
            <select
              v-model="filterCategoryId"
              class="field-control ledger-filter-select"
              aria-label="分类"
            >
              <option value="all">全部分类</option>
              <option
                v-for="c in categories"
                :key="c.id"
                :value="c.id"
              >
                {{ c.name }}
              </option>
            </select>
            <select
              v-model="filterCurrency"
              class="field-control ledger-filter-select"
              aria-label="币种"
            >
              <option value="all">全部币种</option>
              <option value="cny">CNY</option>
              <option value="jpy">JPY</option>
            </select>
            <button type="button" class="btn" @click="loadTransactions">
              检索
            </button>
            <button type="button" class="btn" @click="goNew">新建</button>
          </div>
          <div class="ledger-list-toolbar">
            <button
              type="button"
              class="btn"
              :disabled="selectedIds.size === 0"
              @click="batchDelete"
            >
              删除所选
            </button>
            <p class="ledger-field-note ledger-list-toolbar-hint">
              勾选后可批量删除；编辑跳转「记账 · 新增账单」。
            </p>
          </div>
          <p v-if="loading" class="ledger-field-note">加载中…</p>
          <table v-else class="ledger">
            <thead>
              <tr>
                <th class="ledger-col-check">
                  <input
                    type="checkbox"
                    aria-label="全选本页"
                    :checked="allSelected"
                    @change="toggleSelectAll"
                  />
                </th>
                <th>金额</th>
                <th>币种</th>
                <th>方向</th>
                <th>分类</th>
                <th>发生日</th>
                <th>备注</th>
                <th class="ledger-col-actions">操作</th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="rows.length === 0">
                <td colspan="8" class="ledger-empty-cell">
                  暂无数据，可在「新增账单」中新建，或调整筛选条件。
                </td>
              </tr>
              <tr v-for="r in rows" :key="r.id">
                <td class="ledger-col-check">
                  <input
                    type="checkbox"
                    :aria-label="`选择 ${r.id}`"
                    :checked="rowChecked(r.id)"
                    @change="toggleRow(r.id, $event)"
                  />
                </td>
                <td>{{ r.amount }}</td>
                <td>{{ curLabel(r.currency) }}</td>
                <td>{{ dirLabel(r.direction) }}</td>
                <td>{{ r.categoryName ?? '—' }}</td>
                <td>{{ r.occurredOn }}</td>
                <td>{{ r.note ?? '—' }}</td>
                <td class="ledger-col-actions">
                  <button
                    type="button"
                    class="btn btn-sm"
                    @click="goEdit(r.id)"
                  >
                    编辑
                  </button>
                  <button
                    type="button"
                    class="btn btn-sm btn-danger"
                    @click="deleteOne(r.id)"
                  >
                    删除
                  </button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>
  </div>
</template>
