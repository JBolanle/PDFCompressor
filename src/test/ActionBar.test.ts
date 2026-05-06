import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import { queue } from "$lib/stores/queueStore";
import ActionBar from "$lib/components/ActionBar.svelte";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn().mockResolvedValue(undefined) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));

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
    const btn = screen.getByRole("button");
    expect(btn).not.toBeDisabled();
    expect(btn.textContent).toMatch(/2/);
  });

  it("is disabled when all files are done", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.updateStatus("/tmp/a.pdf", "done");
    render(ActionBar);
    expect(screen.getByRole("button")).toBeDisabled();
  });
});
