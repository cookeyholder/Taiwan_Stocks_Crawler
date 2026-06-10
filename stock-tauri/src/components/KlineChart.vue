<script setup lang="ts">
import { computed } from "vue";
import VChart from "vue-echarts";
import { use } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import { BarChart, CandlestickChart, LineChart } from "echarts/charts";
import {
  GridComponent, DataZoomComponent, TooltipComponent, LegendComponent,
} from "echarts/components";
import type { AnalysisData } from "../stores/analysis";

use([
  CanvasRenderer, BarChart, CandlestickChart, LineChart,
  GridComponent, DataZoomComponent, TooltipComponent, LegendComponent,
]);

const props = defineProps<{
  data: AnalysisData;
  showCandlestick?: boolean;
  showBollinger?: boolean;
}>();

function buildOption(data: AnalysisData): any {
  const dates = data.ohlcv.map((d) => d.trade_date);
  const ohlc = data.ohlcv.map((d) => [d.open_price, d.close_price, d.low_price, d.high_price]);
  const volume = data.ohlcv.map((d) => d.volume_shares ?? 0);

  const total = dates.length;
  const defaultWindow = Math.min(30, Math.max(10, total));
  const zoomStartValue = Math.max(0, total - defaultWindow);

  const gridTop = 40;
  const gridGap = 8;

  const series: any[] = [];
  const grids: any[] = [];
  const xAxes: any[] = [];
  const yAxes: any[] = [];
  const axisIndices: number[] = [];

  function addPane(height: number, idx: number) {
    const top = idx === 0 ? gridTop : grids.reduce((s, g) => s + g.height + gridGap, gridTop);
    grids.push({ left: 60, right: 20, top, height: `${height}%` });
    xAxes.push({
      gridIndex: idx, type: "category", data: dates, show: false, boundaryGap: false,
      axisLabel: { rotate: 45, fontSize: 10 },
    });
    yAxes.push({ gridIndex: idx, scale: true, splitLine: { lineStyle: { type: "dashed" } } });
    axisIndices.push(idx);
  }

  addPane(60, 0);
  addPane(12, 1);
  let rsiIdx = -1;
  let macdIdx = -1;
  if (data.rsi && data.rsi.some((v) => v !== null)) {
    rsiIdx = grids.length;
    addPane(14, rsiIdx);
  }
  if (data.macd_line && data.macd_line.some((v) => v !== null)) {
    macdIdx = grids.length;
    addPane(14, macdIdx);
  }

  if (props.showCandlestick !== false) {
    series.push({
      name: "K線", type: "candlestick", xAxisIndex: 0, yAxisIndex: 0,
      data: ohlc, itemStyle: { color: "#ef5350", color0: "#26a69a" },
    });
  }

  if (props.showBollinger !== false && data.bollinger_upper[0] !== null) {
    series.push({
      name: "上軌", type: "line", xAxisIndex: 0, yAxisIndex: 0,
      data: data.bollinger_upper, smooth: true, symbol: "none",
      lineStyle: { width: 1, type: "dashed", color: "#9ccc65" },
    });
    series.push({
      name: "中軌", type: "line", xAxisIndex: 0, yAxisIndex: 0,
      data: data.bollinger_middle, smooth: true, symbol: "none",
      lineStyle: { width: 1, color: "#ffa726" },
    });
    series.push({
      name: "下軌", type: "line", xAxisIndex: 0, yAxisIndex: 0,
      data: data.bollinger_lower, smooth: true, symbol: "none",
      lineStyle: { width: 1, type: "dashed", color: "#ef5350" },
    });
  }

  if (data.ma5[0] !== null) {
    series.push({ name: "MA5", type: "line", xAxisIndex: 0, yAxisIndex: 0, data: data.ma5, smooth: true, symbol: "none", lineStyle: { width: 1 } });
  }
  if (data.ma10[0] !== null) {
    series.push({ name: "MA10", type: "line", xAxisIndex: 0, yAxisIndex: 0, data: data.ma10, smooth: true, symbol: "none", lineStyle: { width: 1 } });
  }
  if (data.ma20[0] !== null) {
    series.push({ name: "MA20", type: "line", xAxisIndex: 0, yAxisIndex: 0, data: data.ma20, smooth: true, symbol: "none", lineStyle: { width: 1 } });
  }

  series.push({
    name: "成交量", type: "bar", xAxisIndex: 1, yAxisIndex: 1,
    data: volume.map((v) => [v]),
    itemStyle: { color: "#26a69a" },
  });

  if (rsiIdx >= 0) {
    series.push({
      name: "RSI", type: "line", xAxisIndex: rsiIdx, yAxisIndex: rsiIdx,
      data: data.rsi, smooth: true, symbol: "none",
      lineStyle: { width: 1, color: "#e040fb" },
    });
    series.push({
      name: "超買", type: "line", xAxisIndex: rsiIdx, yAxisIndex: rsiIdx,
      data: Array(dates.length).fill(70), symbol: "none",
      lineStyle: { width: 1, type: "dashed", color: "#ef5350" },
    });
    series.push({
      name: "超賣", type: "line", xAxisIndex: rsiIdx, yAxisIndex: rsiIdx,
      data: Array(dates.length).fill(30), symbol: "none",
      lineStyle: { width: 1, type: "dashed", color: "#66bb6a" },
    });
  }

  if (macdIdx >= 0) {
    series.push({
      name: "MACD", type: "line", xAxisIndex: macdIdx, yAxisIndex: macdIdx,
      data: data.macd_line, smooth: true, symbol: "none",
      lineStyle: { width: 1, color: "#1976d2" },
    });
    series.push({
      name: "訊號", type: "line", xAxisIndex: macdIdx, yAxisIndex: macdIdx,
      data: data.macd_signal, smooth: true, symbol: "none",
      lineStyle: { width: 1, type: "dashed", color: "#ff7043" },
    });
    series.push({
      name: "柱狀", type: "bar", xAxisIndex: macdIdx, yAxisIndex: macdIdx,
      data: data.macd_histogram.map((v) => [(v ?? 0) >= 0 ? v : v]),
      itemStyle: { color: (p: any) => (p.value[0] >= 0 ? "#ef5350" : "#26a69a") },
    });
  }

  return {
    animationDurationUpdate: 100,
    animationEasingUpdate: "linear",
    tooltip: { trigger: "axis", axisPointer: { type: "cross" } },
    legend: { top: 5, left: "center", selectedMode: "multiple" },
    grid: grids,
    xAxis: xAxes,
    yAxis: yAxes,
    dataZoom: [{
      type: "inside",
      xAxisIndex: axisIndices,
      startValue: zoomStartValue,
      endValue: total - 1,
      minValueSpan: 5,
      maxValueSpan: total,
    }],
    series,
  };
}

const option = computed(() => buildOption(props.data));
</script>

<template>
  <div class="chart-container">
    <VChart :option="option" autoresize class="chart" />
  </div>
</template>

<style scoped>
.chart-container { width: 100%; height: 100%; min-height: 400px; }
.chart { width: 100%; height: 100%; }
</style>
