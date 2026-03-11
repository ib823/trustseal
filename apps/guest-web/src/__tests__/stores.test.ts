import { describe, it, expect, beforeEach, vi } from "vitest";

describe("RegistrationStore", () => {
  beforeEach(() => {
    vi.resetModules();
  });

  it("initializes with default state", async () => {
    const { useRegistrationStore } = await import("@/lib/stores/registration-store");
    const state = useRegistrationStore.getState();

    expect(state.currentStep).toBe(1);
    expect(state.invite).toBeNull();
    expect(state.registration.fullName).toBe("");
    expect(state.registration.idType).toBeNull();
    expect(state.registration.termsAgreed).toBe(false);
    expect(state.credential).toBeNull();
    expect(state.isSubmitting).toBe(false);
    expect(state.error).toBeNull();
  });

  it("sets step", async () => {
    const { useRegistrationStore } = await import("@/lib/stores/registration-store");

    useRegistrationStore.getState().setStep(2);
    expect(useRegistrationStore.getState().currentStep).toBe(2);

    useRegistrationStore.getState().setStep(3);
    expect(useRegistrationStore.getState().currentStep).toBe(3);
  });

  it("navigates steps with nextStep and prevStep", async () => {
    const { useRegistrationStore } = await import("@/lib/stores/registration-store");

    useRegistrationStore.getState().setStep(1);
    useRegistrationStore.getState().nextStep();
    expect(useRegistrationStore.getState().currentStep).toBe(2);

    useRegistrationStore.getState().nextStep();
    expect(useRegistrationStore.getState().currentStep).toBe(3);

    useRegistrationStore.getState().prevStep();
    expect(useRegistrationStore.getState().currentStep).toBe(2);
  });

  it("does not go below step 1", async () => {
    const { useRegistrationStore } = await import("@/lib/stores/registration-store");

    useRegistrationStore.getState().setStep(1);
    useRegistrationStore.getState().prevStep();
    expect(useRegistrationStore.getState().currentStep).toBe(1);
  });

  it("updates registration data", async () => {
    const { useRegistrationStore } = await import("@/lib/stores/registration-store");

    useRegistrationStore.getState().updateRegistration({
      fullName: "John Doe",
      idType: "ic",
    });

    const state = useRegistrationStore.getState();
    expect(state.registration.fullName).toBe("John Doe");
    expect(state.registration.idType).toBe("ic");
  });

  it("sets invite data", async () => {
    const { useRegistrationStore } = await import("@/lib/stores/registration-store");

    const invite = {
      inviteId: "inv_123",
      propertyName: "Test Property",
      propertyId: "prop_123",
      hostName: "Test Host",
      hostUnit: "A-01-01",
      visitDate: "2026-03-15T10:00:00Z",
      expiresAt: "2026-03-17T10:00:00Z",
    };

    useRegistrationStore.getState().setInvite(invite);
    expect(useRegistrationStore.getState().invite).toEqual(invite);
  });

  it("sets credential data", async () => {
    const { useRegistrationStore } = await import("@/lib/stores/registration-store");

    const credential = {
      credentialId: "cred_123",
      qrCodeData: "vaultpass://verify/cred_123",
      deepLink: "vaultpass://add-pass/cred_123",
      validFrom: "2026-03-15T10:00:00Z",
      validUntil: "2026-03-16T10:00:00Z",
    };

    useRegistrationStore.getState().setCredential(credential);
    expect(useRegistrationStore.getState().credential).toEqual(credential);
  });

  it("manages submitting state", async () => {
    const { useRegistrationStore } = await import("@/lib/stores/registration-store");

    expect(useRegistrationStore.getState().isSubmitting).toBe(false);

    useRegistrationStore.getState().setSubmitting(true);
    expect(useRegistrationStore.getState().isSubmitting).toBe(true);

    useRegistrationStore.getState().setSubmitting(false);
    expect(useRegistrationStore.getState().isSubmitting).toBe(false);
  });

  it("manages error state", async () => {
    const { useRegistrationStore } = await import("@/lib/stores/registration-store");

    expect(useRegistrationStore.getState().error).toBeNull();

    useRegistrationStore.getState().setError("Test error");
    expect(useRegistrationStore.getState().error).toBe("Test error");

    useRegistrationStore.getState().setError(null);
    expect(useRegistrationStore.getState().error).toBeNull();
  });

  it("resets to initial state", async () => {
    const { useRegistrationStore } = await import("@/lib/stores/registration-store");

    // Modify state
    useRegistrationStore.getState().setStep(3);
    useRegistrationStore.getState().updateRegistration({ fullName: "Test" });
    useRegistrationStore.getState().setError("Test error");

    // Reset
    useRegistrationStore.getState().reset();

    const state = useRegistrationStore.getState();
    expect(state.currentStep).toBe(1);
    expect(state.registration.fullName).toBe("");
    expect(state.error).toBeNull();
  });
});
