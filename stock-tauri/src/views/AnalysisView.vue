<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useAnalysisStore } from "../stores/analysis";
import KlineChart from "../components/KlineChart.vue";
import ParamForm from "../components/ParamForm.vue";

const store = useAnalysisStore();
const symbols = ref<[string, string, string | null][]>([]);
const selectedSymbol = ref("");

const selectedInfo = computed(() => {
  if (!selectedSymbol.value) return null;
  const s = symbols.value.find(([sym]) => sym === selectedSymbol.value);
  if (!s) return null;
  return { symbol: s[0], market: s[1], name: s[2] };
});

onMounted(async () => {
  symbols.value = await invoke<[string, string, string | null][]>("get_symbols");
});

watch(selectedSymbol, (val) => {
  if (val) store.fetchAnalysis(val);
});

function onParamsChange() {
  if (selectedSymbol.value) store.fetchAnalysis(selectedSymbol.value);
}
</script>

<template>
  <div class="analysis-view">
    <div class="sidebar">
      <div class="control-group">
        <label>股票</label>
        <select v-model="selectedSymbol" class="symbol-select">
          <option value="" disabled>選擇...</option>
          <option v-for="[sym, mkt, name] in symbols" :key="sym" :value="sym">
            {{ sym }} — {{ name ?? '' }} ({{ mkt }})
          </option>
        </select>
        <div v-if="selectedInfo" class="selected-info">
          <span class="sel-symbol">{{ selectedInfo.symbol }}</span>
          <span class="sel-name">{{ selectedInfo.name }}</span>
          <span class="sel-market">{{ selectedInfo.market }}</span>
        </div>
      </div>
      <ParamForm @change="onParamsChange" />
    </div>
    <div class="chart-area">
      <div v-if="selectedInfo" class="chart-header">
        <span class="chart-symbol">{{ selectedInfo.symbol }}</span>
        <span class="chart-name">{{ selectedInfo.name }}</span>
        <span class="chart-market">{{ selectedInfo.market }}</span>
      </div>
      <div v-if="store.loading" class="loading-overlay">載入中...</div>
      <div v-if="store.error && !store.loading" class="error">{{ store.error }}</div>
      <KlineChart
        v-if="store.data"
        :data="store.data"
        :show-candlestick="store.showCandlestick"
        :show-bollinger="store.showBollinger"
      />
      <div v-else-if="!store.loading && !store.error" class="empty">請選擇股票代號查看分析</div>
    </div>
  </div>
</template>

<style scoped>
.analysis-view { display: flex; gap: 16px; height: 100%; }
.sidebar { width: 240px; flex-shrink: 0; display: flex; flex-direction: column; gap: 12px; }
.chart-area { flex: 1; min-width: 0; position: relative; }
.control-group { display: flex; flex-direction: column; gap: 4px; }
.control-group label { font-weight: 600; font-size: 13px; color: #555; }
.symbol-select { width: 100%; padding: 6px 8px; border: 1px solid #ccc; border-radius: 4px; font-size: 14px; }
.selected-info { display: flex; gap: 6px; align-items: center; font-size: 13px; margin-top: 2px; }
.sel-symbol { font-weight: 700; color: #1976d2; }
.sel-name { color: #555; }
.sel-market { color: #999; font-size: 11px; }
.chart-header { padding: 8px 12px; border-bottom: 1px solid #eee; font-size: 14px; display: flex; gap: 8px; align-items: baseline; }
.chart-symbol { font-weight: 700; color: #1976d2; font-size: 16px; }
.chart-name { color: #555; }
.chart-market { color: #999; font-size: 12px; }
.loading-overlay { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; background: rgba(255,255,255,0.7); color: #888; font-size: 16px; z-index: 10; }
.error, .empty { display: flex; align-items: center; justify-content: center; height: 100%; color: #888; font-size: 16px; }
.error { color: #d32f2f; }
</style>
