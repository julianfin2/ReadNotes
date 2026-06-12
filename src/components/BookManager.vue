<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import BaseModal from "./BaseModal.vue";
import type { Book, BookChapter } from "../types/book";

const emit = defineEmits<{
  booksChanged: [];
}>();

const books = ref<Book[]>([]);
const selectedBookId = ref("");
const createBookModalOpen = ref(false);
const editBookModalOpen = ref(false);
const deleteBookModalOpen = ref(false);
const createChapterModalOpen = ref(false);
const editChapterModalOpen = ref(false);
const deleteChapterModalOpen = ref(false);
const newBookTitle = ref("");
const editingBookTitle = ref("");
const newChapterTitle = ref("");
const editingChapterTitle = ref("");
const editingChapterId = ref("");
const deletingChapterId = ref("");
const errorMessage = ref("");
const isSaving = ref(false);

const selectedBook = computed(
  () => books.value.find((book) => book.id === selectedBookId.value) || null,
);

const editingChapter = computed(
  () => selectedBook.value?.chapters.find((chapter) => chapter.id === editingChapterId.value) || null,
);

watch(
  books,
  (items) => {
    if (items.length === 0) {
      selectedBookId.value = "";
      return;
    }

    if (!items.some((book) => book.id === selectedBookId.value)) {
      selectedBookId.value = items[0].id;
    }
  },
  { immediate: true },
);

onMounted(async () => {
  await loadBooks();
});

async function loadBooks() {
  books.value = await invoke<Book[]>("list_books");
  emit("booksChanged");
}

async function createBook() {
  await runSaving(async () => {
    await invoke("create_book", {
      input: { title: newBookTitle.value },
    });
    newBookTitle.value = "";
    createBookModalOpen.value = false;
    await loadBooks();
  });
}

function startEditingBook() {
  if (!selectedBook.value) {
    return;
  }

  editingBookTitle.value = selectedBook.value.title;
  editBookModalOpen.value = true;
}

async function updateBook() {
  if (!selectedBook.value) {
    return;
  }

  await runSaving(async () => {
    await invoke("update_book", {
      input: {
        id: selectedBook.value?.id,
        title: editingBookTitle.value,
      },
    });
    editBookModalOpen.value = false;
    await loadBooks();
  });
}

async function deleteBook() {
  if (!selectedBook.value) {
    return;
  }

  await runSaving(async () => {
    await invoke("delete_book", { id: selectedBook.value?.id });
    deleteBookModalOpen.value = false;
    await loadBooks();
  });
}

async function createChapter() {
  if (!selectedBook.value) {
    return;
  }

  await runSaving(async () => {
    await invoke("create_book_chapter", {
      input: {
        bookId: selectedBook.value?.id,
        title: newChapterTitle.value,
      },
    });
    newChapterTitle.value = "";
    createChapterModalOpen.value = false;
    await loadBooks();
  });
}

function startEditingChapter(chapter: BookChapter) {
  editingChapterId.value = chapter.id;
  editingChapterTitle.value = chapter.title;
  editChapterModalOpen.value = true;
}

async function updateChapter() {
  if (!editingChapterId.value) {
    return;
  }

  await runSaving(async () => {
    await invoke("update_book_chapter", {
      input: {
        id: editingChapterId.value,
        title: editingChapterTitle.value,
      },
    });
    editingChapterId.value = "";
    editingChapterTitle.value = "";
    editChapterModalOpen.value = false;
    await loadBooks();
  });
}

function requestDeleteChapter(chapterId: string) {
  deletingChapterId.value = chapterId;
  deleteChapterModalOpen.value = true;
}

async function deleteChapter() {
  if (!deletingChapterId.value) {
    return;
  }

  await runSaving(async () => {
    await invoke("delete_book_chapter", { id: deletingChapterId.value });
    deletingChapterId.value = "";
    deleteChapterModalOpen.value = false;
    await loadBooks();
  });
}

async function runSaving(task: () => Promise<void>) {
  errorMessage.value = "";
  isSaving.value = true;

  try {
    await task();
  } catch (error) {
    errorMessage.value = String(error);
  } finally {
    isSaving.value = false;
  }
}
</script>

