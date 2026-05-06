import { writable } from "svelte/store";

export interface ToastMessage {
  id: string;
  message: string;
}

function createToastStore() {
  const { subscribe, update, set } = writable<ToastMessage[]>([]);
  return {
    subscribe,
    show(message: string) {
      const id = crypto.randomUUID();
      update((msgs) => [...msgs, { id, message }]);
      setTimeout(() => update((msgs) => msgs.filter((m) => m.id !== id)), 4000);
    },
    dismiss(id: string) {
      update((msgs) => msgs.filter((m) => m.id !== id));
    },
    clear() {
      set([]);
    },
  };
}

export const toast = createToastStore();
