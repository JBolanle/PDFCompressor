import { describe, it, expect } from "vitest";
import { handleShortcut, type ShortcutState, type ShortcutActions } from "$lib/shortcuts";

function makeEvent(overrides: Partial<{
  metaKey: boolean; shiftKey: boolean; key: string; altKey: boolean; ctrlKey: boolean;
}>): KeyboardEvent {
  return {
    metaKey: false, shiftKey: false, key: "", altKey: false, ctrlKey: false,
    ...overrides,
  } as KeyboardEvent;
}

function makeState(overrides: Partial<ShortcutState> = {}): ShortcutState {
  return { hasPending: false, selectedStatus: null, hasFiles: false, isCompressing: false, ...overrides };
}

function makeActions(): ShortcutActions & { calls: string[] } {
  const calls: string[] = [];
  return {
    calls,
    addFiles:       () => calls.push("addFiles"),
    compress:       () => calls.push("compress"),
    resetSelected:  () => calls.push("resetSelected"),
    revealInFinder: () => calls.push("revealInFinder"),
    clearQueue:     () => calls.push("clearQueue"),
    removeSelected: () => calls.push("removeSelected"),
    selectNext:     () => calls.push("selectNext"),
    selectPrev:     () => calls.push("selectPrev"),
    deselect:       () => calls.push("deselect"),
  };
}

describe("handleShortcut", () => {
  it("cmd+o calls addFiles", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "o" }), makeState(), a);
    expect(a.calls).toContain("addFiles");
  });

  it("cmd+enter calls compress when hasPending and not compressing", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "Enter" }), makeState({ hasPending: true }), a);
    expect(a.calls).toContain("compress");
  });

  it("cmd+enter does nothing when no pending files", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "Enter" }), makeState({ hasPending: false }), a);
    expect(a.calls).not.toContain("compress");
  });

  it("cmd+enter does nothing while compressing", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "Enter" }), makeState({ hasPending: true, isCompressing: true }), a);
    expect(a.calls).not.toContain("compress");
  });

  it("cmd+r resets selected file when status is done", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "r" }), makeState({ selectedStatus: "done" }), a);
    expect(a.calls).toContain("resetSelected");
  });

  it("cmd+r resets selected file when status is error", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "r" }), makeState({ selectedStatus: "error" }), a);
    expect(a.calls).toContain("resetSelected");
  });

  it("cmd+r does nothing when selected file is pending", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, key: "r" }), makeState({ selectedStatus: "pending" }), a);
    expect(a.calls).not.toContain("resetSelected");
  });

  it("cmd+shift+r reveals in Finder when selected is done", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, shiftKey: true, key: "R" }), makeState({ selectedStatus: "done" }), a);
    expect(a.calls).toContain("revealInFinder");
  });

  it("cmd+shift+r does nothing when selected is not done", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, shiftKey: true, key: "R" }), makeState({ selectedStatus: "pending" }), a);
    expect(a.calls).not.toContain("revealInFinder");
  });

  it("cmd+shift+backspace clears queue when files exist", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, shiftKey: true, key: "Backspace" }), makeState({ hasFiles: true }), a);
    expect(a.calls).toContain("clearQueue");
  });

  it("cmd+shift+backspace does nothing when queue is empty", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ metaKey: true, shiftKey: true, key: "Backspace" }), makeState({ hasFiles: false }), a);
    expect(a.calls).not.toContain("clearQueue");
  });

  it("backspace removes selected file", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "Backspace" }), makeState({ selectedStatus: "pending" }), a);
    expect(a.calls).toContain("removeSelected");
  });

  it("backspace does nothing when no file is selected", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "Backspace" }), makeState({ selectedStatus: null }), a);
    expect(a.calls).not.toContain("removeSelected");
  });

  it("arrow down calls selectNext when files exist", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "ArrowDown" }), makeState({ hasFiles: true }), a);
    expect(a.calls).toContain("selectNext");
  });

  it("arrow up calls selectPrev when files exist", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "ArrowUp" }), makeState({ hasFiles: true }), a);
    expect(a.calls).toContain("selectPrev");
  });

  it("escape deselects when a file is selected", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "Escape" }), makeState({ selectedStatus: "done" }), a);
    expect(a.calls).toContain("deselect");
  });

  it("escape does nothing when no file is selected", () => {
    const a = makeActions();
    handleShortcut(makeEvent({ key: "Escape" }), makeState({ selectedStatus: null }), a);
    expect(a.calls).not.toContain("deselect");
  });

  it("returns true when a shortcut is handled", () => {
    const a = makeActions();
    const result = handleShortcut(makeEvent({ metaKey: true, key: "o" }), makeState(), a);
    expect(result).toBe(true);
  });

  it("returns false for an unrecognized key combination", () => {
    const a = makeActions();
    const result = handleShortcut(makeEvent({ key: "z" }), makeState(), a);
    expect(result).toBe(false);
  });
});
