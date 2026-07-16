<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useBacktestStore } from "../stores/backtest";

const store = useBacktestStore();

onMounted(async () => {
  await store.fetchStrategies();
});

onMounted(async () => {
  const d = await (await import("../stores/screener")).useScreenerStore();
  if (d.startDate) {
    store.startDate = d.startDate;
    store.endDate = d.endDate;
  }
});

function fmtPct(v: number): string {
  return (v >= 0 ? "+" : "") + v.toFixed(1) + "%";
}

function fmtMoney(v: number): string {
  return "$" + Math.round(v).toLocaleString();
}
</script>

<template>
  <div class="backtest-view">
    <div class="params-bar">
      <div class="param-group">
        <label>策略</label>
        <select v-model="store.selectedStrategy" @change="store.selectStrategy(store.selectedStrategy)" class="strategy-select">
          <option v-for="s in store.strategies" :key="s.id" :value="s.id">{{ s.name }}</option>
        </select>
      </div>
      <div class="param-group" v-for="param in store.strategies.find(s => s.id === store.selectedStrategy)?.params ?? []" :key="param.name">
        <label>{{ param.name }}</label>
        <input v-model.number="store.strategyParams[param.name]" type="number" :step="param.type === 'float' ? 0.01 : 1" :min="param.min" :max="param.max" />
      </div>
      <div class="param-group">
        <label>起始日期</label>
        <input v-model="store.startDate" type="date" />
      </div>
      <div class="param-group">
        <label>結束日期</label>
        <input v-model="store.endDate" type="date" />
      </div>
      <div class="param-group">
        <label>起始資金</label>
        <input v-model.number="store.capital" type="number" min="0" step="100000" />
      </div>
      <div class="param-group">
        <label>類型</label>
        <div class="checkbox-group">
          <label v-for="t in [{value:'stock',label:'股票'},{value:'etf',label:'ETF'},{value:'warrant',label:'權證'},{value:'etn',label:'ETN'}]" :key="t.value" class="checkbox-label">
            <input type="checkbox" :value="t.value" v-model="store.instrumentTypes" />
            {{ t.label }}
          </label>
        </div>
      </div>
      <button @click="store.runBacktest()" :disabled="store.loading || !store.startDate || !store.endDate" class="btn-primary">
        {{ store.loading ? "執行中..." : "執行回測" }}
      </button>
    </div>

    <div v-if="store.error" class="error-card">{{ store.error }}</div>

    <div v-if="store.report" class="metrics-row">
      <div class="metric-card" :class="store.report.metrics.total_return_pct >= 0 ? 'up' : 'down'">
        <div class="metric-label">總報酬率</div>
        <div class="metric-value">{{ fmtPct(store.report.metrics.total_return_pct) }}</div>
      </div>
      <div class="metric-card">
        <div class="metric-label">勝率</div>
        <div class="metric-value">{{ (store.report.metrics.win_rate * 100).toFixed(1) }}%</div>
        <div class="metric-sub">{{ store.report.metrics.winning_trades }}/{{ store.report.metrics.total_trades }}</div>
      </div>
      <div class="metric-card">
        <div class="metric-label">Sharpe</div>
        <div class="metric-value">{{ store.report.metrics.sharpe_ratio.toFixed(2) }}</div>
      </div>
      <div class="metric-card down">
        <div class="metric-label">最大回撤</div>
        <div class="metric-value">{{ store.report.metrics.max_drawdown_pct.toFixed(1) }}%</div>
      </div>
      <div class="metric-card">
        <div class="metric-label">最終價值</div>
        <div class="metric-value">{{ fmtMoney(store.report.metrics.final_value) }}</div>
      </div>
    </div>

    <div v-if="store.report && store.report.trades.length > 0" class="table-card">
      <h3>交易明細 ({{ store.report.trades.length }})</h3>
      <table class="data-table">
        <thead>
          <tr><th>買入日</th><th>賣出日</th><th>代號</th><th>名稱</th><th>股數</th><th>買價</th><th>賣價</th><th>損益</th><th>報酬率</th></tr>
        </thead>
        <tbody>
          <tr v-for="t in store.report.trades" :key="t.buy_date + t.symbol">
            <td>{{ t.buy_date }}</td>
            <td>{{ t.sell_date || "持有中" }}</td>
            <td>{{ t.symbol }}</td>
            <td>{{ t.name || "-" }}</td>
            <td class="num">{{ t.shares.toLocaleString() }}</td>
            <td class="num">{{ t.buy_price.toFixed(2) }}</td>
            <td class="num">{{ t.sell_price?.toFixed(2) || "-" }}</td>
            <td class="num" :class="t.pnl >= 0 ? 'up' : 'down'">{{ t.pnl >= 0 ? "+" : "" }}{{ t.pnl.toFixed(0) }}</td>
            <td class="num" :class="t.pnl_pct >= 0 ? 'up' : 'down'">{{ t.pnl_pct >= 0 ? "+" : "" }}{{ t.pnl_pct.toFixed(1) }}%</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div v-if="store.report && store.report.signals.length > 0" class="table-card">
      <h3>訊號明細 ({{ store.report.signals.length }})</h3>
      <table class="data-table">
        <thead>
          <tr><th>日期</th><th>代號</th><th>名稱</th><th>方向</th><th>收盤價</th></tr>
        </thead>
        <tbody>
          <tr v-for="s in store.report.signals" :key="s.trade_date + s.symbol">
            <td>{{ s.trade_date }}</td>
            <td>{{ s.symbol }}</td>
            <td>{{ s.name || "-" }}</td>
            <td :class="s.direction > 0 ? 'up' : s.direction < 0 ? 'down' : ''">
              {{ s.direction > 0 ? "買" : s.direction < 0 ? "賣" : "-" }}
            </td>
            <td class="num">{{ s.close_price?.toFixed(2) || "-" }}</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.backtest-view { display: flex; flex-direction: column; gap: 16px; }
