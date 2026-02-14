/**
 * Mock data factories for E2E tests.
 * Types here mirror planned Rust types from PROJ-001.
 */

export interface ProjectManifest {
  name: string;
  version: string;
  description?: string;
  author?: string;
}

export function createMockProject(
  overrides: Partial<ProjectManifest> = {},
): ProjectManifest {
  return {
    name: "Test Project",
    version: "0.1.0",
    description: "A test project for E2E testing",
    author: "Test Author",
    ...overrides,
  };
}
