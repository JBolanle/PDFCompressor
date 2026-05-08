import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { queue } from "$lib/stores/queueStore";

export function basename(path: string): string {
  return path.split("/").pop() ?? path;
}

export async function addPath(path: string): Promise<void> {
  const name = basename(path);
  try {
    const isPdf = await invoke<boolean>("validate_pdf", { path });
    if (!isPdf) {
      queue.addFile({ path, name, size: 0 });
      queue.updateStatus(path, "error", { errorMsg: "Not a valid PDF file" });
      return;
    }
    const meta = await invoke<{ size: number }>("get_file_meta", { path });
    queue.addFile({ path, name, size: meta.size });
  } catch {
    queue.addFile({ path, name, size: 0 });
  }
}

export async function addFiles(): Promise<void> {
  const paths = await open({ multiple: true, filters: [{ name: "PDF", extensions: ["pdf"] }] });
  if (!paths) return;
  const list = Array.isArray(paths) ? paths : [paths];
  for (const path of list) await addPath(path);
}

export function revealInFinder(path: string): void {
  invoke("reveal_in_finder", { path });
}
