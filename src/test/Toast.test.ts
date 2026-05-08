import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/svelte";
import { fireEvent } from "@testing-library/svelte";
import { toast } from "$lib/stores/toastStore";
import Toast from "$lib/components/Toast.svelte";

describe("Toast", () => {
  beforeEach(() => toast.clear());

  it("shows a message when toast.show() is called", async () => {
    render(Toast);
    toast.show("Test error message");
    await waitFor(() => {
      expect(screen.getByText("Test error message")).toBeInTheDocument();
    });
  });

  it("dismisses when the dismiss button is clicked", async () => {
    render(Toast);
    toast.show("Click to dismiss");
    await waitFor(() => screen.getByText("Click to dismiss"));
    const dismissBtn = screen.getByRole("button", { name: /dismiss/i });
    await fireEvent.click(dismissBtn);
    await waitFor(() => {
      expect(screen.queryByText("Click to dismiss")).not.toBeInTheDocument();
    });
  });

  it("showPersistent message stays after 4 s", async () => {
    vi.useFakeTimers();
    render(Toast);
    toast.showPersistent("Persistent message");
    await waitFor(() => screen.getByText("Persistent message"));
    vi.advanceTimersByTime(5000);
    await waitFor(() => {
      expect(screen.getByText("Persistent message")).toBeInTheDocument();
    });
    vi.useRealTimers();
  });

  it("showPersistent renders an action button", async () => {
    render(Toast);
    toast.showPersistent("Update available", { label: "Download", handler: vi.fn() });
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Download" })).toBeInTheDocument();
    });
  });

  it("showPersistent action button calls handler on click", async () => {
    const handler = vi.fn();
    render(Toast);
    toast.showPersistent("Update available", { label: "Download", handler });
    await waitFor(() => screen.getByRole("button", { name: "Download" }));
    await fireEvent.click(screen.getByRole("button", { name: "Download" }));
    expect(handler).toHaveBeenCalledOnce();
  });

  it("dismiss removes a persistent toast", async () => {
    render(Toast);
    toast.showPersistent("Persistent");
    await waitFor(() => screen.getByText("Persistent"));
    const dismissBtn = screen.getByRole("button", { name: /dismiss/i });
    await fireEvent.click(dismissBtn);
    await waitFor(() => {
      expect(screen.queryByText("Persistent")).not.toBeInTheDocument();
    });
  });
});
