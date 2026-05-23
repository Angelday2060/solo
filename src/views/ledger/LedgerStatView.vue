<script setup lang="ts">
import Chart from 'chart.js/auto';
import type {
  Chart as ChartJs,
  ChartConfiguration,
  Plugin,
  TooltipItem,
} from 'chart.js';
import { onBeforeUnmount, onMounted, ref } from 'vue';
import { localYmd, rangeForPreset, type StatPreset } from '../../utils/ledgerDates';
import {
  ledgerGetPeriodRollups,
  ledgerGetStatistics,
  ledgerListCategories,
  type LedgerCategoryDto,
  type LedgerRollupCard,
  type LedgerStatistics,
} from '../../services/ledger';

const dailyCanvas = ref<HTMLCanvasElement | null>(null);
const barCanvas = ref<HTMLCanvasElement | null>(null);
const pieCanvas = ref<HTMLCanvasElement | null>(null);

let charts: ChartJs[] = [];

const palette = [
  'rgba(255, 99, 132, 0.75)',
  'rgba(54, 162, 235, 0.75)',
  'rgba(255, 206, 86, 0.85)',
  'rgba(75, 192, 192, 0.75)',
  'rgba(153, 102, 255, 0.75)',
  'rgba(255, 159, 64, 0.75)',
  'rgba(199, 199, 199, 0.75)',
  'rgba(83, 102, 255, 0.75)',
  'rgba(255, 99, 255, 0.75)',
  'rgba(99, 255, 132, 0.75)',
  'rgba(255, 200, 100, 0.75)',
  'rgba(100, 200, 255, 0.75)',
];

const rollups = ref<LedgerRollupCard[]>([]);
const categories = ref<LedgerCategoryDto[]>([]);
const loadingRollup = ref(false);
const loadingCharts = ref(false);

const preset = ref<StatPreset>('month');
const customFrom = ref('');
const customTo = ref('');
const statDirection = ref<'all' | 'in' | 'out'>('all');
const statCategoryId = ref('all');
const statCurrency = ref<'cny' | 'jpy'>('cny');

function destroyCharts() {
  charts.forEach((c) => c.destroy());
  charts = [];
}

function initCustomRange() {
  const now = new Date();
  const from = new Date(now.getFullYear(), now.getMonth(), 1);
  customFrom.value = localYmd(from);
  customTo.value = localYmd(now);
}

function currentRange(): { from: string; to: string } {
  return rangeForPreset(preset.value, customFrom.value, customTo.value);
}

function formatLedgerAmount(n: number, currency: 'cny' | 'jpy'): string {
  const iso = currency === 'cny' ? 'CNY' : 'JPY';
  const maxFrac = currency === 'jpy' ? 0 : Number.isInteger(n) ? 0 : 2;
  return new Intl.NumberFormat('zh-CN', {
    style: 'currency',
    currency: iso,
    maximumFractionDigits: maxFrac,
  }).format(n);
}

function formatLedgerAxisLabel(n: number, currency: 'cny' | 'jpy'): string {
  const v = Number(n);
  if (currency === 'cny') {
    if (v >= 1e4)
      return `¥${(v / 1e4).toFixed(v % 1e4 === 0 ? 0 : 1)}万`;
    if (v >= 1e3)
      return `¥${(v / 1e3).toFixed(v % 1e3 === 0 ? 0 : 1)}k`;
    return `¥${Math.round(v)}`;
  }
  if (v >= 10000) return `¥${Math.round(v / 10000)}万`;
  if (v >= 1000) return `¥${Math.round(v / 1000)}千`;
  return `¥${Math.round(v)}`;
}

