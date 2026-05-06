import type { FileStatus } from "$lib/stores/queueStore";

export interface ShortcutState {
  hasPending: boolean;
  selectedStatus: FileStatus | null;
  hasFiles: boolean;
  isCompressing: boolean;
}

export interface ShortcutActions {
  addFiles: () => void;
  compress: () => void;
  resetSelected: () => void;
  revealInFinder: () => void;
  clearQueue: () => void;
  removeSelected: () => void;
  selectNext: () => void;
  selectPrev: () => void;
  deselect: () => void;
}

export function handleShortcut(
  e: KeyboardEvent,
  state: ShortcutState,
  actions: ShortcutActions
): boolean {
  const { metaKey: cmd, shiftKey: shift, key } = e;
  const k = key.toLowerCase();

  if (cmd && !shift && k === "o") { actions.addFiles(); return true; }
  if (cmd && !shift && key === "Enter" && state.hasPending && !state.isCompressing) { actions.compress(); return true; }
  if (cmd && !shift && k === "r" && (state.selectedStatus === "done" || state.selectedStatus === "error")) { actions.resetSelected(); return true; }
  if (cmd && shift && k === "r" && state.selectedStatus === "done") { actions.revealInFinder(); return true; }
  if (cmd && shift && key === "Backspace" && state.hasFiles) { actions.clearQueue(); return true; }
  if (!cmd && !shift && key === "Backspace" && state.selectedStatus !== null) { actions.removeSelected(); return true; }
  if (!cmd && key === "ArrowDown" && state.hasFiles) { actions.selectNext(); return true; }
  if (!cmd && key === "ArrowUp" && state.hasFiles) { actions.selectPrev(); return true; }
  if (key === "Escape" && state.selectedStatus !== null) { actions.deselect(); return true; }

  return false;
}
