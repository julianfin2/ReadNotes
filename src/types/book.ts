export type BookChapter = {
  id: string;
  bookId: string;
  title: string;
  sortOrder: number;
  createdAt: string;
  updatedAt: string;
  excerptCount: number;
};

export type Book = {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  chapterCount: number;
  excerptCount: number;
  chapters: BookChapter[];
};
