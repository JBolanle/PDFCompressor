import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import { toast } from "$lib/stores/toastStore";

const listeners: Record<string, (e: unknown) => void> = {};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "get_settings")
      return { output_mode: "same_as_source", output_folder: null, naming: "suffix", auto_update_check: false };
    if (cmd === "check_for_update") return null;
    return null;
  }),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (event: string, handler: (e: unknown) => void) => {
    listeners[event] = handler;
    return () => {};
  }),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: vi.fn(),
}));

import Page from "../routes/+page.svelte";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";

describe("update checking", () => {
  beforeEach(() => {
    toast.clear();
    vi.clearAllMocks();
    Object.keys(listeners).forEach((k) => delete listeners[k]);
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings")
        return { output_mode: "same_as_source", output_folder: null, naming: "suffix", auto_update_check: false };
      if (cmd === "check_for_update") return null;
      return null;
    });
  });

  it("does not call check_for_update on mount when auto_update_check is false", async () => {
    render(Page);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("get_settings"));
    expect(invoke).not.toHaveBeenCalledWith("check_for_update");
  });

  it("calls check_for_update on mount when auto_update_check is true", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings")
        return { output_mode: "same_as_source", output_folder: null, naming: "suffix", auto_update_check: true };
      if (cmd === "check_for_update") return null;
      return null;
    });
    render(Page);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("check_for_update"));
  });

  it("shows update toast on mount when auto_update_check is true and update is available", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings")
        return { output_mode: "same_as_source", output_folder: null, naming: "suffix", auto_update_check: true };
      if (cmd === "check_for_update") return "1.4.0";
      return null;
    });
    render(Page);
    await waitFor(() => {
      expect(screen.getByText("v1.4.0 is available")).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Download" })).toBeInTheDocument();
  });

  it("opens releases page when Download is clicked", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings")
        return { output_mode: "same_as_source", output_folder: null, naming: "suffix", auto_update_check: true };
      if (cmd === "check_for_update") return "1.4.0";
      return null;
    });
    render(Page);
    await waitFor(() => screen.getByRole("button", { name: "Download" }));
    screen.getByRole("button", { name: "Download" }).click();
    expect(openUrl).toHaveBeenCalledWith(
      "https://github.com/JBolanle/PDFCompressor/releases/latest"
    );
  });

  it("shows update toast when menu:check-for-update fires and update is available", async () => {
    render(Page);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("get_settings"));

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "check_for_update") return "1.5.0";
      return null;
    });

    listeners["menu:check-for-update"]?.({});

    await waitFor(() => {
      expect(screen.getByText("v1.5.0 is available")).toBeInTheDocument();
    });
  });

  it("shows 'latest version' toast when menu:check-for-update fires and up to date", async () => {
    render(Page);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("get_settings"));

    listeners["menu:check-for-update"]?.({});

    await waitFor(() => {
      expect(screen.getByText("You're on the latest version")).toBeInTheDocument();
    });
  });

  it("saves toggled auto_update_check when menu:check-for-update-auto fires", async () => {
    render(Page);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("get_settings"));

    listeners["menu:check-for-update-auto"]?.({});

    await waitFor(() => {
      expect(invoke).toHaveBeenCalledWith("save_settings", {
        settings: expect.objectContaining({ auto_update_check: true }),
      });
    });
  });
});
