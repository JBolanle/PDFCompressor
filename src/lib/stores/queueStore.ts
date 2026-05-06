import { writable, derived } from "svelte/store";

export type FileStatus = "pending" | "processing" | "done" | "error";
export type Preset = "max" | "balanced" | "minimal";

export interface FileEntry {
  id: string;
  path: string;
  name: string;
  size: number;
  status: FileStatus;
  preset: Preset;
  dpiOverride?: number;
  compressedSize?: number;
  errorMsg?: string;
}

function createQueueStore() {
  const { subscribe, update, set } = writable<FileEntry[]>([]);

  return {
    subscribe,
    addFile(file: Omit<FileEntry, "id" | "status" | "preset">) {
      update((entries) => {
        if (entries.some((e) => e.path === file.path)) return entries;
        return [...entries, { ...file, id: crypto.randomUUID(), status: "pending", preset: "balanced" }];
      });
    },
    removeFile(id: string) {
      update((entries) => entries.filter((e) => e.id !== id));
    },
    updateStatus(path: string, status: FileStatus, extra: Partial<FileEntry> = {}) {
      update((entries) =>
        entries.map((e) => (e.path === path ? { ...e, status, ...extra } : e))
      );
    },
    updatePreset(id: string, preset: Preset, dpiOverride?: number) {
      update((entries) =>
        entries.map((e) => (e.id === id ? { ...e, preset, dpiOverride } : e))
      );
    },
    updateAllPresets(preset: Preset, dpiOverride?: number) {
      update((entries) =>
        entries.map((e) => (e.status === "pending" ? { ...e, preset, dpiOverride } : e))
      );
    },
    clear() {
      set([]);
    },
  };
}

export const queue = createQueueStore();
export const pendingCount = derived(queue, ($q) => $q.filter((e) => e.status === "pending").length);
export const allFinished = derived(queue, ($q) => $q.length > 0 && $q.every((e) => e.status === "done" || e.status === "error"));
