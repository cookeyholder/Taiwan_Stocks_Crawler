<script setup lang="ts">
import { ref, watch, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useAnalysisStore } from "../stores/analysis";
import KlineChart from "../components/KlineChart.vue";
import ParamForm from "../components/ParamForm.vue";

const store = useAnalysisStore();
const symbols = ref<[string, string, string | null][]>([]);
const selectedSymbol = ref("");

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
      </div>
      <ParamForm @change="onParamsChange" />
    </div>
    <div class="chart-area">
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
.loading-overlay { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; background: rgba(255,255,255,0.7); color: #888; font-size: 16px; z-index: 10; }
.error, .empty { display: flex; align-items: center; justify-content: center; height: 100%; color: #888; font-size: 16px; }
.error { color: #d32f2f; }
</style>
