import { create } from "zustand";
import { visitorsApi } from "@/lib/api/visitors";

export type VisitorStatus = "pending" | "verified" | "denied";

export interface Visitor {
  id: string;
  name: string;
  idHash: string;
  purpose: string;
  hostResident: string;
  hostUnit: string;
  credentialType: "VisitorPass" | "ContractorBadge" | "EmergencyAccess";
  status: VisitorStatus;
  arrivedAt: string;
  expiresAt?: string;
}

interface VisitorState {
  visitors: Visitor[];
  filter: VisitorStatus | "all";
  isLoading: boolean;
  error: string | null;
  lastFetch: number | null;
  setFilter: (filter: VisitorStatus | "all") => void;
  fetchVisitors: () => Promise<void>;
  updateVisitorStatus: (id: string, status: VisitorStatus) => Promise<void>;
}

export const useVisitorStore = create<VisitorState>((set) => ({
  visitors: [],
  filter: "all",
  isLoading: false,
  error: null,
  lastFetch: null,

  setFilter: (filter) => set({ filter }),

  fetchVisitors: async () => {
    set({ isLoading: true, error: null });
    try {
      const data = await visitorsApi.list();
      set({ visitors: data, lastFetch: Date.now(), isLoading: false });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : "Failed to fetch visitors",
        isLoading: false,
      });
    }
  },

  updateVisitorStatus: async (id, status) => {
    try {
      await visitorsApi.updateStatus(id, status);
      set((state) => ({
        visitors: state.visitors.map((v) =>
          v.id === id ? { ...v, status } : v
        ),
      }));
    } catch (err) {
      set({
        error:
          err instanceof Error ? err.message : "Failed to update visitor status",
      });
      throw err;
    }
  },
}));
