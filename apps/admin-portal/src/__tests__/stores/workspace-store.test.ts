import { describe, it, expect, beforeEach } from "vitest";
import { useWorkspaceStore } from "@/lib/stores/workspace-store";

describe("WorkspaceStore", () => {
  beforeEach(() => {
    // Reset store state before each test
    useWorkspaceStore.setState({
      currentWorkspace: null,
      workspaces: [],
      isLoading: false,
    });
  });

  it("should have initial state", () => {
    const state = useWorkspaceStore.getState();
    expect(state.currentWorkspace).toBeNull();
    expect(state.workspaces).toEqual([]);
    expect(state.isLoading).toBe(false);
  });

  it("should set current workspace", () => {
    const workspace = {
      id: "ws_01HQ123ABC",
      name: "Test Property",
      code: "TP",
      address: "123 Test Street",
      totalUnits: 100,
      createdAt: "2024-01-01T00:00:00Z",
    };

    useWorkspaceStore.getState().setCurrentWorkspace(workspace);

    const state = useWorkspaceStore.getState();
    expect(state.currentWorkspace).toEqual(workspace);
  });

  it("should set workspaces list", () => {
    const workspaces = [
      {
        id: "ws_01HQ123ABC",
        name: "Property 1",
        code: "P1",
        address: "123 Street",
        totalUnits: 100,
        createdAt: "2024-01-01T00:00:00Z",
      },
      {
        id: "ws_01HQ456DEF",
        name: "Property 2",
        code: "P2",
        address: "456 Avenue",
        totalUnits: 200,
        createdAt: "2024-02-01T00:00:00Z",
      },
    ];

    useWorkspaceStore.getState().setWorkspaces(workspaces);

    const state = useWorkspaceStore.getState();
    expect(state.workspaces).toHaveLength(2);
    expect(state.workspaces[0].name).toBe("Property 1");
  });

  it("should set loading state", () => {
    useWorkspaceStore.getState().setLoading(true);
    expect(useWorkspaceStore.getState().isLoading).toBe(true);

    useWorkspaceStore.getState().setLoading(false);
    expect(useWorkspaceStore.getState().isLoading).toBe(false);
  });
});
