import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { queue } from "$lib/stores/queueStore";
import ActionBar from "$lib/components/ActionBar.svelte";
import { sendNotification } from "@tauri-apps/plugin-notification";
import { waitFor } from "@testing-library/svelte";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn().mockResolvedValue(undefined) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));
vi.mock("@tauri-apps/plugin-notification", () => ({
  isPermissionGranted: vi.fn().mockResolvedValue(true),
  requestPermission: vi.fn(),
  sendNotification: vi.fn(),
}));

describe("ActionBar", () => {
  beforeEach(() => queue.clear());

  it("shows compress button and is disabled when queue is empty", () => {
    render(ActionBar);
    const btn = screen.getByRole("button");
    expect(btn).toBeDisabled();
  });

  it("shows correct file count and is enabled when files are pending", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.addFile({ path: "/tmp/b.pdf", name: "b.pdf", size: 2000 });
    render(ActionBar);
    const btn = screen.getByRole("button", { name: /compress/i });
    expect(btn).not.toBeDisabled();
    expect(btn.textContent).toMatch(/2/);
  });

  it("compress button is disabled when all files are done", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.updateStatus("/tmp/a.pdf", "done");
    render(ActionBar);
    expect(screen.getByRole("button", { name: /compress/i })).toBeDisabled();
  });

  it("shows Clear queue button when all files are processed", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.updateStatus("/tmp/a.pdf", "done");
    render(ActionBar);
    expect(screen.getByRole("button", { name: /clear queue/i })).toBeInTheDocument();
  });

  it("does not show Clear queue button when queue is empty", () => {
    render(ActionBar);
    expect(screen.queryByRole("button", { name: /clear queue/i })).not.toBeInTheDocument();
  });

  it("shows Clear queue button as soon as a file is added", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    render(ActionBar);
    expect(screen.getByRole("button", { name: /clear queue/i })).toBeInTheDocument();
  });

  it("sends a notification after compression completes with done files", async () => {
    let capturedHandler: ((e: { payload: unknown }) => void) | null = null;

    vi.mocked(listen).mockImplementationOnce((_event, handler) => {
      capturedHandler = handler as (e: { payload: unknown }) => void;
      return Promise.resolve(() => {});
    });

    vi.mocked(invoke).mockImplementation(async (cmd: string) => {
      if (cmd === "compress_files" && capturedHandler) {
        capturedHandler({
          payload: { file: "/tmp/a.pdf", status: "done", compressed_size: 600_000 },
        });
      }
      return undefined;
    });

    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1_000_000 });
    render(ActionBar);

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /compress 1 pdf/i }));

    await waitFor(() => {
      expect(sendNotification).toHaveBeenCalledWith({
        title: "compress[pdf]",
        body: "1 PDF compressed — saved 400 KB total",
      });
    });

    vi.mocked(invoke).mockResolvedValue(undefined);
  });
});
