<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { appearance } from '../theme';
import {
  ledgerCreateCategory,
  ledgerDeleteCategory,
  ledgerListCategories,
  type LedgerCategoryDto,
} from '../services/ledger';

const categoryInput = ref('');
const categories = ref<LedgerCategoryDto[]>([]);
const loading = ref(false);

async function refreshCategories() {
  loading.value = true;
  try {
    categories.value = await ledgerListCategories();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  } finally {
    loading.value = false;
  }
}

async function addCategory() {
  const name = categoryInput.value.trim();
  if (!name) return;
  try {
    await ledgerCreateCategory({ name });
    categoryInput.value = '';
    await refreshCategories();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function removeCategory(row: LedgerCategoryDto) {
  try {
    await ledgerDeleteCategory(row.id);
    await refreshCategories();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

onMounted(() => {
  refreshCategories();
});
</script>

<template>
  <div class="view-root">
    <div class="settings-page">
      <div class="panel">
        <div class="panel-h">通用</div>
        <div class="panel-body settings-panel-body--flush">
          <div class="setting-row">
            <span class="setting-row-label">外观</span>
            <n-radio-group v-model:value="appearance" size="small">
              <n-radio-button value="light">浅色</n-radio-button>
              <n-radio-button value="dark">深色</n-radio-button>
            </n-radio-group>
          </div>
          <div class="setting-row">
            <span class="setting-row-label">数据目录</span>
            <span class="setting-placeholder">待接入</span>
          </div>
        </div>
      </div>
      <div class="panel">
        <div class="panel-h">记账 · 自定义分类</div>
        <div class="panel-body settings-categories-body">
          <p class="ledger-field-note settings-categories-intro">
            在此添加或删除分类；与「记账 · 新增账单 / 账单列表 / 统计视图」共用数据库。
          </p>
          <div class="category-editor-toolbar">
            <input
              v-model="categoryInput"
              type="text"
              class="field-control category-input"
              placeholder="输入新分类名称"
              maxlength="32"
              autocomplete="off"
              :disabled="loading"
              @keydown.enter.prevent="addCategory"
            />
            <button
              type="button"
              class="btn"
              :disabled="loading"
              @click="addCategory"
            >
              添加
            </button>
          </div>
          <p v-if="loading" class="ledger-field-note">加载中…</p>
          <ul v-else class="category-list" aria-label="已保存的分类列表">
            <li
              v-for="c in categories"
              :key="c.id"
              class="category-list-item"
            >
              <span class="category-list-name">{{ c.name }}</span>
              <button
                type="button"
                class="ledger-link-btn danger"
                @click="removeCategory(c)"
              >
                删除
              </button>
            </li>
          </ul>
        </div>
      </div>
    </div>
  </div>
</template>
