import type { Excerpt } from "./excerpt";

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

export type TopicExcerpt = {
  id: string;
  topicId: string;
  excerptId: string;
  nodeId?: string | null;
  reason?: string | null;
  topicReflection?: string | null;
  sortOrder: number;
  addedAt: string;
  updatedAt: string;
  excerpt: Excerpt;
};
