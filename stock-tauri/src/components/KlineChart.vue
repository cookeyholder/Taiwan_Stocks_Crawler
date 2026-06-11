<script setup lang="ts">
import { computed } from "vue";
import VChart from "vue-echarts";
import { use } from "echarts/core";
import { CanvasRenderer } from "echarts/renderers";
import { BarChart, CandlestickChart, LineChart } from "echarts/charts";
import {
  GridComponent, DataZoomComponent, TooltipComponent, LegendComponent,
  TitleComponent,
} from "echarts/components";
import type { AnalysisData } from "../stores/analysis";
import { extractOHLC } from "./tooltip-format";

use([
  CanvasRenderer, BarChart, CandlestickChart, LineChart,
  GridComponent, DataZoomComponent, TooltipComponent, LegendComponent, TitleComponent,
]);

const props = defineProps<{
  data: AnalysisData;
  showCandlestick?: boolean;
  showBollinger?: boolean;
  showMA?: boolean;
}>();

function buildOption(data: AnalysisData): any {
  const dates = data.ohlcv.map((d) => d.trade_date);
  const ohlc = data.ohlcv.map((d) => [d.open_price, d.close_price, d.low_price, d.high_price]);
  const volume = data.ohlcv.map((d) => d.volume_shares ?? 0);

  const total = dates.length;
  const defaultWindow = Math.min(30, Math.max(10, total));
  const zoomStartValue = Math.max(0, total - defaultWindow);

  const gridTop = 40;
  const gridGap = 20;

  const series: any[] = [];
  const grids: any[] = [];
  const xAxes: any[] = [];
  const yAxes: any[] = [];
  const axisIndices: number[] = [];
  let paneY = gridTop;

  function addPane(heightPx: number) {
    const idx = grids.length;
    grids.push({ left: 60, right: 20, top: paneY, height: heightPx });
    paneY += heightPx + gridGap;
    xAxes.push({
      gridIndex: idx, type: "category", data: dates, show: false, boundaryGap: false,
      axisLabel: { rotate: 45, fontSize: 10 },
    });
    yAxes.push({ gridIndex: idx, scale: true, splitLine: { lineStyle: { type: "dashed" } } });
    axisIndices.push(idx);
  }

  // approximate pixel heights for a typical 700px chart container
  const mainH = 300;
  const volH = 50;
  const indH = 80;
  const indHMacd = 120;

  addPane(mainH);
  addPane(volH);
  let rsiIdx = -1;
  let macdIdx = -1;
  if (data.rsi && data.rsi.some((v) => v !== null)) {
    rsiIdx = grids.length;
    addPane(indH);
  }
  if (data.macd_line && data.macd_line.some((v) => v !== null)) {
    macdIdx = grids.length;
    addPane(indHMacd);
  }

  if (props.showCandlestick !== false) {
    series.push({
      name: "K線", type: "candlestick", xAxisIndex: 0, yAxisIndex: 0,
      data: ohlc, itemStyle: { color: "#ef5350", color0: "#26a69a" },
    });
  }

  if (props.showBollinger !== false && data.bollinger_upper.some(v => v !== null)) {
    series.push({
      name: "上軌", type: "line", xAxisIndex: 0, yAxisIndex: 0,
      data: data.bollinger_upper, smooth: true, symbol: "none",
      lineStyle: { width: 2, type: "dashed", color: "#9ccc65" },
    });
    series.push({
      name: "中軌", type: "line", xAxisIndex: 0, yAxisIndex: 0,
      data: data.bollinger_middle, smooth: true, symbol: "none",
      lineStyle: { width: 1.5, color: "#ffa726" },
    });
    series.push({
      name: "下軌", type: "line", xAxisIndex: 0, yAxisIndex: 0,
      data: data.bollinger_lower, smooth: true, symbol: "none",
      lineStyle: { width: 2, type: "dashed", color: "#ef5350" },
    });
  }

  if (props.showMA !== false && data.ma5.some(v => v !== null)) {
    series.push({ name: "MA5", type: "line", xAxisIndex: 0, yAxisIndex: 0, data: data.ma5, smooth: true, symbol: "none", lineStyle: { width: 1 } });
  }
  if (props.showMA !== false && data.ma10.some(v => v !== null)) {
    series.push({ name: "MA10", type: "line", xAxisIndex: 0, yAxisIndex: 0, data: data.ma10, smooth: true, symbol: "none", lineStyle: { width: 1 } });
  }
  if (props.showMA !== false && data.ma20.some(v => v !== null)) {
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
      data: data.macd_histogram.map((v) => v ?? 0),
      barWidth: "60%",
      itemStyle: { color: (p: any) => (p.value >= 0 ? "#ef5350" : "#26a69a") },
    });
  }

  return {
    title: {
      text: `${data.symbol}${data.name ? ` — ${data.name}` : ""}`,
      left: 70,
      top: 6,
      textStyle: { fontSize: 16, fontWeight: "bold", color: "#333" },
    },
    animationDurationUpdate: 100,
    animationEasingUpdate: "linear",
    tooltip: {
      trigger: "axis",
      axisPointer: { type: "cross" },
      formatter: (params: any[]) => {
        const idx = params[0]?.dataIndex;
        const date = params[0]?.axisValue ?? "";
        const lines: string[] = [`${date}`];
        for (const p of params) {
          if (p.seriesName === "K線" && Array.isArray(p.value)) {
            const { open, close, low, high } = extractOHLC(p.value as number[]);
            lines.push(`開盤價: ${open.toFixed(2)}`);
            lines.push(`收盤價: ${close.toFixed(2)}`);
            lines.push(`最低價: ${low.toFixed(2)}`);
            lines.push(`最高價: ${high.toFixed(2)}`);
          } else if (p.seriesName === "超買" || p.seriesName === "超賣") {
            continue;
          } else {
            const raw = Array.isArray(p.value) ? p.value[0] : p.value;
            lines.push(`${p.seriesName}: ${raw == null ? "-" : Number(raw).toFixed(2)}`);
          }
        }
        if (idx != null) {
          if (data.rsi?.[idx] != null) {
            lines.push(`RSI: ${Number(data.rsi[idx]).toFixed(2)}`);
          }
          if (data.macd_line?.[idx] != null) {
            lines.push(`MACD: ${Number(data.macd_line[idx]).toFixed(2)}`);
          }
          if (data.macd_signal?.[idx] != null) {
            lines.push(`訊號: ${Number(data.macd_signal[idx]).toFixed(2)}`);
          }
          if (data.macd_histogram?.[idx] != null) {
            lines.push(`柱狀: ${Number(data.macd_histogram[idx]).toFixed(2)}`);
          }
        }
        return lines.join("<br/>");
      },
    },
    legend: { top: 26, left: "center", selectedMode: "multiple" },
    grid: grids,
    xAxis: xAxes,
    yAxis: yAxes,
    dataZoom: [{
      type: "inside",
      xAxisIndex: axisIndices,
      startValue: zoomStartValue,
      endValue: total - 0.5,
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
