import { describe, it, expect } from "vitest";
import { buildNotificationBody, formatSavedBytes } from "$lib/notification";

describe("formatSavedBytes", () => {
  it("formats bytes under 1 KB", () => {
    expect(formatSavedBytes(500)).toBe("500 B");
  });

  it("formats KB (rounds to nearest KB)", () => {
    expect(formatSavedBytes(840_000)).toBe("840 KB");
  });

  it("formats MB to one decimal place", () => {
    expect(formatSavedBytes(2_400_000)).toBe("2.4 MB");
  });

  it("formats exactly 1 MB", () => {
    expect(formatSavedBytes(1_000_000)).toBe("1.0 MB");
  });
});

describe("buildNotificationBody", () => {
  it("singular noun when 1 PDF succeeds with no errors", () => {
    expect(buildNotificationBody(1, 0, 840_000)).toBe(
      "1 PDF compressed — saved 840 KB total"
    );
  });

  it("plural noun when 3 PDFs succeed with no errors", () => {
    expect(buildNotificationBody(3, 0, 2_400_000)).toBe(
      "3 PDFs compressed — saved 2.4 MB total"
    );
  });

  it("shows mixed result with done and error counts", () => {
    expect(buildNotificationBody(2, 1, 0)).toBe(
      "2 of 3 PDFs compressed — 1 failed"
    );
  });

  it("shows mixed result when only 1 succeeded", () => {
    expect(buildNotificationBody(1, 2, 500_000)).toBe(
      "1 of 3 PDFs compressed — 2 failed"
    );
  });

  it("handles zero saved bytes in success case", () => {
    expect(buildNotificationBody(1, 0, 0)).toBe(
      "1 PDF compressed — saved 0 B total"
    );
  });
});
