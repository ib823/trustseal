import { describe, it, expect, beforeEach } from "vitest";
import { useUserStore } from "@/lib/stores/user-store";

describe("UserStore", () => {
  beforeEach(() => {
    // Reset store state before each test
    useUserStore.setState({
      user: null,
      isAuthenticated: false,
    });
  });

  it("should have initial state", () => {
    const state = useUserStore.getState();
    expect(state.user).toBeNull();
    expect(state.isAuthenticated).toBe(false);
  });

  it("should set user and mark as authenticated", () => {
    const user = {
      id: "user_01HQ123ABC",
      name: "Admin User",
      email: "admin@example.com",
      role: "admin" as const,
    };

    useUserStore.getState().setUser(user);

    const state = useUserStore.getState();
    expect(state.user).toEqual(user);
    expect(state.isAuthenticated).toBe(true);
  });

  it("should clear user on logout", () => {
    const user = {
      id: "user_01HQ123ABC",
      name: "Admin User",
      email: "admin@example.com",
      role: "admin" as const,
    };

    useUserStore.getState().setUser(user);
    expect(useUserStore.getState().isAuthenticated).toBe(true);

    useUserStore.getState().logout();

    const state = useUserStore.getState();
    expect(state.user).toBeNull();
    expect(state.isAuthenticated).toBe(false);
  });

  it("should handle null user", () => {
    useUserStore.getState().setUser(null);

    const state = useUserStore.getState();
    expect(state.user).toBeNull();
    expect(state.isAuthenticated).toBe(false);
  });
});
