import { describe, it, expect, vi, beforeEach } from "vitest";
import { get } from "svelte/store";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    output_mode: "custom_folder",
    output_folder: "/my/folder",
    naming: "overwrite",
    auto_update_check: true,
    default_preset: "balanced",
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
    expect(s.auto_update_check).toBe(false);
  });

  it("load() calls get_settings and updates the store", async () => {
    await settings.load();
    expect(invoke).toHaveBeenCalledWith("get_settings");
    const s = get(settings);
    expect(s.output_mode).toBe("custom_folder");
    expect(s.output_folder).toBe("/my/folder");
    expect(s.naming).toBe("overwrite");
    expect(s.auto_update_check).toBe(true);
  });

  it("save() calls save_settings with current value", async () => {
    const newSettings = {
      output_mode: "same_as_source" as const,
      output_folder: null,
      naming: "suffix" as const,
      auto_update_check: false,
      default_preset: "balanced" as const,
    };
    await settings.save(newSettings);
    expect(invoke).toHaveBeenCalledWith("save_settings", { settings: newSettings });
    expect(get(settings)).toEqual(newSettings);
  });
});
