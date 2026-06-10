<script setup lang="ts">
import { ref } from "vue";
import { useScreenerStore } from "../stores/screener";
import { invoke } from "@tauri-apps/api/core";
import { useAnalysisStore } from "../stores/analysis";
import ScreenerTable from "../components/ScreenerTable.vue";
import KlineChart from "../components/KlineChart.vue";

const store = useScreenerStore();
const analysisStore = useAnalysisStore();
const selectedSymbol = ref("");

async function onRowSelected(symbol: string) {
  selectedSymbol.value = symbol;
  analysisStore.fetchAnalysis(symbol);
}
</script>

<template>
  <div class="screener-view">
    <div class="params-bar">
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
        <input v-model.number="store.minVolume" type="number" min="0" />
      </div>
      <button @click="store.runScreen()" :disabled="store.loading" class="btn-primary">
        {{ store.loading ? "篩選中..." : "開始篩選" }}
      </button>
    </div>
    <div class="screener-content">
      <div class="table-section">
        <h3>篩選結果 ({{ store.results.length }})</h3>
        <ScreenerTable :results="store.results" @select="onRowSelected" />
      </div>
      <div v-if="selectedSymbol && analysisStore.data" class="chart-section">
        <KlineChart :data="analysisStore.data" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.screener-view { display: flex; flex-direction: column; gap: 12px; height: 100%; }
.params-bar { display: flex; gap: 16px; align-items: flex-end; flex-wrap: wrap; padding-bottom: 12px; border-bottom: 1px solid #eee; }
.param-group { display: flex; flex-direction: column; gap: 2px; }
.param-group label { font-size: 12px; color: #666; font-weight: 600; }
.param-group input { padding: 6px 8px; border: 1px solid #ccc; border-radius: 4px; width: 100px; font-size: 14px; }
.btn-primary { padding: 8px 20px; background: #1976d2; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; }
.btn-primary:hover { background: #1565c0; }
.btn-primary:disabled { background: #90caf9; cursor: not-allowed; }
.screener-content { display: flex; gap: 16px; flex: 1; min-height: 0; }
.table-section { flex: 1; min-width: 0; display: flex; flex-direction: column; }
.chart-section { flex: 1; min-width: 0; }
h3 { font-size: 14px; margin-bottom: 8px; color: #333; }
</style>
