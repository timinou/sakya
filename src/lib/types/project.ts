export interface ProjectManifest {
  name: string;
  version: string;
  author?: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
}

export interface RecentProject {
  name: string;
  path: string;
  lastOpened: string;
}