<template>
  <section class="book-manager">
    <aside class="book-list-pane">
      <div class="card-header">
        <div>
          <h3>书籍</h3>
          <p class="subtle-text">{{ books.length }} 本书</p>
        </div>
        <button class="primary-action" type="button" @click="createBookModalOpen = true">
          新建
        </button>
      </div>

      <div class="book-list-scroll">
        <button
          v-for="book in books"
          :key="book.id"
          class="book-list-item"
          :class="{ active: book.id === selectedBookId }"
          type="button"
          @click="selectedBookId = book.id"
        >
          <span class="item-title">{{ book.title }}</span>
          <span class="item-meta">
            {{ book.chapterCount }} 章 / {{ book.excerptCount }} 条摘抄
          </span>
        </button>

        <p v-if="books.length === 0" class="empty-state">还没有记录书籍。</p>
      </div>
    </aside>

    <section v-if="selectedBook" class="book-detail-pane">
      <header class="book-detail-header">
        <div>
          <h3>{{ selectedBook.title }}</h3>
          <p class="subtle-text">
            {{ selectedBook.chapterCount }} 个章节 / {{ selectedBook.excerptCount }} 条摘抄
          </p>
        </div>
        <div class="action-row">
          <button class="secondary-action" type="button" @click="startEditingBook">
            编辑书名
          </button>
          <button class="danger-action" type="button" @click="deleteBookModalOpen = true">
            删除书籍
          </button>
        </div>
      </header>

      <section class="chapter-section">
        <div class="card-header">
          <div>
            <h3>章节</h3>
            <p class="subtle-text">用于摘抄录入时快速选择</p>
          </div>
          <button class="primary-action" type="button" @click="createChapterModalOpen = true">
            新建章节
          </button>
        </div>

        <div class="chapter-list">
          <div v-for="chapter in selectedBook.chapters" :key="chapter.id" class="chapter-row">
            <div>
              <strong>{{ chapter.title }}</strong>
              <p class="subtle-text">{{ chapter.excerptCount }} 条摘抄</p>
            </div>
            <div class="action-row">
              <button class="secondary-action" type="button" @click="startEditingChapter(chapter)">
                编辑
              </button>
              <button
                class="danger-action"
                type="button"
                @click="requestDeleteChapter(chapter.id)"
              >
                删除
              </button>
            </div>
          </div>

          <p v-if="selectedBook.chapters.length === 0" class="empty-state">
            这本书还没有章节。
          </p>
        </div>
      </section>
    </section>

    <section v-else class="book-detail-pane empty-detail">
      <p class="empty-state">选择一本书管理章节。</p>
    </section>

    <p v-if="errorMessage" class="error-message">{{ errorMessage }}</p>
  </section>

  <BaseModal :open="createBookModalOpen" title="新建书籍" @close="createBookModalOpen = false">
    <form class="modal-form" @submit.prevent="createBook">
      <label>
        书名
        <input v-model="newBookTitle" placeholder="例如：置身事内" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="createBookModalOpen = false">
          取消
        </button>
        <button class="primary-action" :disabled="isSaving" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="editBookModalOpen" title="编辑书籍" @close="editBookModalOpen = false">
    <form class="modal-form" @submit.prevent="updateBook">
      <label>
        书名
        <input v-model="editingBookTitle" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="editBookModalOpen = false">
          取消
        </button>
        <button class="primary-action" :disabled="isSaving" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal :open="deleteBookModalOpen" title="删除书籍" @close="deleteBookModalOpen = false">
    <div class="modal-form">
      <p class="reflection">这只会删除候选书籍和章节，不会修改已经录入的摘抄文本。</p>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="deleteBookModalOpen = false">
          取消
        </button>
        <button class="danger-action" type="button" @click="deleteBook">删除</button>
      </div>
    </div>
  </BaseModal>

  <BaseModal
    :open="createChapterModalOpen"
    title="新建章节"
    @close="createChapterModalOpen = false"
  >
    <form class="modal-form" @submit.prevent="createChapter">
      <label>
        章节名
        <input v-model="newChapterTitle" placeholder="例如：地方政府的权力与事务" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="createChapterModalOpen = false">
          取消
        </button>
        <button class="primary-action" :disabled="isSaving" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal
    :open="editChapterModalOpen"
    title="编辑章节"
    @close="editChapterModalOpen = false"
  >
    <form v-if="editingChapter" class="modal-form" @submit.prevent="updateChapter">
      <label>
        章节名
        <input v-model="editingChapterTitle" />
      </label>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="editChapterModalOpen = false">
          取消
        </button>
        <button class="primary-action" :disabled="isSaving" type="submit">保存</button>
      </div>
    </form>
  </BaseModal>

  <BaseModal
    :open="deleteChapterModalOpen"
    title="删除章节"
    @close="deleteChapterModalOpen = false"
  >
    <div class="modal-form">
      <p class="reflection">这只会删除候选章节，不会修改已经录入的摘抄文本。</p>
      <div class="modal-actions">
        <button class="secondary-action" type="button" @click="deleteChapterModalOpen = false">
          取消
        </button>
        <button class="danger-action" type="button" @click="deleteChapter">删除</button>
      </div>
    </div>
  </BaseModal>
</template>
