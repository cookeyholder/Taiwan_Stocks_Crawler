import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface OhlcvPoint {
  trade_date: string;
  open_price: number | null;
  high_price: number | null;
  low_price: number | null;
  close_price: number | null;
  volume_shares: number | null;
}

export interface AnalysisData {
  ohlcv: OhlcvPoint[];
  bollinger_upper: (number | null)[];
  bollinger_middle: (number | null)[];
  bollinger_lower: (number | null)[];
  ma5: (number | null)[];
  ma10: (number | null)[];
  ma20: (number | null)[];
  rsi: (number | null)[];
  macd_line: (number | null)[];
  macd_signal: (number | null)[];
  macd_histogram: (number | null)[];
  symbol: string;
  name: string | null;
}

export const useAnalysisStore = defineStore("analysis", () => {
  const data = ref<AnalysisData | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);
  const selectedSymbol = ref("");

  const maPeriods = ref([5, 10, 20]);
  const rsiPeriod = ref(14);
  const showRsi = ref(true);
  const macdFast = ref(12);
  const macdSlow = ref(26);
  const macdSignal = ref(9);
  const showMacd = ref(true);

  async function fetchAnalysis(symbol: string) {
    if (!symbol) return;
    loading.value = true;
    error.value = null;
    selectedSymbol.value = symbol;
    try {
      const periods = [...maPeriods.value];
      data.value = await invoke<AnalysisData>("get_analysis", {
        symbol,
        maPeriods: periods,
        rsiPeriod: showRsi.value ? rsiPeriod.value : null,
        macdFast: showMacd.value ? macdFast.value : null,
        macdSlow: showMacd.value ? macdSlow.value : null,
        macdSignal: showMacd.value ? macdSignal.value : null,
      });
    } catch (e) {
      error.value = String(e);
      data.value = null;
    } finally {
      loading.value = false;
    }
  }

  return {
    data, loading, error, selectedSymbol,
    maPeriods, rsiPeriod, showRsi, macdFast, macdSlow, macdSignal, showMacd,
    fetchAnalysis,
  };
});
