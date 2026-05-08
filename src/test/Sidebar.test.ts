import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import { queue } from "$lib/stores/queueStore";
import { selectedFileId } from "$lib/stores/selectionStore";
import Sidebar from "$lib/components/Sidebar.svelte";

const dropListeners: Array<(e: { payload: { paths: string[] } }) => void> = [];
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn().mockResolvedValue(true) }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn().mockResolvedValue(null) }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (event: string, handler: (e: unknown) => void) => {
    if (event === "tauri://drag-drop") dropListeners.push(handler as never);
    return () => {};
  }),
}));

describe("Sidebar", () => {
  beforeEach(() => {
    queue.clear();
    selectedFileId.set(null);
  });

  it("renders empty state when queue is empty", () => {
    render(Sidebar);
    expect(screen.getByText(/drop pdfs/i)).toBeInTheDocument();
  });

  it("adds files via tauri://drag-drop event", async () => {
    render(Sidebar);
    await waitFor(() => expect(dropListeners.length).toBeGreaterThan(0));
    dropListeners[0]({ payload: { paths: ["/tmp/a.pdf"] } });
    await waitFor(() => expect(screen.getByText("a.pdf")).toBeInTheDocument());
  });

  it("renders one row per file in the queue", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.addFile({ path: "/tmp/b.pdf", name: "b.pdf", size: 2000 });
    render(Sidebar);
    expect(screen.getByText("a.pdf")).toBeInTheDocument();
    expect(screen.getByText("b.pdf")).toBeInTheDocument();
  });
});
