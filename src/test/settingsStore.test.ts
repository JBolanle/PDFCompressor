import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";

// Mock @tauri-apps/api/core BEFORE importing the store
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    output_mode: "custom_folder",
    output_folder: "/my/folder",
    naming: "overwrite",
  }),
}));

import { settings } from "../lib/stores/settingsStore";
import { invoke } from "@tauri-apps/api/core";

describe("settingsStore", () => {
  beforeEach(() => vi.clearAllMocks());

  it("has correct default values before load", () => {
    const s = get(settings);
    expect(s.output_mode).toBe("same_as_source");
    expect(s.output_folder).toBeNull();
    expect(s.naming).toBe("suffix");
  });

  it("load() calls get_settings and updates the store", async () => {
    await settings.load();
    expect(invoke).toHaveBeenCalledWith("get_settings");
    const s = get(settings);
    expect(s.output_mode).toBe("custom_folder");
    expect(s.output_folder).toBe("/my/folder");
    expect(s.naming).toBe("overwrite");
  });

  it("save() calls save_settings with current value", async () => {
    const newSettings = {
      output_mode: "same_as_source" as const,
      output_folder: null,
      naming: "suffix" as const,
    };
    await settings.save(newSettings);
    expect(invoke).toHaveBeenCalledWith("save_settings", { settings: newSettings });
    expect(get(settings)).toEqual(newSettings);
  });
});
