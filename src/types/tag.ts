export type Tag = {
  id: string;
  name: string;
  parentId?: string | null;
  color?: string | null;
  createdAt: string;
};

export type TagWithCount = Tag & {
  excerptCount: number;
};
