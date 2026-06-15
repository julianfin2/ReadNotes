import type { Tag } from "./tag";

export type Note = {
  id: string;
  content: string;
  createdAt: string;
  updatedAt: string;
  tags: Tag[];
};

export type NoteFilters = {
  search: string;
  tagName: string;
};

export type UpdateNoteInput = {
  id: string;
  content: string;
  tagNames: string[];
};
