import type { Tag } from "./tag";

export type Excerpt = {
  id: string;
  quote: string;
  reflection?: string | null;
  bookId?: string | null;
  chapterId?: string | null;
  bookTitle?: string | null;
  chapterTitle?: string | null;
  createdAt: string;
  updatedAt: string;
  tags: Tag[];
};

export type ExcerptFilters = {
  search: string;
  tagName: string;
  sortBy: "createdAt" | "updatedAt";
  sortDirection: "asc" | "desc";
};

export type UpdateExcerptInput = {
  id: string;
  quote: string;
  reflection: string;
  bookId?: string | null;
  chapterId?: string | null;
  bookTitle: string;
  chapterTitle: string;
  tagNames: string[];
};
