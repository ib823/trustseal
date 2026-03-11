import { describe, it, expect, beforeEach, vi } from "vitest";

// Mock zustand persist
vi.mock("zustand/middleware", async () => {
  const actual = await vi.importActual("zustand/middleware");
  return {
    ...actual,
    persist: (fn: (set: unknown, get: unknown, api: unknown) => unknown) => fn,
  };
});

describe("VisitorStore", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it("initializes with default state", async () => {
    const { useVisitorStore } = await import("@/lib/stores/visitor-store");
    const state = useVisitorStore.getState();

    expect(state.visitors).toEqual([]);
    expect(state.filter).toBe("all");
    expect(state.isLoading).toBe(false);
    expect(state.error).toBeNull();
    expect(state.lastFetch).toBeNull();
  });

  it("sets filter", async () => {
    const { useVisitorStore } = await import("@/lib/stores/visitor-store");

    useVisitorStore.getState().setFilter("pending");

    expect(useVisitorStore.getState().filter).toBe("pending");
  });

  it("has fetchVisitors function", async () => {
    const { useVisitorStore } = await import("@/lib/stores/visitor-store");
    expect(typeof useVisitorStore.getState().fetchVisitors).toBe("function");
  });

  it("has updateVisitorStatus function", async () => {
    const { useVisitorStore } = await import("@/lib/stores/visitor-store");
    expect(typeof useVisitorStore.getState().updateVisitorStatus).toBe(
      "function"
    );
  });
});

describe("OfflineStore", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it("initializes with default state", async () => {
    const { useOfflineStore } = await import("@/lib/stores/offline-store");
    const state = useOfflineStore.getState();

    expect(state.isOnline).toBe(true);
    expect(state.queue).toEqual([]);
  });

  it("adds action to queue", async () => {
    const { useOfflineStore } = await import("@/lib/stores/offline-store");

    useOfflineStore.getState().addToQueue({
      type: "verify",
      visitorId: "vis_123",
      timestamp: 1234567890,
    });

    const state = useOfflineStore.getState();
    expect(state.queue).toHaveLength(1);
    expect(state.queue[0].type).toBe("verify");
    expect(state.queue[0].visitorId).toBe("vis_123");
  });

  it("removes action from queue", async () => {
    const { useOfflineStore } = await import("@/lib/stores/offline-store");

    useOfflineStore.getState().addToQueue({
      type: "verify",
      visitorId: "vis_123",
      timestamp: 1234567890,
    });

    expect(useOfflineStore.getState().queue).toHaveLength(1);

    useOfflineStore.getState().removeFromQueue("vis_123", "verify");

    expect(useOfflineStore.getState().queue).toHaveLength(0);
  });

  it("clears queue", async () => {
    const { useOfflineStore } = await import("@/lib/stores/offline-store");

    useOfflineStore.getState().addToQueue({
      type: "verify",
      visitorId: "vis_123",
      timestamp: 1234567890,
    });
    useOfflineStore.getState().addToQueue({
      type: "deny",
      visitorId: "vis_456",
      timestamp: 1234567891,
    });

    expect(useOfflineStore.getState().queue).toHaveLength(2);

    useOfflineStore.getState().clearQueue();

    expect(useOfflineStore.getState().queue).toHaveLength(0);
  });

  it("sets online status", async () => {
    const { useOfflineStore } = await import("@/lib/stores/offline-store");

    useOfflineStore.getState().setOnline(false);

    expect(useOfflineStore.getState().isOnline).toBe(false);

    useOfflineStore.getState().setOnline(true);

    expect(useOfflineStore.getState().isOnline).toBe(true);
  });
});

describe("OverrideStore", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it("initializes with null override target", async () => {
    const { useOverrideStore } = await import("@/lib/stores/override-store");
    const state = useOverrideStore.getState();

    expect(state.overrideTarget).toBeNull();
  });

  it("sets override target", async () => {
    const { useOverrideStore } = await import("@/lib/stores/override-store");
    const mockVisitor = {
      id: "vis_123",
      name: "Test Visitor",
      idHash: "sha256:abc",
      purpose: "Testing",
      hostResident: "Host Name",
      hostUnit: "A-01-01",
      credentialType: "VisitorPass" as const,
      status: "pending" as const,
      arrivedAt: new Date().toISOString(),
    };

    useOverrideStore.getState().setOverrideTarget(mockVisitor);

    expect(useOverrideStore.getState().overrideTarget).toEqual(mockVisitor);
  });

  it("clears override target", async () => {
    const { useOverrideStore } = await import("@/lib/stores/override-store");
    const mockVisitor = {
      id: "vis_123",
      name: "Test Visitor",
      idHash: "sha256:abc",
      purpose: "Testing",
      hostResident: "Host Name",
      hostUnit: "A-01-01",
      credentialType: "VisitorPass" as const,
      status: "pending" as const,
      arrivedAt: new Date().toISOString(),
    };

    useOverrideStore.getState().setOverrideTarget(mockVisitor);

    expect(useOverrideStore.getState().overrideTarget).not.toBeNull();

    useOverrideStore.getState().clearOverrideTarget();

    expect(useOverrideStore.getState().overrideTarget).toBeNull();
  });
});
