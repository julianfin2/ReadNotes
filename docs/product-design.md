# ReadNotes Product Design

## Product Positioning

ReadNotes is a personal excerpt and topic research manager.

It is not a general note-taking app, a document editor, or a reading source adapter. Excerpts are entered manually by the user, so the product should focus on organizing, retrieving, reusing, and developing those excerpts into topic-based research or writing.

Core principle:

```text
Excerpts are materials.
Tags are retrieval tools.
Topics are research or writing projects.
Subtopics are structure inside a topic.
Topic excerpt links explain why a material matters in a specific project.
```

## Core Concepts

### Excerpt

An excerpt is a reusable piece of source text plus the user's initial understanding.

An excerpt is global. It does not belong to one topic. It can be linked to multiple topics and can mean different things in each topic.

Primary responsibilities:

- Store the quoted text.
- Store the user's initial reflection.
- Support global tags.
- Support optional book title and chapter title.
- Preserve created and updated timestamps for sorting and future review.

### Tag

A tag is a stable, reusable classification label for retrieval.

Tags answer questions like:

- Which excerpts are related to power?
- Which excerpts are writing materials?
- Which excerpts are about education, anxiety, self-deception, or desire?

Tags are different from topics. Tags are usually broad and long-lived. Topics are usually project-oriented and may eventually become essays, reports, lectures, or research summaries.

### Topic

A topic is an active research or writing project.

Examples:

- Modern sources of anxiety
- How to write character desire
- The concept of order in Confucian ethics

A topic should support:

- A title.
- A description.
- A research question.
- A status, such as collecting, organizing, drafting, finished, or paused.
- Nested subtopics.
- Linked excerpts.
- Topic-level summaries and draft material.

### Topic Node

A topic node is a subtopic inside a topic.

Topic nodes are not global tags. They only make sense inside their parent topic.

Example:

```text
Topic: Modern sources of anxiety
- Time pressure
- Social comparison
  - Social media comparison
  - Peer competition
- Identity uncertainty
- Consumerism
```

### Topic Excerpt

A topic excerpt link is the relationship between an excerpt and a topic.

This is the most important relationship in the model. The same excerpt may be useful in several topics, and its meaning may differ in each one.

The topic excerpt link should store:

- Which excerpt is collected into which topic.
- Which subtopic it belongs to.
- Why the user added it to this topic.
- The user's topic-specific reflection.
- Sorting position inside the topic or subtopic.
- Added and updated timestamps.

The global excerpt reflection and the topic-specific reflection must stay separate.

### Excerpt Provenance

Source work management is not part of the current product scope.

The product does not need to support PDF, web clipping, Kindle import, or source-specific adapters. For excerpt provenance, each excerpt should only store:

- Book title
- Chapter title

## Data Model

### Type Shapes

```ts
type Excerpt = {
  id: string
  quote: string
  reflection?: string
  bookTitle?: string
  chapterTitle?: string
  createdAt: string
  updatedAt: string
}

type Tag = {
  id: string
  name: string
  parentId?: string
  color?: string
  createdAt: string
}

type ExcerptTag = {
  excerptId: string
  tagId: string
}

type Topic = {
  id: string
  title: string
  description?: string
  researchQuestion?: string
  status: "collecting" | "organizing" | "drafting" | "finished" | "paused"
  createdAt: string
  updatedAt: string
}

type TopicNode = {
  id: string
  topicId: string
  parentId?: string
  title: string
  summary?: string
  sortOrder: number
  createdAt: string
  updatedAt: string
}

type TopicExcerpt = {
  id: string
  topicId: string
  excerptId: string
  nodeId?: string
  reason?: string
  topicReflection?: string
  sortOrder: number
  addedAt: string
  updatedAt: string
}

```

### SQLite Schema Draft

