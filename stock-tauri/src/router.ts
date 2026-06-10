import { createRouter, createWebHistory } from "vue-router";
import AnalysisView from "./views/AnalysisView.vue";
import ScreenerView from "./views/ScreenerView.vue";
import DownloadView from "./views/DownloadView.vue";

export default createRouter({
  history: createWebHistory(),
  routes: [
    { path: "/", redirect: "/analysis" },
    { path: "/analysis", component: AnalysisView, meta: { title: "單股分析" } },
    { path: "/screener", component: ScreenerView, meta: { title: "布林擠壓選股器" } },
    { path: "/download", component: DownloadView, meta: { title: "下載管理" } },
  ],
});
