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

export interface RangeMeta {
  start_date: string;
  end_date: string;
  trading_days: number;
  cached_days: number;
  calculated_days: number;
  failed_days: string[];
}

export interface RangeScreenResult {
  results: ScreenResult[];
  meta: RangeMeta;
}

interface RangeProgress {
  current: number;
  total: number;
  date: string;
  cached: boolean;
}

export const useScreenerStore = defineStore("screener", () => {
  const results = ref<ScreenResult[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  const maxBandwidth = ref(0.08);
  const minDays = ref(5);
  const minVolume = ref(1000000);
  const instrumentTypes = ref<string[]>(["stock"]);
  const startDate = ref("");
  const endDate = ref("");
  const dateLoading = ref(false);

  async function fetchLatestTradeDate() {
    dateLoading.value = true;
    try {
      const d = await invoke<string | null>("get_latest_trade_date");
      if (d) {
        startDate.value = d;
        endDate.value = d;
      }
    } catch {
      // 保留空日期，讓使用者手動輸入
    } finally {
      dateLoading.value = false;
    }
  }

  const meta = ref<RangeMeta | null>(null);
  const progress = ref<RangeProgress>({ current: 0, total: 0, date: "", cached: false });
  const navigating = ref(false);

  const lastEmptyDate = ref("");

  const clearing = ref(false);
  const cacheMessage = ref("");
  const cacheCount = ref(0);

  async function fetchCacheCount() {
    try {
      cacheCount.value = await invoke<number>("get_screener_cache_count");
    } catch {
      // 忽略
    }
  }

  async function runRangeScreen() {
    loading.value = true;
    error.value = null;
    lastEmptyDate.value = "";
    meta.value = null;
    progress.value = { current: 0, total: 0, date: "", cached: false };

    let unlisten: (() => void) | undefined;
    try {
      const { listen } = await import("@tauri-apps/api/event");
      unlisten = await listen<RangeProgress>("range-progress", (e) => {
        progress.value = e.payload;
      });

      const rangeResult = await invoke<RangeScreenResult>("screen_bollinger_range", {
        startDate: startDate.value,
        endDate: endDate.value,
        maxBandwidth: maxBandwidth.value,
        minDays: minDays.value,
        instrumentTypes: instrumentTypes.value.length > 0 ? instrumentTypes.value : null,
        minVolume: minVolume.value,
      });
      results.value = rangeResult.results;
      meta.value = rangeResult.meta;
      if (results.value.length === 0 && startDate.value) {
        lastEmptyDate.value = startDate.value;
      }
      await fetchCacheCount();
    } catch (e) {
      error.value = String(e);
      results.value = [];
    } finally {
      loading.value = false;
      navigating.value = false;
      if (unlisten) unlisten();
    }
  }

  async function navigateDate(direction: -1 | 1) {
    if (loading.value || navigating.value) return;
    navigating.value = true;
    const [sy, sm, sd] = startDate.value.split('-').map(Number);
    const [ey, em, ed] = endDate.value.split('-').map(Number);
    const start = new Date(sy, sm - 1, sd);
    const end = new Date(ey, em - 1, ed);
    start.setDate(start.getDate() + direction);
    end.setDate(end.getDate() + direction);
    const fmt = (d: Date) => {
      const y = d.getFullYear();
      const m = String(d.getMonth() + 1).padStart(2, '0');
      const day = String(d.getDate()).padStart(2, '0');
      return `${y}-${m}-${day}`;
    };
    startDate.value = fmt(start);
    endDate.value = fmt(end);
    await runRangeScreen();
  }

  async function clearCache() {
    clearing.value = true;
    error.value = null;
    try {
      const deleted = await invoke<number>("clear_screener_cache");
      await fetchCacheCount();
      const msg = cacheMessage.value = `已清除 ${deleted} 筆快取`;
      setTimeout(() => {
        if (cacheMessage.value === msg) cacheMessage.value = "";
      }, 2500);
    } catch (e) {
      error.value = String(e);
    } finally {
      clearing.value = false;
    }
  }

  async function cancelScreening() {
    try {
      await invoke("cancel_screening");
    } catch {
      // 忽略
    }
  }

  return {
    results, loading, error,
    maxBandwidth, minDays, minVolume, instrumentTypes,
    startDate, endDate, dateLoading,
    fetchLatestTradeDate, runRangeScreen, lastEmptyDate,
    navigateDate, navigating,
    clearCache, clearing, cacheMessage, cacheCount, fetchCacheCount,
    meta, progress, cancelScreening,
  };
});