```sql
CREATE TABLE excerpts (
  id TEXT PRIMARY KEY,
  quote TEXT NOT NULL,
  reflection TEXT,
  book_title TEXT,
  chapter_title TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE tags (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  parent_id TEXT,
  color TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY (parent_id) REFERENCES tags(id)
);

CREATE TABLE excerpt_tags (
  excerpt_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  PRIMARY KEY (excerpt_id, tag_id),
  FOREIGN KEY (excerpt_id) REFERENCES excerpts(id) ON DELETE CASCADE,
  FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

CREATE TABLE topics (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL,
  description TEXT,
  research_question TEXT,
  status TEXT NOT NULL DEFAULT 'collecting',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE topic_nodes (
  id TEXT PRIMARY KEY,
  topic_id TEXT NOT NULL,
  parent_id TEXT,
  title TEXT NOT NULL,
  summary TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE,
  FOREIGN KEY (parent_id) REFERENCES topic_nodes(id) ON DELETE CASCADE
);

CREATE TABLE topic_excerpts (
  id TEXT PRIMARY KEY,
  topic_id TEXT NOT NULL,
  excerpt_id TEXT NOT NULL,
  node_id TEXT,
  reason TEXT,
  topic_reflection TEXT,
  sort_order INTEGER NOT NULL DEFAULT 0,
  added_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (topic_id) REFERENCES topics(id) ON DELETE CASCADE,
  FOREIGN KEY (excerpt_id) REFERENCES excerpts(id) ON DELETE CASCADE,
  FOREIGN KEY (node_id) REFERENCES topic_nodes(id) ON DELETE SET NULL
);

```

## First-Version Product Scope

### Must Have

- Create, edit, and delete excerpts.
- Add quote text, initial reflection, optional book title, and optional chapter title.
- Create and assign tags.
- Search excerpts by text, tags, and time.
- Create topics.
- Create nested topic nodes.
- Add excerpts to topics.
- Assign topic excerpts to topic nodes.
- Add topic-specific reasons and reflections.

### Should Have

- Reorder topic nodes.
- Reorder excerpts inside a topic node.
- Filter topic excerpts by tag, book title, and chapter title.
- Keep all user data local by default.

### Not In First Version

- PDF import.
- Web clipping.
- Kindle import.
- Multi-device sync.
- AI summarization.
- Rich text editing.
- Sharing or publishing.
- User accounts.
- Markdown export.

## Main Screens

### Quick Capture

Purpose: record an excerpt with minimal friction.

Fields:

- Quote
- Initial reflection
- Tags
- Optional book title
- Optional chapter title
- Optional related topics

Expected interactions:

- Save quickly with keyboard.
- Create tags inline.
- Link to one or more topics while entering.

### Excerpt Library

Purpose: browse and search all excerpt materials.

Capabilities:

- Full-text search.
- Filter by tag, book title, chapter title, and date.
- Sort by created time and updated time.
- Open excerpt detail.

### Topic Workspace

Purpose: collect, structure, interpret, and prepare writing around a topic.

Suggested layout:

```text
Left: topic tree and nested subtopics
Center: excerpts collected under the selected topic or subtopic
Right: selected excerpt details, reason, and topic-specific reflection
Bottom or side panel: topic summary or draft material
```

### Tag Browser

Purpose: retrieve all excerpts under a tag or tag family.

Capabilities:

- Show tag hierarchy.
- Show matching excerpts.
- Support selecting multiple tags.

## Implementation Notes

- Prefer local-first storage.
- SQLite is the recommended first storage layer.
- Use SQLite FTS for excerpt search.
- Keep the model source-agnostic. Manual entry is the primary input path.
- Avoid making tags and topic nodes share the same table unless a clear need emerges. Their product meaning is different.
- Preserve timestamps on link objects, not only on main records, because "when this excerpt entered this topic" matters.

## Current Implementation Status

- SQLite initialization and first schema migration are implemented in the Tauri backend.
- Excerpt CRUD is implemented and exposed through Tauri commands.
- Global tag CRUD is implemented and exposed through Tauri commands.
- Excerpts can be linked to global tags through `excerpt_tags`.
- Quick capture supports inline tag names. Missing tags are created automatically during excerpt creation.
- Topic, topic node, and topic excerpt link CRUD is implemented and exposed through Tauri commands.
- The frontend is split into focused components under `src/components` and shared types under `src/types`.
- A minimal topic workspace exists for creating topics, creating first-level topic nodes, and collecting existing excerpts into topics.
- Excerpt library search and filtering is implemented with SQLite FTS for quote/reflection search, tag filtering, and configurable time sorting.
- Excerpt maintenance is implemented in the frontend: edit quote/reflection/book title/chapter title/tags and delete excerpts.
- Topic workspace maintenance is implemented in the frontend: edit/delete topics, edit/delete nested topic nodes, and edit/remove topic excerpt links.
- Tag management is implemented in the frontend: create/edit/delete tags, assign parent tags, show excerpt counts, and browse excerpts by selected tag.
- Excerpts support lightweight source fields: book title and chapter title.
- Source work workflows are not planned for the current scope.
