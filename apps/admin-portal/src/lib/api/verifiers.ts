import { apiClient } from "./client";

export interface Verifier {
  id: string;
  name: string;
  location: string;
  model: string;
  firmwareVersion: string;
  status: "online" | "offline" | "degraded";
  signalStrength: "strong" | "medium" | "weak";
  lastSeen: string;
  todayEvents: number;
  todayDenied: number;
  uptime: string;
  config: VerifierConfig;
}

export interface VerifierConfig {
  bleEnabled: boolean;
  nfcEnabled: boolean;
  offlineMode: boolean;
  statusListTtl: number;
  statusListStaleThreshold: number;
}

export interface CreateVerifierInput {
  name: string;
  location: string;
  model: string;
}

export interface UpdateVerifierInput {
  name?: string;
  location?: string;
  config?: Partial<VerifierConfig>;
}

export async function getVerifiers(workspaceId: string) {
  return apiClient.get<Verifier[]>(`/api/v1/workspaces/${workspaceId}/verifiers`);
}

export async function getVerifier(workspaceId: string, verifierId: string) {
  return apiClient.get<Verifier>(
    `/api/v1/workspaces/${workspaceId}/verifiers/${verifierId}`
  );
}

export async function createVerifier(
  workspaceId: string,
  input: CreateVerifierInput
) {
  return apiClient.post<Verifier>(
    `/api/v1/workspaces/${workspaceId}/verifiers`,
    input
  );
}

export async function updateVerifier(
  workspaceId: string,
  verifierId: string,
  input: UpdateVerifierInput
) {
  return apiClient.patch<Verifier>(
    `/api/v1/workspaces/${workspaceId}/verifiers/${verifierId}`,
    input
  );
}

export async function deleteVerifier(workspaceId: string, verifierId: string) {
  return apiClient.delete(`/api/v1/workspaces/${workspaceId}/verifiers/${verifierId}`);
}

export async function restartVerifier(workspaceId: string, verifierId: string) {
  return apiClient.post(
    `/api/v1/workspaces/${workspaceId}/verifiers/${verifierId}/restart`,
    {}
  );
}

export async function updateVerifierFirmware(
  workspaceId: string,
  verifierId: string
) {
  return apiClient.post(
    `/api/v1/workspaces/${workspaceId}/verifiers/${verifierId}/firmware-update`,
    {}
  );
}
