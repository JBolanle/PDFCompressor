import { writable } from "svelte/store";

export const selectedFileId = writable<string | null>(null);
