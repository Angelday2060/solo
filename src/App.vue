<script setup lang="ts">
import type { GlobalTheme } from 'naive-ui';
import { darkTheme, zhCN, dateZhCN } from 'naive-ui';
import { computed, onMounted, ref, watch } from 'vue';
import { RouterLink, RouterView, useRoute, useRouter } from 'vue-router';
import { appearance } from './theme';
import { fetchDbHealth } from './services/db';

const route = useRoute();
const router = useRouter();

const naiveTheme = computed<GlobalTheme | null>(() =>
  appearance.value === 'dark' ? darkTheme : null,
);

const pageTitle = computed(() => (route.meta.title as string) ?? 'solo');

const isLedgerRoute = computed(() => route.path.startsWith('/ledger'));

const ledgerExpanded = ref(false);

watch(
  isLedgerRoute,
  (on) => {
    if (on) ledgerExpanded.value = true;
  },
  { immediate: true },
);

function toggleLedgerNav() {
  ledgerExpanded.value = !ledgerExpanded.value;
}

function goSettings() {
  router.push({ name: 'settings' });
}

onMounted(() => {
  fetchDbHealth()
    .then((h) => {
      if (import.meta.env.DEV) console.debug('[solo db]', h.dbPath, h.sqliteVersion);
    })
    .catch(() => {});
});
</script>

<template>
  <n-config-provider
    :theme="naiveTheme"
    :locale="zhCN"
    :date-locale="dateZhCN"
  >
    <div class="app">
      <aside class="sidebar" aria-label="主导航">
        <div class="brand">solo</div>
        <nav class="nav">
          <RouterLink to="/diary" class="nav-item" active-class="active">
            日记
          </RouterLink>
          <div
            class="nav-ledger-group"
            :class="{ active: isLedgerRoute, open: ledgerExpanded }"
          >
            <button
              type="button"
              class="nav-ledger-toggle"
              :aria-expanded="ledgerExpanded"
              aria-controls="nav-ledger-subnav"
              @click="toggleLedgerNav"
            >
              <span class="nav-ledger-chevron" aria-hidden="true" />
              <span class="nav-ledger-title">记账</span>
            </button>
            <div
              id="nav-ledger-subnav"
              v-show="ledgerExpanded"
              class="nav-ledger-children"
              role="group"
              aria-label="记账子页面"
            >
              <RouterLink
                to="/ledger/stat"
                class="nav-item nav-subitem"
                active-class="active"
              >
                统计视图
              </RouterLink>
              <RouterLink
                to="/ledger/list"
                class="nav-item nav-subitem"
                active-class="active"
              >
                账单列表
              </RouterLink>
              <RouterLink
                to="/ledger/detail"
                class="nav-item nav-subitem"
                active-class="active"
              >
                新增账单
              </RouterLink>
            </div>
          </div>
          <RouterLink to="/notes" class="nav-item" active-class="active">
            笔记
          </RouterLink>
          <RouterLink to="/schedule" class="nav-item" active-class="active">
            日程
          </RouterLink>
        </nav>
      </aside>
      <div class="main">
        <header class="topbar">
          <h1>{{ pageTitle }}</h1>
          <div class="topbar-end">
            <button type="button" class="btn" @click="goSettings">
              设置
            </button>
          </div>
        </header>
        <div class="workspace">
          <RouterView />
        </div>
      </div>
    </div>
  </n-config-provider>
</template>
