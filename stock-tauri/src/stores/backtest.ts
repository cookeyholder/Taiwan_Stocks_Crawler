import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

export interface ParamDef {
  name: string;
  type: string;
  default: number;
  min: number;
  max: number;
}

export interface StrategyInfo {
  id: string;
  name: string;
  params: ParamDef[];
}

export interface PerformanceMetrics {
  total_return_pct: number;
  annualized_return_pct: number;
  win_rate: number;
  total_trades: number;
  winning_trades: number;
  losing_trades: number;
  sharpe_ratio: number;
  max_drawdown_pct: number;
  final_value: number;
}

export interface TradeRecord {
  symbol: string;
  name: string | null;
  buy_date: string;
  buy_price: number;
  sell_date: string | null;
  sell_price: number | null;
  shares: number;
  pnl: number;
  pnl_pct: number;
}

export interface BacktestReport {
  signals: any[];
  metrics: PerformanceMetrics;
  trades: TradeRecord[];
  equity_curve: number[];
  trade_dates: string[];
}

export const useBacktestStore = defineStore("backtest", () => {
  const strategies = ref<StrategyInfo[]>([]);
  const selectedStrategy = ref("");
  const strategyParams = ref<Record<string, number>>({});
  const startDate = ref("");
  const endDate = ref("");
  const capital = ref(1000000);
  const instrumentTypes = ref<string[]>(["stock"]);
  const loading = ref(false);
  const report = ref<BacktestReport | null>(null);
  const error = ref<string | null>(null);

  async function fetchStrategies() {
    try {
      strategies.value = await invoke<StrategyInfo[]>("list_strategies");
      if (strategies.value.length > 0 && !selectedStrategy.value) {
        selectedStrategy.value = strategies.value[0].id;
        updateParamsForStrategy();
      }
    } catch (e) {
      error.value = String(e);
    }
  }

  function updateParamsForStrategy() {
    const s = strategies.value.find((s) => s.id === selectedStrategy.value);
    if (s) {
      const p: Record<string, number> = {};
      for (const param of s.params) {
        p[param.name] = param.default;
      }
      strategyParams.value = p;
    }
  }

  function selectStrategy(id: string) {
    selectedStrategy.value = id;
    updateParamsForStrategy();
    report.value = null;
    error.value = null;
  }

  async function runBacktest() {
    if (!startDate.value || !endDate.value) return;
    loading.value = true;
    error.value = null;
    report.value = null;
    try {
      report.value = await invoke<BacktestReport>("run_backtest", {
        params: {
          strategy: selectedStrategy.value,
          params: strategyParams.value,
          startDate: startDate.value,
          endDate: endDate.value,
          capital: capital.value,
          instrumentTypes: instrumentTypes.value.length > 0 ? instrumentTypes.value : null,
        },
      });
    } catch (e) {
      error.value = String(e);
    } finally {
      loading.value = false;
    }
  }

  return {
    strategies, selectedStrategy, strategyParams,
    startDate, endDate, capital, instrumentTypes,
    loading, report, error,
    fetchStrategies, selectStrategy, runBacktest,
  };
});
