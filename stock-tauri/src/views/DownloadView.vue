<script setup lang="ts">
import { ref, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useDownloadStore } from "../stores/download";

const store = useDownloadStore();
const startDate = ref("");
const endDate = ref("");

async function startDownload() {
  if (!startDate.value || !endDate.value) return;
  await store.startDownload(startDate.value, endDate.value);
}

async function fetchToday() {
  try {
    const msg = await invoke<string>("fetch_today");
    store.result = msg;
  } catch (e) {
    store.result = `Error: ${e}`;
  }
}

onUnmounted(() => store.cleanup());
</script>

<template>
  <div class="download-view">
    <div class="controls-card">
      <h3>下載今日資料</h3>
      <button @click="fetchToday" class="btn-primary">下載今日</button>
    </div>

    <div class="controls-card">
      <h3>批次下載</h3>
      <div class="date-inputs">
        <div class="param-group">
          <label>起始日期</label>
          <input v-model="startDate" type="date" />
        </div>
        <div class="param-group">
          <label>結束日期</label>
          <input v-model="endDate" type="date" />
        </div>
        <button @click="startDownload" :disabled="store.running || !startDate || !endDate" class="btn-primary">
          {{ store.running ? "下載中..." : "開始下載" }}
        </button>
      </div>
    </div>

    <div v-if="store.progress" class="progress-card">
      <h3>下載進度</h3>
      <div class="progress-bar">
        <div class="progress-fill" :style="{ width: (store.progress.current / store.progress.total * 100) + '%' }"></div>
      </div>
      <p class="progress-text">
        {{ store.progress.current }} / {{ store.progress.total }} — {{ store.progress.date }}
      </p>
    </div>

    <div v-if="store.result" class="result-card">
      <h3>結果</h3>
      <pre>{{ store.result }}</pre>
    </div>
  </div>
</template>

<style scoped>
.download-view { display: flex; flex-direction: column; gap: 16px; max-width: 600px; }
.controls-card, .progress-card, .result-card { background: #f9f9f9; border: 1px solid #eee; border-radius: 8px; padding: 16px; }
h3 { font-size: 15px; margin-bottom: 12px; color: #333; }
.date-inputs { display: flex; gap: 12px; align-items: flex-end; flex-wrap: wrap; }
.param-group { display: flex; flex-direction: column; gap: 2px; }
.param-group label { font-size: 12px; color: #666; font-weight: 600; }
.param-group input { padding: 6px 8px; border: 1px solid #ccc; border-radius: 4px; font-size: 14px; }
.btn-primary { padding: 8px 20px; background: #1976d2; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 14px; }
.btn-primary:hover { background: #1565c0; }
.btn-primary:disabled { background: #90caf9; cursor: not-allowed; }
.progress-bar { height: 8px; background: #e0e0e0; border-radius: 4px; overflow: hidden; margin-bottom: 8px; }
.progress-fill { height: 100%; background: #1976d2; transition: width 0.3s; }
.progress-text { font-size: 13px; color: #666; }
pre { font-size: 13px; white-space: pre-wrap; word-break: break-all; }
</style>
