<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { localYmd } from '../../utils/ledgerDates';
import {
  ledgerCreateTransaction,
  ledgerGetTransaction,
  ledgerListCategories,
  ledgerUpdateTransaction,
  type LedgerCategoryDto,
} from '../../services/ledger';

const route = useRoute();
const router = useRouter();

const categories = ref<LedgerCategoryDto[]>([]);
const loading = ref(false);
const saving = ref(false);

const editId = computed(() => {
  const raw = route.query.id;
  return typeof raw === 'string' && raw.length > 0 ? raw : null;
});

const isEdit = computed(() => editId.value != null);

const amount = ref<string | number>('');
const currency = ref<'cny' | 'jpy'>('cny');
const direction = ref<'in' | 'out'>('out');
const categoryId = ref('');
const occurredOn = ref('');
const note = ref('');

function defaultToday() {
  occurredOn.value = localYmd(new Date());
}

async function loadCategories() {
  try {
    categories.value = await ledgerListCategories();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function loadTxIfEdit() {
  const id = editId.value;
  if (!id) {
    amount.value = '';
    currency.value = 'cny';
    direction.value = 'out';
    categoryId.value = '';
    note.value = '';
    defaultToday();
    return;
  }
  loading.value = true;
  try {
    const tx = await ledgerGetTransaction(id);
    if (!tx) {
      window.alert('记录不存在');
      router.replace({ name: 'ledger-detail', query: {} });
      defaultToday();
      return;
    }
    amount.value = tx.amount;
    currency.value = tx.currency.toLowerCase() === 'jpy' ? 'jpy' : 'cny';
    direction.value = tx.direction === 'in' ? 'in' : 'out';
    categoryId.value = tx.categoryId ?? '';
    occurredOn.value = tx.occurredOn;
    note.value = tx.note ?? '';
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}

function amountAsString(): string {
  const v = amount.value;
  if (v === '' || v === null || v === undefined) return '';
  return String(v).trim();
}

async function save() {
  saving.value = true;
  try {
    const cat =
      categoryId.value && categoryId.value.length > 0
        ? categoryId.value
        : null;
    const payloadBase = {
      amount: amountAsString(),
      currency: currency.value,
      direction: direction.value,
      categoryId: cat,
      occurredOn: occurredOn.value,
      note: note.value.trim() ? note.value.trim() : null,
    };
    if (isEdit.value && editId.value) {
      await ledgerUpdateTransaction({
        id: editId.value,
        ...payloadBase,
      });
    } else {
      await ledgerCreateTransaction(payloadBase);
    }
    router.push({ name: 'ledger-list' });
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  } finally {
    saving.value = false;
  }
}

function cancel() {
  router.push({ name: 'ledger-list' });
}

onMounted(async () => {
  await loadCategories();
  await loadTxIfEdit();
});

watch(
  () => route.query.id,
  async () => {
    await loadTxIfEdit();
  },
);
</script>

<template>
  <div class="view-root">
    <div class="layout-ledger">
      <div class="ledger-detail-root panel">
        <div class="panel-h">
          {{ isEdit ? '单笔流水 · 编辑' : '单笔流水 · 新建' }}
        </div>
        <div class="panel-body">
          <p v-if="loading" class="ledger-field-note">加载中…</p>
          <div
            v-else
            class="ledger-detail-grid ledger-detail-grid--form"
          >
            <span class="field-label">金额</span>
            <input
              v-model="amount"
              class="field-control"
              type="number"
              step="0.01"
              inputmode="decimal"
              placeholder="0.00"
            />
            <span class="field-label">币种</span>
            <div>
              <select v-model="currency" class="field-control" aria-label="币种">
                <option value="cny">人民币（CNY）</option>
                <option value="jpy">日元（JPY）</option>
              </select>
              <p class="ledger-field-note">仅支持人民币与日元。</p>
            </div>
            <span class="field-label">收支</span>
            <select
              v-model="direction"
              class="field-control"
              aria-label="收支"
            >
              <option value="out">支出</option>
              <option value="in">收入</option>
            </select>
            <span class="field-label">分类</span>
            <div>
              <select
                v-model="categoryId"
                class="field-control"
                aria-label="分类"
              >
                <option value="">未分类</option>
                <option
                  v-for="c in categories"
                  :key="c.id"
                  :value="c.id"
                >
                  {{ c.name }}
                </option>
              </select>
              <p class="ledger-field-note">
                分类在「设置」中维护。
              </p>
            </div>
            <span class="field-label">发生日期</span>
            <div>
              <input
                v-model="occurredOn"
                class="field-control"
                type="date"
                aria-label="发生日期"
              />
              <p class="ledger-field-note">新建时默认为今天。</p>
            </div>
            <span class="field-label">备注</span>
            <textarea
              v-model="note"
              class="field-control"
              placeholder="选填"
            />
            <div class="ledger-detail-actions">
              <button
                type="button"
                class="btn"
                :disabled="saving"
                @click="save"
              >
                保存
              </button>
              <button type="button" class="btn" @click="cancel">
                取消
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
