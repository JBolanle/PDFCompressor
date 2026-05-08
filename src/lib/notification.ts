export function formatBytes(bytes: number): string {
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
  if (bytes >= 1_000) return `${Math.round(bytes / 1_000)} KB`;
  return `${bytes} B`;
}

export function buildNotificationBody(
  doneCount: number,
  errorCount: number,
  savedBytes: number
): string {
  if (errorCount === 0) {
    const noun = doneCount === 1 ? "PDF" : "PDFs";
    return `${doneCount} ${noun} compressed — saved ${formatBytes(savedBytes)} total`;
  }
  const total = doneCount + errorCount;
  return `${doneCount} of ${total} PDFs compressed — ${errorCount} failed`;
}
