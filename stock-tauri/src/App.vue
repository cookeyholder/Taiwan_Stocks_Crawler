<script setup lang="ts">
import { RouterView, useRouter, useRoute } from "vue-router";

const router = useRouter();
const route = useRoute();

const tabs = [
  { path: "/analysis", label: "單股分析" },
  { path: "/screener", label: "布林擠壓選股器" },
  { path: "/download", label: "下載管理" },
];
</script>

<template>
  <div id="app-root">
    <nav class="tab-bar">
      <button
        v-for="tab in tabs"
        :key="tab.path"
        :class="{ active: route.path === tab.path }"
        @click="router.push(tab.path)"
      >
        {{ tab.label }}
      </button>
    </nav>
    <main>
      <RouterView />
    </main>
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
html, body, #app, #app-root { height: 100%; width: 100%; }
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; font-size: 14px; }
#app-root { display: flex; flex-direction: column; }
.tab-bar {
  display: flex; gap: 0; background: #f5f5f5; border-bottom: 1px solid #ddd; flex-shrink: 0;
}
.tab-bar button {
  padding: 10px 24px; border: none; background: transparent; cursor: pointer;
  font-size: 14px; border-bottom: 2px solid transparent; transition: all 0.2s;
}
.tab-bar button.active { background: #fff; border-bottom-color: #1976d2; color: #1976d2; font-weight: 600; }
.tab-bar button:hover:not(.active) { background: #e8e8e8; }
main { flex: 1; overflow: auto; padding: 16px; }
</style>
