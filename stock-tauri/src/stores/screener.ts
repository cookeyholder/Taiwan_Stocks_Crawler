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
  latest_upper_band: number | null;
  break_upper: boolean;
}

export const useScreenerStore = defineStore("screener", () => {
  const results = ref<ScreenResult[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const maxBandwidth = ref(0.08);
  const minDays = ref(5);
  const minVolume = ref(1000000);
  const instrumentTypes = ref<string[]>(["stock"]);
  const date = ref("");
  const dateLoading = ref(false);

  async function fetchLatestTradeDate() {
    dateLoading.value = true;
    try {
      const d = await invoke<string | null>("get_latest_trade_date");
      if (d) date.value = d;
    } catch {
      // 保留空日期，讓使用者手動輸入
    } finally {
      dateLoading.value = false;
    }
  }

  const lastEmptyDate = ref("");

  const clearing = ref(false);

  async function runScreen() {
    loading.value = true;
    error.value = null;
    lastEmptyDate.value = "";
    try {
      results.value = await invoke<ScreenResult[]>("screen_bollinger_with_cache", {
        maxBandwidth: maxBandwidth.value,
        minDays: minDays.value,
        instrumentTypes: instrumentTypes.value.length > 0 ? instrumentTypes.value : null,
        minVolume: minVolume.value,
        date: date.value || null,
      });
      if (results.value.length === 0 && date.value) {
        lastEmptyDate.value = date.value;
      }
    } catch (e) {
      error.value = String(e);
      results.value = [];
    } finally {
      loading.value = false;
    }
  }

  async function clearCache() {
    clearing.value = true;
    error.value = null;
    try {
      const deleted = await invoke<number>("clear_screener_cache");
      console.log(`已清除 ${deleted} 筆快取`);
    } catch (e) {
      error.value = String(e);
    } finally {
      clearing.value = false;
    }
  }

  return {
    results, loading, error,
    maxBandwidth, minDays, minVolume, instrumentTypes,
    date, dateLoading,
    fetchLatestTradeDate, runScreen, lastEmptyDate,
    clearCache, clearing,
  };
});
