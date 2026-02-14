import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, beforeEach } from "vitest";
import Page from "./+page.svelte";
import {
  mockInvokeWith,
  resetInvokeMocks,
  mockedInvoke,
} from "../tests/tauri-test-utils";

describe("+page.svelte", () => {
  beforeEach(() => {
    resetInvokeMocks();
  });

  it("renders the welcome heading", () => {
    render(Page);
    expect(
      screen.getByRole("heading", { name: /welcome to tauri/i }),
    ).toBeInTheDocument();
  });

  it("renders all three logos", () => {
    render(Page);
    expect(screen.getByAltText("Vite Logo")).toBeInTheDocument();
    expect(screen.getByAltText("Tauri Logo")).toBeInTheDocument();
    expect(screen.getByAltText("SvelteKit Logo")).toBeInTheDocument();
  });

  it("renders the greet form with input and button", () => {
    render(Page);
    expect(
      screen.getByPlaceholderText("Enter a name..."),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /greet/i }),
    ).toBeInTheDocument();
  });

  it("calls greet command on form submit and displays response", async () => {
    mockInvokeWith("greet", (args) => {
      return `Hello, ${args?.name}! You've been greeted from Rust!`;
    });

    render(Page);

    const input = screen.getByPlaceholderText("Enter a name...");
    const button = screen.getByRole("button", { name: /greet/i });

    await fireEvent.input(input, { target: { value: "World" } });
    await fireEvent.click(button);

    expect(mockedInvoke).toHaveBeenCalledWith("greet", { name: "World" });
    expect(
      await screen.findByText(
        "Hello, World! You've been greeted from Rust!",
      ),
    ).toBeInTheDocument();
  });
});
