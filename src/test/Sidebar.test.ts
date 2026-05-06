import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import { queue } from "$lib/stores/queueStore";
import { selectedFileId } from "$lib/stores/selectionStore";
import Sidebar from "$lib/components/Sidebar.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn().mockResolvedValue(true) }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn().mockResolvedValue(null) }));

describe("Sidebar", () => {
  beforeEach(() => {
    queue.clear();
    selectedFileId.set(null);
  });

  it("renders empty state when queue is empty", () => {
    render(Sidebar);
    expect(screen.getByText(/drop pdfs/i)).toBeInTheDocument();
  });

  it("renders one row per file in the queue", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.addFile({ path: "/tmp/b.pdf", name: "b.pdf", size: 2000 });
    render(Sidebar);
    expect(screen.getByText("a.pdf")).toBeInTheDocument();
    expect(screen.getByText("b.pdf")).toBeInTheDocument();
  });
});
