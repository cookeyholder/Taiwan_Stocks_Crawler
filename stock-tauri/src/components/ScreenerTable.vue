<script setup lang="ts">
import type { ScreenResult } from "../stores/screener";
import { ref, computed } from "vue";
import { open } from "@tauri-apps/plugin-shell";

const props = defineProps<{ results: ScreenResult[] }>();
const emit = defineEmits<{ select: [symbol: string] }>();

function openYahoo(symbol: string) {
  open(`https://tw.stock.yahoo.com/quote/${symbol}.TW`);
}

type SortKey = "symbol" | "name" | "latest_close" | "latest_volume" | "max_bandwidth" | "latest_bandwidth" | "break_upper";
const sortKey = ref<SortKey>("symbol");
const sortOrder = ref<"asc" | "desc">("asc");

function toggleSort(key: SortKey) {
  if (sortKey.value === key) {
    sortOrder.value = sortOrder.value === "asc" ? "desc" : "asc";
  } else {
    sortKey.value = key;
    sortOrder.value = "asc";
  }
}

const sorted = computed(() => {
  const arr = [...props.results];
  const asc = sortOrder.value === "asc";
  arr.sort((a, b) => {
    const aIsStock = /^[1-9]\d{3}$/.test(a.symbol);
    const bIsStock = /^[1-9]\d{3}$/.test(b.symbol);
    if (aIsStock !== bIsStock) return aIsStock ? -1 : 1;
    const va = a[sortKey.value];
    const vb = b[sortKey.value];
    if (va == null) return 1;
    if (vb == null) return -1;
    if (va === vb) return 0;
    if (typeof va === "boolean") {
      return asc ? (va ? -1 : 1) : (va ? 1 : -1);
    }
    if (typeof va === "string" && typeof vb === "string") {
      return asc ? va.localeCompare(vb) : vb.localeCompare(va);
    }
    const na = Number(va);
    const nb = Number(vb);
    return asc ? na - nb : nb - na;
  });
  return arr;
});

function sortIndicator(key: SortKey): string {
  if (sortKey.value !== key) return "";
  return sortOrder.value === "asc" ? " ▲" : " ▼";
}

function formatPrice(v: number | null): string {
  return v != null ? v.toFixed(2) : "-";
}
function formatVolume(v: number | null): string {
  return v != null ? v.toLocaleString() : "-";
}
function formatPct(v: number): string {
  return (v * 100).toFixed(2) + "%";
}
</script>

<template>
  <div class="table-wrapper">
    <table>
      <thead>
        <tr>
          <th @click="toggleSort('symbol')" class="sortable">代號{{ sortIndicator('symbol') }}</th>
          <th @click="toggleSort('name')" class="sortable">名稱{{ sortIndicator('name') }}</th>
          <th @click="toggleSort('latest_close')" class="sortable">最新價{{ sortIndicator('latest_close') }}</th>
          <th @click="toggleSort('latest_volume')" class="sortable">成交量{{ sortIndicator('latest_volume') }}</th>
          <th @click="toggleSort('max_bandwidth')" class="sortable">最大頻寬{{ sortIndicator('max_bandwidth') }}</th>
          <th @click="toggleSort('latest_bandwidth')" class="sortable">最新頻寬{{ sortIndicator('latest_bandwidth') }}</th>
          <th @click="toggleSort('break_upper')" class="sortable">突破上通道{{ sortIndicator('break_upper') }}</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="r in sorted"
          :key="r.symbol"
          @click="emit('select', r.symbol)"
          class="clickable"
        >
          <td @click.stop="openYahoo(r.symbol)" class="link">{{ r.symbol }}</td>
          <td @click.stop="openYahoo(r.symbol)" class="link">{{ r.name ?? "-" }}</td>
          <td>{{ formatPrice(r.latest_close) }}</td>
          <td>{{ formatVolume(r.latest_volume) }}</td>
          <td>{{ formatPct(r.max_bandwidth) }}</td>
          <td>{{ formatPct(r.latest_bandwidth) }}</td>
          <td>{{ r.break_upper ? "✓" : "" }}</td>
        </tr>
        <tr v-if="sorted.length === 0">
          <td colspan="7" class="empty-row">尚無資料</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.table-wrapper { overflow: auto; flex: 1; }
table { width: 100%; border-collapse: collapse; font-size: 14px; }
th { position: sticky; top: 0; background: #f5f5f5; padding: 8px 10px; text-align: left; font-weight: 600; border-bottom: 2px solid #ddd; user-select: none; }
th.sortable { cursor: pointer; }
th.sortable:hover { background: #e0e0e0; }
td { padding: 6px 10px; border-bottom: 1px solid #eee; }
tr.clickable { cursor: pointer; }
tr.clickable:hover { background: #e3f2fd; }
td.link { color: #1976d2; text-decoration: underline; cursor: pointer; }
td.link:hover { color: #1565c0; }
.empty-row { text-align: center; color: #999; padding: 40px; }
</style>
