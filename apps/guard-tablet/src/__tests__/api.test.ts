import { describe, it, expect, vi, beforeEach } from "vitest";

describe("API Client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("exports apiClient instance", async () => {
    const { apiClient } = await import("@/lib/api/client");
    expect(apiClient).toBeDefined();
    expect(typeof apiClient.get).toBe("function");
    expect(typeof apiClient.post).toBe("function");
    expect(typeof apiClient.put).toBe("function");
    expect(typeof apiClient.delete).toBe("function");
  });

  it("apiClient can set token", async () => {
    const { apiClient } = await import("@/lib/api/client");
    expect(() => apiClient.setToken("test-token")).not.toThrow();
  });

  it("apiClient can clear token", async () => {
    const { apiClient } = await import("@/lib/api/client");
    expect(() => apiClient.clearToken()).not.toThrow();
  });
});

describe("Visitors API", () => {
  it("exports visitorsApi with list method", async () => {
    const { visitorsApi } = await import("@/lib/api/visitors");
    expect(visitorsApi).toBeDefined();
    expect(typeof visitorsApi.list).toBe("function");
  });

  it("exports visitorsApi with updateStatus method", async () => {
    const { visitorsApi } = await import("@/lib/api/visitors");
    expect(typeof visitorsApi.updateStatus).toBe("function");
  });

  it("list returns mock visitors", async () => {
    const { visitorsApi } = await import("@/lib/api/visitors");
    const visitors = await visitorsApi.list();

    expect(Array.isArray(visitors)).toBe(true);
    expect(visitors.length).toBeGreaterThan(0);

    // Check visitor structure
    const visitor = visitors[0];
    expect(visitor).toHaveProperty("id");
    expect(visitor).toHaveProperty("name");
    expect(visitor).toHaveProperty("idHash");
    expect(visitor).toHaveProperty("purpose");
    expect(visitor).toHaveProperty("hostResident");
    expect(visitor).toHaveProperty("hostUnit");
    expect(visitor).toHaveProperty("credentialType");
    expect(visitor).toHaveProperty("status");
    expect(visitor).toHaveProperty("arrivedAt");
  });

  it("visitor has valid credentialType", async () => {
    const { visitorsApi } = await import("@/lib/api/visitors");
    const visitors = await visitorsApi.list();

    for (const visitor of visitors) {
      expect(["VisitorPass", "ContractorBadge", "EmergencyAccess"]).toContain(
        visitor.credentialType
      );
    }
  });

  it("visitor has valid status", async () => {
    const { visitorsApi } = await import("@/lib/api/visitors");
    const visitors = await visitorsApi.list();

    for (const visitor of visitors) {
      expect(["pending", "verified", "denied"]).toContain(visitor.status);
    }
  });
});

describe("API Functions", () => {
  it("exports getVisitors function", async () => {
    const { getVisitors } = await import("@/lib/api/visitors");
    expect(typeof getVisitors).toBe("function");
  });

  it("exports getVisitor function", async () => {
    const { getVisitor } = await import("@/lib/api/visitors");
    expect(typeof getVisitor).toBe("function");
  });

  it("exports submitEntryDecision function", async () => {
    const { submitEntryDecision } = await import("@/lib/api/visitors");
    expect(typeof submitEntryDecision).toBe("function");
  });

  it("exports submitOverride function", async () => {
    const { submitOverride } = await import("@/lib/api/visitors");
    expect(typeof submitOverride).toBe("function");
  });

  it("exports getOverrideReasonCodes function", async () => {
    const { getOverrideReasonCodes } = await import("@/lib/api/visitors");
    expect(typeof getOverrideReasonCodes).toBe("function");
  });

  it("exports getDailyStats function", async () => {
    const { getDailyStats } = await import("@/lib/api/visitors");
    expect(typeof getDailyStats).toBe("function");
  });
});
