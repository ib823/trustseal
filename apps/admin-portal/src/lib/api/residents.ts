import { apiClient } from "./client";

export interface Resident {
  id: string;
  name: string;
  email: string;
  phone: string;
  unitNumber: string;
  credentialStatus: "active" | "suspended" | "pending" | "revoked";
  credentialId?: string;
  lastAccess?: string;
  createdAt: string;
}

export interface CreateResidentInput {
  name: string;
  email: string;
  phone: string;
  unitNumber: string;
}

export interface UpdateResidentInput {
  name?: string;
  email?: string;
  phone?: string;
  unitNumber?: string;
}

export async function getResidents(workspaceId: string) {
  return apiClient.get<Resident[]>(`/api/v1/workspaces/${workspaceId}/residents`);
}

export async function getResident(workspaceId: string, residentId: string) {
  return apiClient.get<Resident>(
    `/api/v1/workspaces/${workspaceId}/residents/${residentId}`
  );
}

export async function createResident(
  workspaceId: string,
  input: CreateResidentInput
) {
  return apiClient.post<Resident>(
    `/api/v1/workspaces/${workspaceId}/residents`,
    input
  );
}

export async function updateResident(
  workspaceId: string,
  residentId: string,
  input: UpdateResidentInput
) {
  return apiClient.patch<Resident>(
    `/api/v1/workspaces/${workspaceId}/residents/${residentId}`,
    input
  );
}

export async function deleteResident(workspaceId: string, residentId: string) {
  return apiClient.delete(`/api/v1/workspaces/${workspaceId}/residents/${residentId}`);
}

export async function issueCredential(workspaceId: string, residentId: string) {
  return apiClient.post<{ credentialId: string }>(
    `/api/v1/workspaces/${workspaceId}/residents/${residentId}/credential`,
    {}
  );
}

export async function revokeCredential(
  workspaceId: string,
  residentId: string,
  reason: string
) {
  return apiClient.delete(
    `/api/v1/workspaces/${workspaceId}/residents/${residentId}/credential?reason=${encodeURIComponent(reason)}`
  );
}

export async function suspendCredential(
  workspaceId: string,
  residentId: string,
  reason: string
) {
  return apiClient.post(
    `/api/v1/workspaces/${workspaceId}/residents/${residentId}/credential/suspend`,
    { reason }
  );
}

export async function reinstateCredential(
  workspaceId: string,
  residentId: string
) {
  return apiClient.post(
    `/api/v1/workspaces/${workspaceId}/residents/${residentId}/credential/reinstate`,
    {}
  );
}