/** 横向条形图内侧右对齐金额（与原型一致） */
function createBarInnerLabelsPlugin(
  formatMoney: (n: number) => string,
): Plugin<'bar'> {
  return {
    id: 'ledgerBarInnerLabels',
    afterDatasetsDraw(chart) {
      const opts = chart.options;
      if ((chart.config as { type: string }).type !== 'bar' || opts.indexAxis !== 'y')
        return;
      const ctx = chart.ctx;
      const ds = chart.data.datasets[0];
      const meta = chart.getDatasetMeta(0);
      if (!ds?.data || !meta?.data?.length) return;
      meta.data.forEach((rawEl, i) => {
        const bar = rawEl as unknown as {
          x: number;
          y: number;
          base: number;
        };
        const value = Number(ds.data[i]);
        if (value == null || Number.isNaN(value)) return;
        const x0 = bar.base;
        const x1 = bar.x;
        if (
          x0 == null ||
          x1 == null ||
          Number.isNaN(x0) ||
          Number.isNaN(x1)
        ) {
          return;
        }
        const left = Math.min(x0, x1);
        const right = Math.max(x0, x1);
        const barW = right - left;
        if (barW < 4) return;
        const cy = bar.y;
        const text = formatMoney(value);
        const pad = 8;
        let fontSize = 12;
        let tw = 0;
        ctx.save();
        for (;;) {
          ctx.font = `600 ${fontSize}px system-ui, "Segoe UI", "Microsoft YaHei", sans-serif`;
          tw = ctx.measureText(text).width;
          if (tw + pad <= barW || fontSize <= 9) break;
          fontSize -= 1;
        }
        if (tw + pad > barW) {
          ctx.restore();
          return;
        }
        ctx.textAlign = 'right';
        ctx.textBaseline = 'middle';
        ctx.lineJoin = 'round';
        ctx.lineWidth = 3;
        ctx.strokeStyle = 'rgba(255,255,255,0.92)';
        ctx.fillStyle = '#1a1a1a';
        const tx = right - pad;
        ctx.strokeText(text, tx, cy);
        ctx.fillText(text, tx, cy);
        ctx.restore();
      });
    },
  };
}

/** 饼图扇区内金额 + 占比（与原型一致） */
function createPieInnerLabelsPlugin(
  formatMoney: (n: number) => string,
): Plugin<'pie'> {
  return {
    id: 'ledgerPieInnerLabels',
    afterDatasetsDraw(chart) {
      if ((chart.config as { type: string }).type !== 'pie') return;
      const ctx = chart.ctx;
      const ds = chart.data.datasets[0];
      const meta = chart.getDatasetMeta(0);
      if (!ds?.data || !meta?.data?.length) return;
      const pieTotal = (ds.data as number[]).reduce((s, x) => s + x, 0);
      if (!pieTotal) return;
      meta.data.forEach((arc, i) => {
        const value = Number(ds.data[i]);
        if (!value) return;
        const share = value / pieTotal;
        if (share < 0.04) return;
        const pos = arc.tooltipPosition(false);
        const lineAmount = formatMoney(value);
        const linePct = `${(share * 100).toFixed(1)}%`;
        ctx.save();
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.lineJoin = 'round';
        const yAmt = pos.y - 8;
        const yPct = pos.y + 8;
        ctx.font =
          '600 11px system-ui, "Segoe UI", "Microsoft YaHei", sans-serif';
        ctx.lineWidth = 3;
        ctx.strokeStyle = 'rgba(255,255,255,0.92)';
        ctx.fillStyle = '#1a1a1a';
        ctx.strokeText(lineAmount, pos.x, yAmt);
        ctx.fillText(lineAmount, pos.x, yAmt);
        ctx.font =
          '500 10px system-ui, "Segoe UI", "Microsoft YaHei", sans-serif';
        ctx.strokeText(linePct, pos.x, yPct);
        ctx.fillText(linePct, pos.x, yPct);
        ctx.restore();
      });
    },
  };
}

function pieSliceColor(barRgba: string): string {
  return barRgba.replace(/[\d.]+\)\s*$/, '0.88)');
}

