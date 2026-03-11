import { create } from "zustand";
import type { Visitor } from "./visitor-store";

interface OverrideState {
  overrideTarget: Visitor | null;
  setOverrideTarget: (visitor: Visitor | null) => void;
  clearOverrideTarget: () => void;
}

export const useOverrideStore = create<OverrideState>((set) => ({
  overrideTarget: null,

  setOverrideTarget: (visitor) => set({ overrideTarget: visitor }),

  clearOverrideTarget: () => set({ overrideTarget: null }),
}));
