<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useScreenerStore } from "../stores/screener";
import { invoke } from "@tauri-apps/api/core";
import type { AnalysisData } from "../stores/analysis";
import ScreenerTable from "../components/ScreenerTable.vue";
import KlineChart from "../components/KlineChart.vue";

const store = useScreenerStore();
const chartData = ref<AnalysisData | null>(null);

const types = [
  { value: "stock", label: "股票" },
  { value: "etf", label: "ETF" },
  { value: "warrant", label: "權證" },
  { value: "etn", label: "ETN" },
];
const selectedSymbol = ref("");

onMounted(async () => {
  await store.fetchLatestTradeDate();
  await store.fetchCacheCount();
});

function onStartDateChange(e: Event) {
  store.startDate = (e.target as HTMLInputElement).value;
}

function onEndDateChange(e: Event) {
  store.endDate = (e.target as HTMLInputElement).value;
}

function onMinVolumeInput(e: Event) {
  const raw = (e.target as HTMLInputElement).value.replace(/,/g, "");
  const n = parseInt(raw, 10);
  if (!isNaN(n) && n >= 0) {
    store.minVolume = n;
  }
}

async function onRowSelected(symbol: string) {
  selectedSymbol.value = symbol;
  try {
    chartData.value = await invoke<AnalysisData>("get_analysis", {
      symbol,
      maPeriods: [5, 10, 20],
      rsiPeriod: 14,
      macdFast: 12,
      macdSlow: 26,
      macdSignal: 9,
    });
  } catch (e) {
    console.error("Failed to fetch chart data:", e);
    chartData.value = null;
  }
}
</script>

<template>
  <div class="screener-view">
    <div class="params-bar">
      <div class="param-group">
        <label>起始日期</label>
        <div class="date-row">
          <button
            @click="store.navigateDate(-1)"
            :disabled="store.loading || store.navigating"
            class="btn-nav"
          >◀</button>
          <input :value="store.startDate" type="date" @change="onStartDateChange" />
        </div>
      </div>
      <div class="param-group">
        <label>結束日期</label>
        <div class="date-row">
          <input :value="store.endDate" type="date" @change="onEndDateChange" />
          <button
            @click="store.navigateDate(1)"
            :disabled="store.loading || store.navigating"
            class="btn-nav"
          >▶</button>
        </div>
      </div>
      <div class="param-group">
        <label>最大帶寬</label>
        <input v-model.number="store.maxBandwidth" type="number" step="0.01" min="0" />
      </div>
      <div class="param-group">
        <label>持續天數</label>
        <input v-model.number="store.minDays" type="number" min="1" />
      </div>
      <div class="param-group">
        <label>最低成交量</label>
        <input
          :value="store.minVolume.toLocaleString()"
          @input="onMinVolumeInput"
          type="text"
          inputmode="numeric"
        />
      </div>
      <div class="param-group">
        <label>類型</label>
        <div class="checkbox-group">
          <label v-for="t in types" :key="t.value" class="checkbox-label">
            <input type="checkbox" :value="t.value" v-model="store.instrumentTypes" />
            {{ t.label }}
          </label>
        </div>
      </div>
      <button
        @click="store.runRangeScreen()"
        :disabled="store.loading || store.dateLoading"
        class="btn-primary"
      >
        {{ store.dateLoading ? "載入中..." : store.loading ? "篩選中..." : "開始篩選" }}
      </button>
      <button
        v-if="store.loading"
        @click="store.cancelScreening()"
        class="btn-cancel"
      >
        取消
      </button>
      <button
        @click="store.clearCache()"
        :disabled="store.loading || store.dateLoading || store.clearing"
        class="btn-clear"
      >
        {{ store.clearing ? "清除中..." : "清除快取" }}
      </button>
      <span v-if="store.cacheCount > 0" class="cache-badge">{{ store.cacheCount }}</span>
    </div>
    <div v-if="store.progress.total > 0 && store.loading" class="progress-bar">
      <div class="progress-fill" :style="{ width: (store.progress.current / store.progress.total * 100) + '%' }"></div>
      <span class="progress-text">處理中 {{ store.progress.current }}/{{ store.progress.total }} 天</span>
    </div>
    <div v-if="store.meta" class="meta-bar">
      已掃描 {{ store.meta.start_date }} → {{ store.meta.end_date }}，
      共 {{ store.meta.trading_days }} 個交易日
      （快取 {{ store.meta.cached_days }} 天，
      <template v-if="store.meta.failed_days.length > 0">失敗 {{ store.meta.failed_days.length }} 天，</template>
      實際計算 {{ store.meta.calculated_days }} 天）
    </div>
    <transition name="toast">
      <div v-if="store.cacheMessage" class="cache-toast">{{ store.cacheMessage }}</div>
    </transition>
    <div class="screener-content">
      <div class="table-section">
        <h3>篩選結果 ({{ store.results.length }})</h3>
        <p v-if="store.lastEmptyDate" class="empty-hint">
          該日期（{{ store.lastEmptyDate }}）無交易資料
        </p>
        <ScreenerTable :results="store.results" @select="onRowSelected" />
      </div>
      <div v-if="selectedSymbol && chartData" class="chart-section">
        <KlineChart
          :data="chartData"
          :show-candlestick="true"
          :show-bollinger="true"
          :show-ma="false"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.screener-view { display: flex; flex-direction: column; gap: 12px; height: 100%; }
