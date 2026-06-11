import type { Tag } from "./tag";

export type Excerpt = {
  id: string;
  quote: string;
  reflection?: string | null;
  sourceWorkId?: string | null;
  location?: string | null;
  importance: number;
  status: "inbox" | "processed" | "archived";
  createdAt: string;
  updatedAt: string;
  tags: Tag[];
};
