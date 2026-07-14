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
  store.runScreen();
});

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
        <label>日期</label>
        <input v-model="store.date" type="date" class="date-input" />
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
        @click="store.runScreen()"
        :disabled="store.loading || store.dateLoading"
        class="btn-primary"
      >
        {{ store.dateLoading ? "載入中..." : store.loading ? "篩選中..." : "開始篩選" }}
      </button>
    </div>
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
.btn-primary { padding: 8px 20px; background: #1976d2; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 15px; }
.btn-primary:hover { background: #1565c0; }
.btn-primary:disabled { background: #90caf9; cursor: not-allowed; }
.date-input { padding: 6px 8px; border: 1px solid #ccc; border-radius: 4px; width: 150px; font-size: 15px; }
.empty-hint { color: #d32f2f; font-size: 14px; margin: 8px 0; }
.checkbox-group { display: flex; gap: 8px; flex-wrap: wrap; }
.checkbox-label { display: flex; align-items: center; gap: 3px; font-size: 13px; cursor: pointer; }
.screener-content { display: flex; gap: 16px; flex: 1; min-height: 0; }
.table-section { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.chart-section { flex: 1; min-width: 0; }
h3 { font-size: 16px; margin-bottom: 8px; color: #333; }
</style>
