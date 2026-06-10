<script setup lang="ts">
import { watch, ref } from "vue";
import { useAnalysisStore } from "../stores/analysis";

const store = useAnalysisStore();
const emit = defineEmits<{ change: [] }>();

let debounceTimer: ReturnType<typeof setTimeout> | null = null;

const localMa5 = ref(store.maPeriods[0]);
const localMa10 = ref(store.maPeriods[1]);
const localMa20 = ref(store.maPeriods[2]);
const localRsiPeriod = ref(store.rsiPeriod);
const localMacdFast = ref(store.macdFast);
const localMacdSlow = ref(store.macdSlow);
const localMacdSignal = ref(store.macdSignal);

function applyChanges() {
  store.maPeriods = [localMa5.value, localMa10.value, localMa20.value];
  store.rsiPeriod = localRsiPeriod.value;
  store.macdFast = localMacdFast.value;
  store.macdSlow = localMacdSlow.value;
  store.macdSignal = localMacdSignal.value;
  if (debounceTimer) clearTimeout(debounceTimer);
  debounceTimer = setTimeout(() => emit("change"), 300);
}

watch([localMa5, localMa10, localMa20, localRsiPeriod, localMacdFast, localMacdSlow, localMacdSignal], applyChanges);
</script>

<template>
  <div class="param-form">
    <h4>均線週期</h4>
    <div class="param-row">
      <label>MA5</label>
      <input v-model.number="localMa5" type="number" min="1" />
    </div>
    <div class="param-row">
      <label>MA10</label>
      <input v-model.number="localMa10" type="number" min="1" />
    </div>
    <div class="param-row">
      <label>MA20</label>
      <input v-model.number="localMa20" type="number" min="1" />
    </div>

    <h4>RSI</h4>
    <div class="param-row">
      <label>
        <input v-model="store.showRsi" type="checkbox" @change="emit('change')" />
        顯示 RSI
      </label>
    </div>
    <div class="param-row">
      <label>週期</label>
      <input v-model.number="localRsiPeriod" type="number" min="1" />
    </div>

    <h4>MACD</h4>
    <div class="param-row">
      <label>
        <input v-model="store.showMacd" type="checkbox" @change="emit('change')" />
        顯示 MACD
      </label>
    </div>
    <div class="param-row">
      <label>快線</label>
      <input v-model.number="localMacdFast" type="number" min="1" />
    </div>
    <div class="param-row">
      <label>慢線</label>
      <input v-model.number="localMacdSlow" type="number" min="1" />
    </div>
    <div class="param-row">
      <label>訊號</label>
      <input v-model.number="localMacdSignal" type="number" min="1" />
    </div>
  </div>
</template>

<style scoped>
.param-form { display: flex; flex-direction: column; gap: 8px; }
h4 { font-size: 13px; color: #555; margin-top: 4px; padding-top: 8px; border-top: 1px solid #eee; }
.param-row { display: flex; align-items: center; gap: 8px; }
.param-row label { font-size: 13px; color: #666; min-width: 50px; }
.param-row input[type="number"] { padding: 4px 6px; border: 1px solid #ccc; border-radius: 4px; width: 60px; font-size: 13px; }
.param-row input[type="checkbox"] { margin-right: 4px; }
</style>
