import { defineStore } from "pinia";
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface DownloadProgress {
  current: number;
  total: number;
  date: string;
}

export const useDownloadStore = defineStore("download", () => {
  const progress = ref<DownloadProgress | null>(null);
  const running = ref(false);
  const result = ref<string | null>(null);

  let unlisten: (() => void) | null = null;

  async function startDownload(startDate: string, endDate: string) {
    running.value = true;
    result.value = null;
    progress.value = null;

    unlisten = await listen<DownloadProgress>("download-progress", (event) => {
      progress.value = event.payload;
    });

    try {
      result.value = await invoke<string>("batch_download", { startDate, endDate });
    } catch (e) {
      result.value = `Error: ${e}`;
    } finally {
      running.value = false;
      if (unlisten) {
        unlisten();
        unlisten = null;
      }
    }
  }

  function cleanup() {
    if (unlisten) {
      unlisten();
      unlisten = null;
    }
  }

  return { progress, running, result, startDownload, cleanup };
});
