import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import { toast } from "$lib/stores/toastStore";

// Capture listen handlers so tests can trigger menu events
const listeners: Record<string, (e: unknown) => void> = {};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "get_settings") return { output_mode: "same_as_source", output_folder: null, naming: "suffix" };
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
    // Reset mock to default (no update)
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return { output_mode: "same_as_source", output_folder: null, naming: "suffix" };
      if (cmd === "check_for_update") return null;
      return null;
    });
  });

  it("shows a persistent update toast on mount when update is available", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return { output_mode: "same_as_source", output_folder: null, naming: "suffix" };
      if (cmd === "check_for_update") return "1.4.0";
      return null;
    });

    render(Page);

    await waitFor(() => {
      expect(screen.getByText("v1.4.0 is available")).toBeInTheDocument();
    });
    expect(screen.getByRole("button", { name: "Download" })).toBeInTheDocument();
  });

  it("shows no update toast on mount when already up to date", async () => {
    render(Page);
    // Wait for mount to settle
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("check_for_update"));
    expect(screen.queryByText(/is available/)).not.toBeInTheDocument();
  });

  it("opens releases page when Download is clicked", async () => {
    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "get_settings") return { output_mode: "same_as_source", output_folder: null, naming: "suffix" };
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
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("check_for_update"));

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
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("check_for_update"));

    listeners["menu:check-for-update"]?.({});

    await waitFor(() => {
      expect(screen.getByText("You're on the latest version")).toBeInTheDocument();
    });
  });
});
