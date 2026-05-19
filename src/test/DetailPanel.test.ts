import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { get } from "svelte/store";
import { queue } from "$lib/stores/queueStore";
import { selectedFileId } from "$lib/stores/selectionStore";
import { settings } from "$lib/stores/settingsStore";
import DetailPanel from "$lib/components/DetailPanel.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/plugin-dialog", () => ({
  open: vi.fn().mockResolvedValue(null),
}));

import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";

describe("DetailPanel", () => {
  beforeEach(async () => {
    queue.clear();
    selectedFileId.set(null);
    await settings.save({ output_mode: "same_as_source", output_folder: null, naming: "suffix", auto_update_check: false, default_preset: "balanced" });
    vi.clearAllMocks();
  });

  // ── existing tests ────────────────────────────────────────────────────────

  it("shows file name pre-compression", () => {
    queue.addFile({ path: "/tmp/report.pdf", name: "report.pdf", size: 3_200_000 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    expect(screen.getByText("report.pdf")).toBeInTheDocument();
  });

  it("shows Show in Finder button when status is done", () => {
    queue.addFile({ path: "/tmp/report.pdf", name: "report.pdf", size: 3_200_000 });
    queue.updateStatus("/tmp/report.pdf", "done", { compressedSize: 1_100_000 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    expect(screen.getByText(/show in finder/i)).toBeInTheDocument();
  });

  it("shows Apply DPI to all files button when a pending file is selected", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    expect(screen.getByRole("button", { name: /apply dpi to all files/i })).toBeInTheDocument();
  });

  it("Apply DPI to all files updates preset on all other pending files", async () => {
    const user = userEvent.setup();
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.addFile({ path: "/tmp/b.pdf", name: "b.pdf", size: 2000 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    await user.click(screen.getByRole("button", { name: /apply dpi to all files/i }));
    expect(get(queue)[1].preset).toBe("balanced");
  });

  it("does not show Apply DPI to all files button when selected file is done", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.updateStatus("/tmp/a.pdf", "done", { compressedSize: 500 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    expect(screen.queryByRole("button", { name: /apply dpi to all files/i })).not.toBeInTheDocument();
  });

  // ── settings section ──────────────────────────────────────────────────────

  it("shows settings section when no file is selected", () => {
    render(DetailPanel);
    expect(screen.getByText("Output")).toBeInTheDocument();
    expect(screen.getByText("Naming")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /advanced/i })).toBeInTheDocument();
  });

  it("shows settings section when a file is selected", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    selectedFileId.set(get(queue)[0].id);
    render(DetailPanel);
    expect(screen.getByText("Output")).toBeInTheDocument();
    expect(screen.getByText("Naming")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /advanced/i })).toBeInTheDocument();
  });

  it("Default preset is hidden inside the Advanced drawer by default", () => {
    render(DetailPanel);
    expect(screen.queryByText("Default preset")).not.toBeInTheDocument();
    const toggle = screen.getByRole("button", { name: /advanced/i });
    expect(toggle).toHaveAttribute("aria-expanded", "false");
  });

  it("clicking Advanced reveals the Default preset section", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    await user.click(screen.getByRole("button", { name: /advanced/i }));
    expect(screen.getByText("Default preset")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /advanced/i })).toHaveAttribute("aria-expanded", "true");
  });

  it("changing output mode to custom_folder saves immediately", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    await user.click(screen.getByRole("radio", { name: /custom folder/i }));
    expect(invoke).toHaveBeenCalledWith("save_settings", {
      settings: expect.objectContaining({ output_mode: "custom_folder" }),
    });
  });

  it("changing file naming to overwrite saves immediately", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    await user.click(screen.getByRole("radio", { name: /overwrite original/i }));
    expect(invoke).toHaveBeenCalledWith("save_settings", {
      settings: expect.objectContaining({ naming: "overwrite" }),
    });
  });

  it("shows Choose button when output mode is custom_folder", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    await user.click(screen.getByRole("radio", { name: /custom folder/i }));
    expect(screen.getByRole("button", { name: /choose/i })).toBeInTheDocument();
  });

  it("does not show Choose button when output mode is same_as_source", () => {
    render(DetailPanel);
    expect(screen.queryByRole("button", { name: /choose/i })).not.toBeInTheDocument();
  });

  it("pickFolder saves the selected folder path immediately", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    // Switch to custom_folder mode so Choose… button appears
    await user.click(screen.getByRole("radio", { name: /custom folder/i }));
    vi.mocked(open).mockResolvedValueOnce("/Users/me/Documents");
    await user.click(screen.getByRole("button", { name: /choose/i }));
    expect(invoke).toHaveBeenCalledWith("save_settings", {
      settings: expect.objectContaining({ output_folder: "/Users/me/Documents" }),
    });
  });

  it("Advanced drawer reveals Default preset radios after expand", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    await user.click(screen.getByRole("button", { name: /advanced/i }));
    expect(screen.getByText("Default preset")).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: /^Max/i })).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: /^Balanced/i })).toBeInTheDocument();
    expect(screen.getByRole("radio", { name: /^Minimal/i })).toBeInTheDocument();
  });

  it("changing default preset to max saves immediately", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    await user.click(screen.getByRole("button", { name: /advanced/i }));
    await user.click(screen.getByRole("radio", { name: /^Max/i }));
    expect(invoke).toHaveBeenCalledWith("save_settings", {
      settings: expect.objectContaining({ default_preset: "max" }),
    });
  });

  it("changing default preset to minimal saves immediately", async () => {
    const user = userEvent.setup();
    render(DetailPanel);
    await user.click(screen.getByRole("button", { name: /advanced/i }));
    await user.click(screen.getByRole("radio", { name: /^Minimal/i }));
    expect(invoke).toHaveBeenCalledWith("save_settings", {
      settings: expect.objectContaining({ default_preset: "minimal" }),
    });
  });
});