function buildDailyConfig(
  stats: LedgerStatistics,
  currency: 'cny' | 'jpy',
): ChartConfiguration<'line'> {
  const fmt = (n: number) => formatLedgerAmount(n, currency);
  const labels = stats.daily.map((d) => {
    const parts = d.date.split('-');
    const m = Number(parts[1]);
    const day = Number(parts[2]);
    return `${m}/${day}`;
  });
  const sumOut = stats.daily.reduce((s, d) => s + d.outSum, 0);
  const nDays = stats.daily.length;
  const cur = currency.toUpperCase();

  const subtitleText = `支出合计 ${fmt(sumOut)} · 共 ${nDays} 个有数据日`;

  const datasets: ChartConfiguration<'line'>['data']['datasets'] = [
    {
      label: '支出',
      data: stats.daily.map((d) => d.outSum),
      borderColor: 'rgb(220, 90, 90)',
      backgroundColor: 'rgba(220, 90, 90, 0.08)',
      fill: true,
      tension: 0.25,
      pointRadius: 3,
      pointHoverRadius: 5,
    },
  ];

  return {
    type: 'line',
    data: { labels, datasets },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      interaction: { mode: 'index', intersect: false },
      plugins: {
        legend: { display: false },
        title: {
          display: true,
          align: 'start',
          text: `按日支出 · 当前币种 ${cur}`,
          font: { size: 13, weight: 600 },
          padding: { bottom: 4 },
        },
        subtitle: {
          display: true,
          align: 'start',
          text: subtitleText,
          color: '#666',
          font: { size: 11 },
          padding: { bottom: 8 },
        },
        tooltip: {
          callbacks: {
            title(items: TooltipItem<'line'>[]) {
              return items.length ? `日期 ${items[0]!.label}` : '';
            },
            label(ctx: TooltipItem<'line'>) {
              const v = ctx.parsed.y;
              if (v == null) return ctx.dataset.label ?? '';
              return ` ${ctx.dataset.label}：${fmt(v)}`;
            },
            footer(items: TooltipItem<'line'>[]) {
              if (!items.length) return [];
              const idx = items[0]!.dataIndex;
              const row = stats.daily[idx];
              if (!row) return [];
              if (idx > 0) {
                const prev = stats.daily[idx - 1]!;
                if (prev.outSum > 0) {
                  const mom =
                    ((row.outSum - prev.outSum) / prev.outSum) * 100;
                  return [`支出环比：${mom.toFixed(1)}%`];
                }
              }
              return [];
            },
          },
        },
      },
      scales: {
        y: {
          beginAtZero: true,
          title: { display: true, text: '金额' },
          ticks: {
            callback: (tickValue) =>
              formatLedgerAxisLabel(
                typeof tickValue === 'number' ? tickValue : Number(tickValue),
                currency,
              ),
          },
        },
      },
    },
  };
}