.params-bar { display: flex; gap: 12px; align-items: flex-end; flex-wrap: wrap; padding-bottom: 12px; border-bottom: 1px solid #eee; }
.param-group { display: flex; flex-direction: column; gap: 2px; }
.param-group label { font-size: 13px; color: #666; font-weight: 600; }
.param-group input, .strategy-select { padding: 6px 8px; border: 1px solid #ccc; border-radius: 4px; font-size: 14px; }
.strategy-select { min-width: 140px; }
.btn-primary { padding: 8px 20px; background: #1976d2; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 15px; }
.btn-primary:hover { background: #1565c0; }
.btn-primary:disabled { background: #90caf9; cursor: not-allowed; }
.metrics-row { display: flex; gap: 12px; flex-wrap: wrap; }
.metric-card { background: #f9f9f9; border: 1px solid #eee; border-radius: 8px; padding: 14px 18px; min-width: 130px; text-align: center; }
.metric-card.up { border-color: #a5d6a7; }
.metric-card.down { border-color: #ef9a9a; }
.metric-label { font-size: 12px; color: #888; margin-bottom: 4px; }
.metric-value { font-size: 20px; font-weight: 700; }
.metric-card.up .metric-value { color: #2e7d32; }
.metric-card.down .metric-value { color: #c62828; }
.metric-sub { font-size: 12px; color: #999; }
.table-card { background: #f9f9f9; border: 1px solid #eee; border-radius: 8px; padding: 16px; }
h3 { font-size: 15px; margin-bottom: 10px; color: #333; }
.data-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.data-table th, .data-table td { padding: 6px 10px; text-align: left; border-bottom: 1px solid #eee; }
.data-table th { font-weight: 600; color: #666; white-space: nowrap; }
.data-table td.num { text-align: right; font-variant-numeric: tabular-nums; }
.data-table td.up { color: #2e7d32; }
.data-table td.down { color: #c62828; }
.checkbox-group { display: flex; gap: 8px; flex-wrap: wrap; }
.checkbox-label { display: flex; align-items: center; gap: 3px; font-size: 13px; cursor: pointer; }
.checkbox-label input { width: auto; padding: 0; border: none; }
.error-card { background: #ffebee; border: 1px solid #ef9a9a; border-radius: 8px; padding: 12px 16px; color: #c62828; font-size: 14px; }
</style>
