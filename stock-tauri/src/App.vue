<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { RouterView, useRouter, useRoute } from "vue-router";

const router = useRouter();
const route = useRoute();

const now = ref(new Date());
let timer: ReturnType<typeof setInterval>;

onMounted(() => { timer = setInterval(() => { now.value = new Date(); }, 1000); });
onUnmounted(() => clearInterval(timer));

function pad(n: number): string { return n.toString().padStart(2, "0"); }

const tabs = [
  { path: "/analysis", label: "個股分析" },
  { path: "/screener", label: "布林通道選股" },
  { path: "/download", label: "下載管理" },
];
</script>

<template>
  <div id="app-root">
    <nav class="tab-bar">
      <div class="tab-group">
        <button
          v-for="tab in tabs"
          :key="tab.path"
          :class="{ active: route.path === tab.path }"
          @click="router.push(tab.path)"
        >
          {{ tab.label }}
        </button>
      </div>
      <div class="clock">
        {{ now.getFullYear() }}-{{ pad(now.getMonth()+1) }}-{{ pad(now.getDate()) }}
        {{ pad(now.getHours()) }}:{{ pad(now.getMinutes()) }}:{{ pad(now.getSeconds()) }}
      </div>
    </nav>
    <main>
      <RouterView />
    </main>
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
html, body, #app, #app-root { height: 100%; width: 100%; }
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; font-size: 15px; }
#app-root { display: flex; flex-direction: column; }
.tab-bar {
  display: flex; align-items: center; justify-content: space-between;
  background: #f5f5f5; border-bottom: 1px solid #ddd; flex-shrink: 0;
}
.tab-group { display: flex; gap: 0; }
.tab-bar button {
  padding: 10px 24px; border: none; background: transparent; cursor: pointer;
  font-size: 15px; border-bottom: 2px solid transparent; transition: all 0.2s;
}
.tab-bar button.active { background: #fff; border-bottom-color: #1976d2; color: #1976d2; font-weight: 600; }
.tab-bar button:hover:not(.active) { background: #e8e8e8; }
.clock {
  padding: 10px 20px; font-size: 16px; font-weight: 600; font-variant-numeric: tabular-nums;
  color: #333; letter-spacing: 1px; user-select: none;
}
main { flex: 1; overflow: auto; padding: 16px; }
</style>
