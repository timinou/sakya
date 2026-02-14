import { render, screen, fireEvent } from "@testing-library/svelte";
import { describe, it, expect, beforeEach, vi } from "vitest";
import Page from "./+page.svelte";
import {
  mockInvokeMultiple,
  resetInvokeMocks,
} from "../tests/tauri-test-utils";
import { projectState } from "$lib/stores";

// Access the mocked dialog open
const dialogOpen = vi.mocked(
  (await import("@tauri-apps/plugin-dialog")).open,
);

function mockProjectCommands() {
  mockInvokeMultiple({
    create_project: (args: Record<string, unknown> | undefined) => ({
      name: args?.name ?? "Test Project",
      version: "0.1.0",
      author: null,
      description: null,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-01T00:00:00Z",
    }),
    open_project: () => ({
      name: "Opened Project",
      version: "0.1.0",
      author: null,
      description: null,
      createdAt: "2026-01-01T00:00:00Z",
      updatedAt: "2026-01-01T00:00:00Z",
    }),
  });
}

describe("+page.svelte - Project Launcher", () => {
  beforeEach(() => {
    resetInvokeMocks();
    dialogOpen.mockReset();
    projectState.close();
  });

  it("renders Sakya title and subtitle when no project open", () => {
    render(Page);
    expect(
      screen.getByRole("heading", { name: /sakya/i }),
    ).toBeInTheDocument();
    expect(screen.getByText("A writing application")).toBeInTheDocument();
  });

  it("shows Create Project and Open Project buttons", () => {
    render(Page);
    expect(
      screen.getByRole("button", { name: /create project/i }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /open project/i }),
    ).toBeInTheDocument();
  });

  it("clicking Create Project shows the create form", async () => {
    render(Page);
    const createBtn = screen.getByRole("button", { name: /create project/i });
    await fireEvent.click(createBtn);

    expect(screen.getByLabelText(/project name/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/location/i)).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /choose folder/i }),
    ).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /^create$/i })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /cancel/i })).toBeInTheDocument();
  });

  it("clicking Cancel hides the create form", async () => {
    render(Page);
    const createBtn = screen.getByRole("button", { name: /create project/i });
    await fireEvent.click(createBtn);

    // Form is visible
    expect(screen.getByLabelText(/project name/i)).toBeInTheDocument();

    const cancelBtn = screen.getByRole("button", { name: /cancel/i });
    await fireEvent.click(cancelBtn);

    // Form is hidden, action buttons are back
    expect(screen.queryByLabelText(/project name/i)).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: /create project/i }),
    ).toBeInTheDocument();
  });

  it("shows error message when projectState has error", async () => {
    projectState.error = "Project path does not exist";
    render(Page);
    expect(
      screen.getByRole("alert"),
    ).toHaveTextContent("Project path does not exist");
  });

  it("shows loading state when isLoading", () => {
    projectState.isLoading = true;
    render(Page);
    expect(screen.getByLabelText(/loading/i)).toBeInTheDocument();
    expect(screen.getByText(/loading project/i)).toBeInTheDocument();
    // Reset
    projectState.isLoading = false;
  });

  it("Create button is disabled when name or path is empty", async () => {
    render(Page);
    const createBtn = screen.getByRole("button", { name: /create project/i });
    await fireEvent.click(createBtn);

    const submitBtn = screen.getByRole("button", { name: /^create$/i });
    expect(submitBtn).toBeDisabled();
  });

  it("calls dialog open when Choose Folder is clicked", async () => {
    dialogOpen.mockResolvedValue("/mock/folder/path");

    render(Page);
    const createBtn = screen.getByRole("button", { name: /create project/i });
    await fireEvent.click(createBtn);

    const chooseBtn = screen.getByRole("button", { name: /choose folder/i });
    await fireEvent.click(chooseBtn);

    expect(dialogOpen).toHaveBeenCalledWith({
      directory: true,
      title: "Choose Project Folder",
    });
  });

  it("calls dialog open when Open Project is clicked", async () => {
    dialogOpen.mockResolvedValue("/mock/existing/project");
    mockProjectCommands();

    render(Page);
    const openBtn = screen.getByRole("button", { name: /open project/i });
    await fireEvent.click(openBtn);

    expect(dialogOpen).toHaveBeenCalledWith({
      directory: true,
      title: "Open Project",
    });
  });
});
