import { invoke } from "@tauri-apps/api/core";

export type DraftRecord<T> = {
  entityType: string;
  entityId: string;
  payload: T;
  updatedAt: string;
};

type RawDraftRecord = {
  entityType: string;
  entityId: string;
  payload: string;
  updatedAt: string;
};

export async function getDraftPayload<T>(
  entityType: string,
  entityId: string,
): Promise<DraftRecord<T> | null> {
  const draft = await invoke<RawDraftRecord | null>("get_draft", {
    entityType,
    entityId,
  });

  if (!draft) {
    return null;
  }

  return {
    ...draft,
    payload: JSON.parse(draft.payload) as T,
  };
}

export async function saveDraftPayload<T>(
  entityType: string,
  entityId: string,
  payload: T,
) {
  await invoke("save_draft", {
    input: {
      entityType,
      entityId,
      payload: JSON.stringify(payload),
    },
  });
}

export async function deleteDraftPayload(entityType: string, entityId: string) {
  await invoke("delete_draft", {
    entityType,
    entityId,
  });
}
