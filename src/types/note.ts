export type NoteTargetType = "excerpt" | "topic" | "topicNode" | "topicExcerpt";

export type Note = {
  id: string;
  targetType: NoteTargetType;
  targetId: string;
  content: string;
  createdAt: string;
  updatedAt: string;
};

export type TimelineEntry = {
  id: string;
  kind: "excerptCreated" | "topicExcerptAdded" | "noteCreated";
  occurredAt: string;
  title: string;
  content?: string | null;
  targetType?: NoteTargetType | null;
  targetId?: string | null;
  relatedExcerptId?: string | null;
  relatedTopicId?: string | null;
};
