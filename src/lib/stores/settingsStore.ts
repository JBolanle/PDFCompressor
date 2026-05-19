import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type Preset = "max" | "balanced" | "minimal";

export interface AppSettings {
  output_mode: "same_as_source" | "custom_folder";
  output_folder: string | null;
  naming: "suffix" | "overwrite";
  auto_update_check: boolean;
  default_preset: Preset;
}

const DEFAULT_SETTINGS: AppSettings = {
  output_mode: "same_as_source",
  output_folder: null,
  naming: "suffix",
  auto_update_check: false,
  default_preset: "balanced",
};

function createSettingsStore() {
  const { subscribe, set } = writable<AppSettings>({ ...DEFAULT_SETTINGS });

  return {
    subscribe,
    async load() {
      try {
        const s = await invoke<AppSettings>("get_settings");
        set(s);
      } catch {
        set({ ...DEFAULT_SETTINGS });
      }
    },
    async save(s: AppSettings) {
      set(s);
      await invoke("save_settings", { settings: s });
    },
  };
}

export const settings = createSettingsStore();
