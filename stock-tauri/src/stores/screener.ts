import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface ScreenResult {
  symbol: string;
  name: string | null;
  latest_close: number | null;
  latest_volume: number | null;
  max_bandwidth: number;
  latest_bandwidth: number;
  break_upper: boolean;
}

export const useScreenerStore = defineStore("screener", () => {
  const results = ref<ScreenResult[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const maxBandwidth = ref(0.08);
  const minDays = ref(5);
  const minVolume = ref(0);
  const instrumentTypes = ref<string[]>([]);

  async function runScreen() {
    loading.value = true;
    error.value = null;
    try {
      results.value = await invoke<ScreenResult[]>("screen_bollinger", {
        maxBandwidth: maxBandwidth.value,
        minDays: minDays.value,
        instrumentTypes: instrumentTypes.value.length > 0 ? instrumentTypes.value : null,
        minVolume: minVolume.value,
      });
    } catch (e) {
      error.value = String(e);
      results.value = [];
    } finally {
      loading.value = false;
    }
  }

  return {
    results, loading, error,
    maxBandwidth, minDays, minVolume, instrumentTypes,
    runScreen,
  };
});
