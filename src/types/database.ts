export type DatabaseInfo = {
  currentPath: string;
  defaultPath: string;
  usingDefault: boolean;
  startupIssue: DatabaseStartupIssue | null;
};

export type DatabaseStartupIssue = {
  configuredPath: string;
  reason: string;
};