function renderCharts(stats: LedgerStatistics) {
  destroyCharts();
  Chart.defaults.font.family =
    'system-ui, "Segoe UI", "Microsoft YaHei", sans-serif';
  Chart.defaults.color = '#444';

  const cur = statCurrency.value;
  const fmt = (n: number) => formatLedgerAmount(n, cur);

  if (dailyCanvas.value) {
    charts.push(
      new Chart(
        dailyCanvas.value,
        buildDailyConfig(stats, cur),
      ),
    );
  }

  const cats = stats.categoryExpense;
  const barLabels = cats.map((c) => c.name);
  const barVals = cats.map((c) => c.total);
  const barColors = barLabels.map((_, i) => palette[i % palette.length]!);
  const barSum = barVals.reduce((s, v) => s + v, 0);
  const flowKind = statDirection.value === 'in' ? '收入' : '支出';
  const barInnerPlugin = createBarInnerLabelsPlugin(fmt);
  const pieInnerPlugin = createPieInnerLabelsPlugin(fmt);

  if (barCanvas.value && barLabels.length > 0) {
    charts.push(
      new Chart(barCanvas.value, {
        type: 'bar',
        plugins: [barInnerPlugin],
        data: {
          labels: barLabels,
          datasets: [
            {
              label:
                statDirection.value === 'in' ? '收入合计' : '支出合计',
              data: barVals,
              backgroundColor: barColors,
              borderWidth: 0,
            },
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          indexAxis: 'y',
          plugins: {
            legend: { display: false },
            title: {
              display: true,
              align: 'start',
              text: `分类排行 · ${flowKind}`,
              font: { size: 13, weight: 600 },
              padding: { bottom: 2 },
            },
            subtitle: {
              display: true,
              align: 'start',
              text: `榜内合计 ${fmt(barSum)} · ${barLabels.length} 个分类`,
              color: '#666',
              font: { size: 11 },
              padding: { bottom: 8 },
            },
            tooltip: {
              callbacks: {
                title(items) {
                  if (!items.length) return '';
                  const rank = items[0]!.dataIndex + 1;
                  return `第 ${rank} 名 · ${items[0]!.label}`;
                },
                label(ctx) {
                  const v =
                    typeof ctx.raw === 'number' ? ctx.raw : Number(ctx.raw);
                  const share = barSum
                    ? ((v / barSum) * 100).toFixed(1)
                    : '0';
                  return ` 金额：${fmt(v)}（占本榜 ${share}%）`;
                },
              },
            },
          },
          scales: {
            x: {
              beginAtZero: true,
              title: { display: true, text: '金额' },
              ticks: {
                callback: (tickValue) =>
                  formatLedgerAxisLabel(
                    typeof tickValue === 'number'
                      ? tickValue
                      : Number(tickValue),
                    cur,
                  ),
              },
            },
          },
        },
      }),
    );
  }

  if (pieCanvas.value && cats.length > 0) {
    const pieTotal = cats.reduce((s, c) => s + c.total, 0);
    charts.push(
      new Chart(pieCanvas.value, {
        type: 'pie',
        plugins: [pieInnerPlugin],
        data: {
          labels: barLabels,
          datasets: [
            {
              data: barVals,
              backgroundColor: barColors.map((c) => pieSliceColor(c)),
              borderWidth: 1,
              borderColor: '#fff',
            },
          ],
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            title: {
              display: true,
              align: 'start',
              text: `${flowKind}结构 · 分类占比（按金额）`,
              font: { size: 13, weight: 600 },
              padding: { bottom: 2 },
            },
            subtitle: {
              display: true,
              align: 'start',
              text: `分类合计 ${fmt(pieTotal)} · ${barLabels.length} 个分类`,
              color: '#666',
              font: { size: 11 },
              padding: { bottom: 4 },
            },
            legend: { position: 'right' },
            tooltip: {
              callbacks: {
                title(items) {
                  return items.length ? items[0]!.label : '';
                },
                label(ctx) {
                  const raw = ctx.raw;
                  const v = typeof raw === 'number' ? raw : Number(raw);
                  const pct = pieTotal
                    ? ((v / pieTotal) * 100).toFixed(1)
                    : '0';
                  return ` ${fmt(v)}（占合计 ${pct}%）`;
                },
              },
            },
          },
        },
      }),
    );
  }
}

async function loadRollups() {
  loadingRollup.value = true;
  try {
    const r = await ledgerGetPeriodRollups();
    rollups.value = r.cards;
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  } finally {
    loadingRollup.value = false;
  }
}

async function loadCategories() {
  try {
    categories.value = await ledgerListCategories();
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  }
}

async function applyFilters() {
  const { from, to } = currentRange();
  if (!from || !to) {
    window.alert('请选择有效日期范围');
    return;
  }
  if (preset.value === 'custom' && from > to) {
    window.alert('开始日不能晚于结束日');
    return;
  }
  loadingCharts.value = true;
  try {
    const stats = await ledgerGetStatistics({
      dateFrom: from,
      dateTo: to,
      direction: statDirection.value,
      categoryId:
        statCategoryId.value !== 'all' ? statCategoryId.value : undefined,
      currency: statCurrency.value,
    });
    renderCharts(stats);
  } catch (e) {
    window.alert(e instanceof Error ? e.message : String(e));
  } finally {
    loadingCharts.value = false;
  }
}

onMounted(async () => {
  initCustomRange();
  await loadCategories();
  await loadRollups();
  await applyFilters();
});

onBeforeUnmount(() => {
  destroyCharts();
});
</script>

