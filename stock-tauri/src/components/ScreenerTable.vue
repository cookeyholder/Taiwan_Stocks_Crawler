<script setup lang="ts">
import type { ScreenResult } from "../stores/screener";
import { computed } from "vue";

const props = defineProps<{ results: ScreenResult[] }>();
const emit = defineEmits<{ select: [symbol: string] }>();

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
          <th>代號</th>
          <th>名稱</th>
          <th>最新價</th>
          <th>成交量</th>
          <th>最大頻寬</th>
          <th>最新頻寬</th>
        </tr>
      </thead>
      <tbody>
        <tr
          v-for="r in results"
          :key="r.symbol"
          @click="emit('select', r.symbol)"
          class="clickable"
        >
          <td>{{ r.symbol }}</td>
          <td>{{ r.name ?? "-" }}</td>
          <td>{{ formatPrice(r.latest_close) }}</td>
          <td>{{ formatVolume(r.latest_volume) }}</td>
          <td>{{ formatPct(r.max_bandwidth) }}</td>
          <td>{{ formatPct(r.latest_bandwidth) }}</td>
        </tr>
        <tr v-if="results.length === 0">
          <td colspan="6" class="empty-row">尚無資料</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.table-wrapper { overflow: auto; flex: 1; }
table { width: 100%; border-collapse: collapse; font-size: 13px; }
th { position: sticky; top: 0; background: #f5f5f5; padding: 8px 10px; text-align: left; font-weight: 600; border-bottom: 2px solid #ddd; }
td { padding: 6px 10px; border-bottom: 1px solid #eee; }
tr.clickable { cursor: pointer; }
tr.clickable:hover { background: #e3f2fd; }
.empty-row { text-align: center; color: #999; padding: 40px; }
</style>
