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
  });
});

describe("Invites API", () => {
  it("exports getInvite function", async () => {
    const { getInvite } = await import("@/lib/api/invites");
    expect(typeof getInvite).toBe("function");
  });

  it("exports submitRegistration function", async () => {
    const { submitRegistration } = await import("@/lib/api/invites");
    expect(typeof submitRegistration).toBe("function");
  });

  it("getInvite returns valid invite for demo ID", async () => {
    const { getInvite } = await import("@/lib/api/invites");
    const response = await getInvite("demo");

    expect(response.isValid).toBe(true);
    expect(response.isExpired).toBe(false);
    expect(response.invite).toBeDefined();
    expect(response.invite.inviteId).toBe("demo");
    expect(response.invite.propertyName).toBeDefined();
    expect(response.invite.hostName).toBeDefined();
  });

  it("getInvite returns expired for 'expired' ID", async () => {
    const { getInvite } = await import("@/lib/api/invites");
    const response = await getInvite("expired");

    expect(response.isValid).toBe(false);
    expect(response.isExpired).toBe(true);
  });

  it("getInvite rejects invalid ID", async () => {
    const { getInvite } = await import("@/lib/api/invites");

    await expect(getInvite("invalid")).rejects.toThrow();
  });

  it("submitRegistration returns credential", async () => {
    const { submitRegistration } = await import("@/lib/api/invites");
    const response = await submitRegistration({
      inviteId: "demo",
      fullName: "Test User",
      idType: "ic",
      idHash: "sha256:abc123...",
      purpose: "social",
      livenessVerified: false,
    });

    expect(response.success).toBe(true);
    expect(response.credential).toBeDefined();
    expect(response.credential.credentialId).toBeDefined();
    expect(response.credential.qrCodeData).toBeDefined();
    expect(response.credential.deepLink).toBeDefined();
  });
});

describe("invitesApi", () => {
  it("exports invitesApi object with get and register", async () => {
    const { invitesApi } = await import("@/lib/api/invites");
    expect(invitesApi).toBeDefined();
    expect(typeof invitesApi.get).toBe("function");
    expect(typeof invitesApi.register).toBe("function");
  });
});
