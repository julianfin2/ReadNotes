import type { Excerpt } from "./excerpt";
import type { Note } from "./note";

export type TopicStatus =
  | "collecting"
  | "organizing"
  | "drafting"
  | "finished"
  | "paused";

export type Topic = {
  id: string;
  title: string;
  description?: string | null;
  researchQuestion?: string | null;
  status: TopicStatus;
  createdAt: string;
  updatedAt: string;
};

export type TopicNode = {
  id: string;
  topicId: string;
  parentId?: string | null;
  title: string;
  summary?: string | null;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
};

export type TopicMaterial = {
  id: string;
  topicId: string;
  materialType: "excerpt" | "note";
  materialId: string;
  excerptId?: string | null;
  noteId?: string | null;
  nodeId?: string | null;
  reason?: string | null;
  topicReflection?: string | null;
  sortOrder: number;
  addedAt: string;
  updatedAt: string;
  excerpt?: Excerpt | null;
  note?: Note | null;
};

