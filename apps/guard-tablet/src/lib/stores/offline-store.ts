import { create } from "zustand";
import { persist } from "zustand/middleware";

export interface QueuedAction {
  type: "verify" | "deny" | "override";
  visitorId: string;
  timestamp: number;
  reason?: string;
}

interface OfflineState {
  isOnline: boolean;
  queue: QueuedAction[];
  setOnline: (online: boolean) => void;
  addToQueue: (action: QueuedAction) => void;
  removeFromQueue: (visitorId: string, type: QueuedAction["type"]) => void;
  clearQueue: () => void;
  processQueue: () => Promise<void>;
}

export const useOfflineStore = create<OfflineState>()(
  persist(
    (set, get) => ({
      isOnline: typeof window !== "undefined" ? navigator.onLine : true,
      queue: [],

      setOnline: (online) => {
        set({ isOnline: online });
        if (online) {
          get().processQueue();
        }
      },

      addToQueue: (action) =>
        set((state) => ({
          queue: [...state.queue, action],
        })),

      removeFromQueue: (visitorId, type) =>
        set((state) => ({
          queue: state.queue.filter(
            (a) => !(a.visitorId === visitorId && a.type === type)
          ),
        })),

      clearQueue: () => set({ queue: [] }),

      processQueue: async () => {
        const state = get();
        if (!state.isOnline || state.queue.length === 0) return;

        // Process each queued action
        for (const action of state.queue) {
          try {
            // In a real app, this would call the API
            // For now, we just remove from queue on success
            state.removeFromQueue(action.visitorId, action.type);
          } catch {
            // Keep in queue on failure
          }
        }
      },
    }),
    {
      name: "guard-tablet-offline",
    }
  )
);

// Set up online/offline listeners
if (typeof window !== "undefined") {
  window.addEventListener("online", () => {
    useOfflineStore.getState().setOnline(true);
  });
  window.addEventListener("offline", () => {
    useOfflineStore.getState().setOnline(false);
  });
}
