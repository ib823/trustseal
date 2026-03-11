import { apiClient } from "./client";
import type { Visitor, VisitorStatus } from "@/lib/stores/visitor-store";

export interface VisitorListParams {
  status?: VisitorStatus | "all";
  search?: string;
  limit?: number;
  offset?: number;
}

export interface VisitorListResponse {
  visitors: Visitor[];
  total: number;
}

export interface EntryDecision {
  visitorId: string;
  decision: "allow" | "deny";
  reason?: string;
}

export interface OverrideRequest {
  visitorId: string;
  reasonCode: string;
  biometricToken: string;
}

export interface OverrideResponse {
  success: boolean;
  logId: string;
  timestamp: string;
}

export async function getVisitors(
  tenantId: string,
  params?: VisitorListParams
) {
  const queryParams: Record<string, string> = {};

  if (params?.status && params.status !== "all") {
    queryParams.status = params.status;
  }
  if (params?.search) {
    queryParams.search = params.search;
  }
  if (params?.limit) {
    queryParams.limit = params.limit.toString();
  }
  if (params?.offset) {
    queryParams.offset = params.offset.toString();
  }

  return apiClient.get<VisitorListResponse>(
    `/api/v1/${tenantId}/visitors`,
    queryParams
  );
}

export async function getVisitor(tenantId: string, visitorId: string) {
  return apiClient.get<Visitor>(`/api/v1/${tenantId}/visitors/${visitorId}`);
}

export async function submitEntryDecision(
  tenantId: string,
  decision: EntryDecision
) {
  return apiClient.post<{ success: boolean; logId: string }>(
    `/api/v1/${tenantId}/entry-decisions`,
    decision
  );
}

export async function submitOverride(
  tenantId: string,
  override: OverrideRequest
) {
  return apiClient.post<OverrideResponse>(
    `/api/v1/${tenantId}/overrides`,
    override
  );
}

export async function getOverrideReasonCodes(tenantId: string) {
  return apiClient.get<{ codes: { code: string; label: string }[] }>(
    `/api/v1/${tenantId}/override-codes`
  );
}

export async function getDailyStats(tenantId: string) {
  return apiClient.get<{
    totalEntries: number;
    verified: number;
    denied: number;
    pending: number;
  }>(`/api/v1/${tenantId}/stats/today`);
}

// Mock data for development
const MOCK_VISITORS: Visitor[] = [
  {
    id: "vis_01HQXYZ1234567890ABCDEF",
    name: "Ahmad bin Abdullah",
    idHash: "sha256:a1b2c3d4...f8g9",
    purpose: "Visiting family",
    hostResident: "Siti binti Hassan",
    hostUnit: "A-12-03",
    credentialType: "VisitorPass",
    status: "pending",
    arrivedAt: new Date(Date.now() - 300000).toISOString(),
    expiresAt: new Date(Date.now() + 3600000).toISOString(),
  },
  {
    id: "vis_01HQXYZ2234567890ABCDEF",
    name: "Lee Wei Ming",
    idHash: "sha256:e5f6g7h8...j0k1",
    purpose: "Delivery",
    hostResident: "Tan Mei Ling",
    hostUnit: "B-05-08",
    credentialType: "VisitorPass",
    status: "pending",
    arrivedAt: new Date(Date.now() - 600000).toISOString(),
  },
  {
    id: "vis_01HQXYZ3234567890ABCDEF",
    name: "Kumar Rajan",
    idHash: "sha256:l2m3n4o5...q7r8",
    purpose: "Plumbing repair",
    hostResident: "Wong Chee Keong",
    hostUnit: "C-08-15",
    credentialType: "ContractorBadge",
    status: "verified",
    arrivedAt: new Date(Date.now() - 1800000).toISOString(),
    expiresAt: new Date(Date.now() + 7200000).toISOString(),
  },
  {
    id: "vis_01HQXYZ4234567890ABCDEF",
    name: "Fatimah binti Omar",
    idHash: "sha256:s9t0u1v2...x4y5",
    purpose: "Social visit",
    hostResident: "Aminah binti Yusof",
    hostUnit: "A-03-12",
    credentialType: "VisitorPass",
    status: "denied",
    arrivedAt: new Date(Date.now() - 2400000).toISOString(),
  },
];

// Simplified API wrapper for the store
export const visitorsApi = {
  list: async (): Promise<Visitor[]> => {
    // In production, this would call the real API
    // For now, return mock data
    return new Promise((resolve) => {
      setTimeout(() => resolve([...MOCK_VISITORS]), 500);
    });
  },

  updateStatus: async (id: string, status: VisitorStatus): Promise<void> => {
    // In production, this would call the real API
    return new Promise((resolve) => {
      setTimeout(() => {
        const visitor = MOCK_VISITORS.find((v) => v.id === id);
        if (visitor) {
          visitor.status = status;
        }
        resolve();
      }, 300);
    });
  },
};
