import type { InviteData, CredentialData } from "@/lib/stores/registration-store";

export interface InviteResponse {
  invite: InviteData;
  isValid: boolean;
  isExpired: boolean;
}

export interface RegistrationRequest {
  inviteId: string;
  fullName: string;
  idType: "ic" | "passport";
  idHash: string;
  purpose: string;
  purposeOther?: string;
  vehicleNumber?: string;
  livenessVerified: boolean;
}

export interface RegistrationResponse {
  success: boolean;
  credential: CredentialData;
}

// Mock data for development
const MOCK_INVITE: InviteData = {
  inviteId: "inv_01HQXYZ1234567890ABCDEF",
  propertyName: "Setia Sky Residences",
  propertyId: "prop_01HQXYZ",
  hostName: "Ahmad bin Abdullah",
  hostUnit: "A-12-03",
  visitDate: new Date(Date.now() + 86400000).toISOString(), // Tomorrow
  expiresAt: new Date(Date.now() + 172800000).toISOString(), // 2 days
};

const MOCK_CREDENTIAL: CredentialData = {
  credentialId: "cred_01HQXYZ1234567890ABCDEF",
  qrCodeData: "vaultpass://verify/cred_01HQXYZ1234567890ABCDEF",
  deepLink: "vaultpass://add-pass/cred_01HQXYZ1234567890ABCDEF",
  validFrom: new Date().toISOString(),
  validUntil: new Date(Date.now() + 86400000).toISOString(),
};

export async function getInvite(inviteId: string): Promise<InviteResponse> {
  // In production, this would call the real API
  // For now, return mock data
  return new Promise((resolve, reject) => {
    setTimeout(() => {
      if (inviteId === "invalid") {
        reject(new Error("Invite not found"));
        return;
      }
      if (inviteId === "expired") {
        resolve({
          invite: { ...MOCK_INVITE, inviteId },
          isValid: false,
          isExpired: true,
        });
        return;
      }
      resolve({
        invite: { ...MOCK_INVITE, inviteId },
        isValid: true,
        isExpired: false,
      });
    }, 500);
  });
}

export async function submitRegistration(
  _data: RegistrationRequest
): Promise<RegistrationResponse> {
  // In production, this would call the real API
  return new Promise((resolve) => {
    setTimeout(() => {
      resolve({
        success: true,
        credential: MOCK_CREDENTIAL,
      });
    }, 1000);
  });
}

export const invitesApi = {
  get: getInvite,
  register: submitRegistration,
};
