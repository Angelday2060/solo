import {
  createRouter,
  createWebHistory,
  type RouteRecordRaw,
} from 'vue-router';

const routes: RouteRecordRaw[] = [
  { path: '/', redirect: '/diary' },
  {
    path: '/diary',
    name: 'diary',
    component: () => import('../views/DiaryView.vue'),
    meta: { title: '日记' },
  },
  {
    path: '/ledger/stat',
    name: 'ledger-stat',
    component: () => import('../views/ledger/LedgerStatView.vue'),
    meta: { title: '记账 · 统计视图' },
  },
  {
    path: '/ledger/list',
    name: 'ledger-list',
    component: () => import('../views/ledger/LedgerListView.vue'),
    meta: { title: '记账 · 账单列表' },
  },
  {
    path: '/ledger/detail',
    name: 'ledger-detail',
    component: () => import('../views/ledger/LedgerDetailView.vue'),
    meta: { title: '记账 · 新增账单' },
  },
  {
    path: '/notes',
    name: 'notes',
    component: () => import('../views/NotesView.vue'),
    meta: { title: '笔记' },
  },
  {
    path: '/schedule',
    name: 'schedule',
    component: () => import('../views/ScheduleView.vue'),
    meta: { title: '日程' },
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('../views/SettingsView.vue'),
    meta: { title: '设置' },
  },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});