<template>
  <div class="view-root">
    <div class="layout-ledger">
      <div class="ledger-stat-preview">
        <div class="panel stat-filters">
          <div class="panel-h">筛选 · 统计口径</div>
          <div class="panel-body stat-filters-body">
            <div class="stat-filter-row">
              <span class="stat-filter-label">时间范围</span>
              <select v-model="preset" class="stat-filter-control" aria-label="时间范围">
                <option value="week">本周</option>
                <option value="month">本月</option>
                <option value="year">本年</option>
                <option value="custom">自定义</option>
              </select>
              <span class="stat-filter-label">自定义起止</span>
              <input
                v-model="customFrom"
                class="stat-filter-date"
                type="date"
                aria-label="开始日期"
                :disabled="preset !== 'custom'"
              />
              <span class="stat-filter-sep">—</span>
              <input
                v-model="customTo"
                class="stat-filter-date"
                type="date"
                aria-label="结束日期"
                :disabled="preset !== 'custom'"
              />
            </div>
            <div class="stat-filter-row">
              <span class="stat-filter-label">收支</span>
              <select
                v-model="statDirection"
                class="stat-filter-control"
                aria-label="收支"
              >
                <option value="all">全部</option>
                <option value="in">仅收入</option>
                <option value="out">仅支出</option>
              </select>
              <span class="stat-filter-label">分类</span>
              <select
                v-model="statCategoryId"
                class="stat-filter-control"
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
              <span class="stat-filter-label">币种</span>
              <select
                v-model="statCurrency"
                class="stat-filter-control"
                aria-label="币种"
              >
                <option value="cny">人民币（CNY）</option>
                <option value="jpy">日元（JPY）</option>
              </select>
              <button
                type="button"
                class="btn"
                style="margin-left: 4px"
                :disabled="loadingCharts"
                @click="applyFilters"
              >
                应用
              </button>
            </div>
            <p class="ledger-field-note">
              折线图仅展示按日支出，仍受「收支」筛选（仅收入时序列为 0）；分类柱状/饼图在「全部」口径下按支出累计（与后端一致）。下方四张卡片为今日/本周/本月/本年各币种汇总，来自数据库实时聚合。
            </p>
          </div>
        </div>
        <div class="ledger-summary">
          <template v-if="loadingRollup">
            <div class="panel">
              <div class="panel-h">汇总</div>
              <div class="panel-body">
                <p class="ledger-field-note">加载中…</p>
              </div>
            </div>
          </template>
          <template v-else>
            <div v-for="card in rollups" :key="card.period" class="panel">
              <div class="panel-h">{{ card.label }}</div>
              <div class="panel-body ledger-summary-card-body">
                <div class="rollup-line">
                  <span>CNY 收入</span>
                  <span>{{ card.cnyIncome }}</span>
                </div>
                <div class="rollup-line">
                  <span>CNY 支出</span>
                  <span>{{ card.cnyExpense }}</span>
                </div>
                <div class="rollup-line">
                  <span>JPY 收入</span>
                  <span>{{ card.jpyIncome }}</span>
                </div>
                <div class="rollup-line">
                  <span>JPY 支出</span>
                  <span>{{ card.jpyExpense }}</span>
                </div>
              </div>
            </div>
          </template>
        </div>
        <p v-if="loadingCharts" class="ledger-field-note" style="padding: 0 4px">
          图表更新中…
        </p>
        <div class="stat-charts">
          <div class="stat-chart-panel">
            <div class="panel-h">
              每日支出 · 折线图（当前币种 ·
              {{ statCurrency.toUpperCase() }}）
            </div>
            <div class="panel-body panel-body--chart">
              <canvas ref="dailyCanvas" aria-label="每日支出折线图" />
            </div>
          </div>
          <div class="stat-charts-row">
            <div class="stat-chart-panel">
              <div class="panel-h">分类排行 · 柱状图</div>
              <div class="panel-body panel-body--chart">
                <canvas ref="barCanvas" aria-label="分类柱状图" />
              </div>
            </div>
            <div class="stat-chart-panel">
              <div class="panel-h">分类占比 · 饼图</div>
              <div class="panel-body panel-body--chart">
                <canvas ref="pieCanvas" aria-label="分类饼图" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
