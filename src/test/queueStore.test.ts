import { describe, it, expect, beforeEach } from "vitest";
import { get } from "svelte/store";
import { queue, pendingCount } from "../lib/stores/queueStore";

describe("queueStore", () => {
  beforeEach(() => queue.clear());

  it("addFile appends an entry with pending status and balanced preset", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    const entries = get(queue);
    expect(entries).toHaveLength(1);
    expect(entries[0].name).toBe("a.pdf");
    expect(entries[0].status).toBe("pending");
    expect(entries[0].preset).toBe("balanced");
    expect(typeof entries[0].id).toBe("string");
  });

  it("rejects duplicate paths silently", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    expect(get(queue)).toHaveLength(1);
  });

  it("removeFile removes by id", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    const id = get(queue)[0].id;
    queue.removeFile(id);
    expect(get(queue)).toHaveLength(0);
  });

  it("updateStatus mutates the correct entry", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.addFile({ path: "/tmp/b.pdf", name: "b.pdf", size: 2000 });
    queue.updateStatus("/tmp/a.pdf", "done", { compressedSize: 500 });
    const [a, b] = get(queue);
    expect(a.status).toBe("done");
    expect(a.compressedSize).toBe(500);
    expect(b.status).toBe("pending");
  });

  it("updatePreset changes preset and dpiOverride for the given id", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    const id = get(queue)[0].id;
    queue.updatePreset(id, "max", 60);
    const entry = get(queue)[0];
    expect(entry.preset).toBe("max");
    expect(entry.dpiOverride).toBe(60);
  });

  it("pendingCount derived store counts only pending entries", () => {
    queue.addFile({ path: "/tmp/a.pdf", name: "a.pdf", size: 1000 });
    queue.addFile({ path: "/tmp/b.pdf", name: "b.pdf", size: 2000 });
    queue.updateStatus("/tmp/a.pdf", "done");
    expect(get(pendingCount)).toBe(1);
  });
});
