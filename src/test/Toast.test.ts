import { describe, it, expect, beforeEach } from "vitest";
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
});
