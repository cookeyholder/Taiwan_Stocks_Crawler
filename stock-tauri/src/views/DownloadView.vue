<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useDownloadStore } from "../stores/download";

const store = useDownloadStore();
const startDate = ref("");
const endDate = ref("");
const deleteStart = ref("");
const deleteEnd = ref("");
const deleteMessage = ref("");

const dbFirstDate = ref("");
const dbLastDate = ref("");
const dbDayCount = ref(0);
const dbLoading = ref(true);

function getTodayStr(): string {
  const now = new Date();
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone: "Asia/Taipei",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(now);
  const get = (t: string) => parts.find((p) => p.type === t)!.value;
  return `${get("year")}-${get("month")}-${get("day")}`;
}

function calcNextDay(isoDate: string): string {
  const [y, m, d] = isoDate.split("-").map(Number);
  const date = new Date(y, m - 1, d);
  date.setDate(date.getDate() + 1);
  const yy = date.getFullYear();
  const mm = String(date.getMonth() + 1).padStart(2, "0");
  const dd = String(date.getDate()).padStart(2, "0");
  return `${yy}-${mm}-${dd}`;
}

async function refreshDbInfo() {
  try {
    const [min, max, count] = await invoke<[string | null, string | null, number]>("get_date_range");
    dbFirstDate.value = min ?? "";
    dbLastDate.value = max ?? "";
    dbDayCount.value = count;
  } catch {
  }
}

onMounted(async () => {
  dbLoading.value = true;
  await refreshDbInfo();
  dbLoading.value = false;
});

async function startDownload() {
  if (!startDate.value || !endDate.value) return;
  await store.startDownload(startDate.value, endDate.value);
  await refreshDbInfo();
}

async function fillToToday() {
  if (store.running) return;
  store.result = null;
  try {
    const latest = await invoke<string | null>("get_latest_trade_date");
    if (!latest) {
      store.result = "資料庫尚無資料，請先手動指定日期下載";
      return;
    }
    const today = getTodayStr();
    if (latest >= today) {
      store.result = "資料已是最新";
      return;
    }
    const nextDay = calcNextDay(latest);
    await store.startDownload(nextDay, today);
    await refreshDbInfo();
  } catch (e) {
    store.result = `Error: ${e}`;
  }
}

async function deleteRange() {
  if (!deleteStart.value || !deleteEnd.value) return;
  if (!confirm(`確定要刪除 ${deleteStart.value} ~ ${deleteEnd.value} 的資料嗎？`)) return;
  deleteMessage.value = "刪除中...";
  try {
    const msg = await invoke<string>("delete_date_range", {
      start: deleteStart.value,
      end: deleteEnd.value,
    });
    deleteMessage.value = msg;
    await refreshDbInfo();
  } catch (e) {
    deleteMessage.value = `錯誤: ${e}`;
  }
}

onUnmounted(() => store.cleanup());
</script>

<template>
  <div class="download-view">
    <div class="info-card">
      <h3>資料庫概況</h3>
      <div v-if="dbLoading" class="info-text">載入中...</div>
      <div v-else-if="dbDayCount === 0" class="info-text">資料庫尚無資料</div>
      <div v-else class="info-text">
        <span class="info-item">最早日期：<strong>{{ dbFirstDate }}</strong></span>
        <span class="info-item">最晚日期：<strong>{{ dbLastDate }}</strong></span>
        <span class="info-item">交易日數：<strong>{{ dbDayCount }} 天</strong></span>
      </div>
    </div>

    <div class="controls-card">
      <h3>補至今日資料</h3>
      <button @click="fillToToday" :disabled="store.running" class="btn-primary">補至今日資料</button>
    </div>

    <div class="controls-card">
      <h3>批次下載</h3>
      <div class="date-inputs">
        <div class="param-group">
          <label>起始日期</label>
          <input v-model="startDate" type="text" placeholder="YYYY-MM-DD" maxlength="10" />
        </div>
        <div class="param-group">
          <label>結束日期</label>
          <input v-model="endDate" type="text" placeholder="YYYY-MM-DD" maxlength="10" />
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

    <div class="controls-card danger-card">
      <h3>刪除資料</h3>
      <div class="date-inputs">
        <div class="param-group">
          <label>起始日期</label>
          <input v-model="deleteStart" type="text" placeholder="YYYY-MM-DD" maxlength="10" />
        </div>
        <div class="param-group">
          <label>結束日期</label>
          <input v-model="deleteEnd" type="text" placeholder="YYYY-MM-DD" maxlength="10" />
        </div>
        <button @click="deleteRange" :disabled="!deleteStart || !deleteEnd" class="btn-danger">
          刪除區間
        </button>
      </div>
      <p v-if="deleteMessage" class="delete-message">{{ deleteMessage }}</p>
    </div>

    <div v-if="store.result" class="result-card">
      <h3>結果</h3>
      <pre>{{ store.result }}</pre>
    </div>
  </div>
</template>

<style scoped>
.download-view { display: flex; flex-direction: column; gap: 16px; }
.info-card { background: #e3f2fd; border: 1px solid #90caf9; border-radius: 8px; padding: 16px; }
.info-text { font-size: 15px; color: #333; display: flex; gap: 20px; flex-wrap: wrap; }
.controls-card, .progress-card, .result-card { background: #f9f9f9; border: 1px solid #eee; border-radius: 8px; padding: 16px; }
h3 { font-size: 16px; margin-bottom: 12px; color: #333; }
.date-inputs { display: flex; gap: 12px; align-items: flex-end; flex-wrap: wrap; }
.param-group { display: flex; flex-direction: column; gap: 2px; }
.param-group label { font-size: 14px; color: #666; font-weight: 600; }
.param-group input { padding: 6px 8px; border: 1px solid #ccc; border-radius: 4px; font-size: 15px; }
.btn-primary { padding: 8px 20px; background: #1976d2; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 15px; }
.btn-primary:hover { background: #1565c0; }
.btn-primary:disabled { background: #90caf9; cursor: not-allowed; }
.danger-card { border-color: #ef9a9a; }
.btn-danger { padding: 8px 20px; background: #d32f2f; color: #fff; border: none; border-radius: 4px; cursor: pointer; font-size: 15px; }
.btn-danger:hover { background: #b71c1c; }
.btn-danger:disabled { background: #ef9a9a; cursor: not-allowed; }
.delete-message { margin-top: 8px; font-size: 14px; color: #d32f2f; }
.progress-bar { height: 8px; background: #e0e0e0; border-radius: 4px; overflow: hidden; margin-bottom: 8px; }
.progress-fill { height: 100%; background: #1976d2; transition: width 0.3s; }
.progress-text { font-size: 14px; color: #666; }
pre { font-size: 14px; white-space: pre-wrap; word-break: break-all; }
</style>