.params-bar { display: flex; gap: 16px; align-items: flex-end; flex-wrap: wrap; padding-bottom: 12px; border-bottom: 1px solid #eee; }
.param-group { display: flex; flex-direction: column; gap: 2px; }
.param-group label { font-size: 14px; color: #666; font-weight: 600; }
.param-group input { padding: 6px 8px; border: 1px solid #ccc; border-radius: 4px; width: 100px; font-size: 15px; }
.date-row { display: flex; align-items: center; gap: 4px; }
.btn-nav { padding: 4px 10px; border: 1px solid #ccc; border-radius: 4px; cursor: pointer; font-size: 16px; background: #fff; line-height: 1; }
.btn-nav:hover { background: #f0f0f0; }
.btn-nav:disabled { opacity: 0.4; cursor: not-allowed; }
.btn-primary { padding: 8px 20px; background: #1976d2; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 15px; }
.btn-primary:hover { background: #1565c0; }
.btn-primary:disabled { background: #90caf9; cursor: not-allowed; }
.btn-cancel { padding: 8px 16px; border: 1px solid #d32f2f; border-radius: 4px; cursor: pointer; font-size: 14px; background: #fff; color: #d32f2f; }
.btn-cancel:hover { background: #ffebee; }
.btn-clear { padding: 8px 16px; border: 1px solid #ccc; border-radius: 4px; cursor: pointer; font-size: 14px; background: #fff; color: #666; }
.btn-clear:hover { background: #f5f5f5; border-color: #999; }
.btn-clear:disabled { opacity: 0.5; cursor: not-allowed; }
.cache-badge { background: #e3f2fd; color: #1976d2; font-size: 12px; padding: 2px 8px; border-radius: 10px; white-space: nowrap; }
input[type="date"] { width: 180px; }
.progress-bar { position: relative; height: 24px; background: #e0e0e0; border-radius: 4px; overflow: hidden; }
.progress-fill { height: 100%; background: #1976d2; transition: width 0.3s ease; border-radius: 4px; }
.progress-text { position: absolute; top: 0; left: 0; right: 0; bottom: 0; display: flex; align-items: center; justify-content: center; font-size: 13px; color: #fff; font-weight: 600; }
.meta-bar { font-size: 13px; color: #666; padding: 4px 0; }
.empty-hint { color: #d32f2f; font-size: 14px; margin: 8px 0; }
.checkbox-group { display: flex; gap: 8px; flex-wrap: wrap; }
.checkbox-label { display: flex; align-items: center; gap: 3px; font-size: 13px; cursor: pointer; }
.checkbox-label input { width: auto; padding: 0; border: none; }
.screener-content { display: flex; gap: 16px; flex: 1; min-height: 0; }
.table-section { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.chart-section { flex: 1; min-width: 0; }
h3 { font-size: 16px; margin-bottom: 8px; color: #333; }
.cache-toast { position: fixed; top: 16px; right: 16px; background: #4caf50; color: #fff; padding: 10px 20px; border-radius: 6px; font-size: 14px; box-shadow: 0 2px 8px rgba(0,0,0,0.2); z-index: 1000; animation: toast-in 0.3s ease; }
@keyframes toast-in { from { opacity: 0; transform: translateX(20px); } to { opacity: 1; transform: translateX(0); } }
</style>
