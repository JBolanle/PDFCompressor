import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import { get } from "svelte/store";
import { queue } from "$lib/stores/queueStore";
import DetailPanel from "$lib/components/DetailPanel.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("DetailPanel", () => {
  beforeEach(() => queue.clear());

  it("shows placeholder when no file is selected", () => {
    render(DetailPanel, { props: { selectedFileId: null } });
    expect(screen.getByText(/select a file/i)).toBeInTheDocument();
  });

  it("shows file name pre-compression", () => {
    queue.addFile({ path: "/tmp/report.pdf", name: "report.pdf", size: 3_200_000 });
    const id = get(queue)[0].id;
    render(DetailPanel, { props: { selectedFileId: id } });
    expect(screen.getByText("report.pdf")).toBeInTheDocument();
  });

  it("shows Show in Finder button when status is done", () => {
    queue.addFile({ path: "/tmp/report.pdf", name: "report.pdf", size: 3_200_000 });
    queue.updateStatus("/tmp/report.pdf", "done", { compressedSize: 1_100_000 });
    const id = get(queue)[0].id;
    render(DetailPanel, { props: { selectedFileId: id } });
    expect(screen.getByText(/show in finder/i)).toBeInTheDocument();
  });
});
