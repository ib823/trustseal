import { create } from "zustand";

export type IdType = "ic" | "passport";
export type VisitPurpose = "social" | "delivery" | "service" | "business" | "other";

export interface RegistrationData {
  fullName: string;
  idType: IdType | null;
  idNumber: string;
  purpose: VisitPurpose | null;
  purposeOther: string;
  vehicleNumber: string;
  termsAgreed: boolean;
  livenessVerified: boolean;
}

export interface InviteData {
  inviteId: string;
  propertyName: string;
  propertyId: string;
  hostName: string;
  hostUnit: string;
  visitDate: string;
  expiresAt: string;
}

export interface CredentialData {
  credentialId: string;
  qrCodeData: string;
  deepLink: string;
  validFrom: string;
  validUntil: string;
}

interface RegistrationState {
  currentStep: number;
  invite: InviteData | null;
  registration: RegistrationData;
  credential: CredentialData | null;
  isSubmitting: boolean;
  error: string | null;

  setStep: (step: number) => void;
  nextStep: () => void;
  prevStep: () => void;
  setInvite: (invite: InviteData) => void;
  updateRegistration: (data: Partial<RegistrationData>) => void;
  setCredential: (credential: CredentialData) => void;
  setSubmitting: (submitting: boolean) => void;
  setError: (error: string | null) => void;
  reset: () => void;
}

const initialRegistration: RegistrationData = {
  fullName: "",
  idType: null,
  idNumber: "",
  purpose: null,
  purposeOther: "",
  vehicleNumber: "",
  termsAgreed: false,
  livenessVerified: false,
};

export const useRegistrationStore = create<RegistrationState>((set) => ({
  currentStep: 1,
  invite: null,
  registration: initialRegistration,
  credential: null,
  isSubmitting: false,
  error: null,

  setStep: (step) => set({ currentStep: step }),
  nextStep: () => set((state) => ({ currentStep: state.currentStep + 1 })),
  prevStep: () => set((state) => ({ currentStep: Math.max(1, state.currentStep - 1) })),
  setInvite: (invite) => set({ invite }),
  updateRegistration: (data) =>
    set((state) => ({
      registration: { ...state.registration, ...data },
    })),
  setCredential: (credential) => set({ credential }),
  setSubmitting: (submitting) => set({ isSubmitting: submitting }),
  setError: (error) => set({ error }),
  reset: () =>
    set({
      currentStep: 1,
      registration: initialRegistration,
      credential: null,
      isSubmitting: false,
      error: null,
    }),
}));
