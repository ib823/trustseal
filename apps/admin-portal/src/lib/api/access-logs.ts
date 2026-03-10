import { apiClient } from "./client";

export interface AccessLog {
  id: string;
  residentId?: string;
  residentName: string;
  unitNumber: string;
  verifierId: string;
  verifierName: string;
  direction: "entry" | "exit";
  status: "granted" | "denied";
  reason?: string;
  credentialId?: string;
  timestamp: string;
}

export interface AccessLogFilters {
  status?: "granted" | "denied";
  direction?: "entry" | "exit";
  verifierId?: string;
  residentId?: string;
  startDate?: string;
  endDate?: string;
  limit?: number;
  offset?: number;
}

export interface AccessLogStats {
  totalEntries: number;
  totalExits: number;
  totalDenied: number;
  uniqueResidents: number;
}

export async function getAccessLogs(
  workspaceId: string,
  filters?: AccessLogFilters
) {
  const params: Record<string, string> = {};

  if (filters) {
    if (filters.status) params.status = filters.status;
    if (filters.direction) params.direction = filters.direction;
    if (filters.verifierId) params.verifier_id = filters.verifierId;
    if (filters.residentId) params.resident_id = filters.residentId;
    if (filters.startDate) params.start_date = filters.startDate;
    if (filters.endDate) params.end_date = filters.endDate;
    if (filters.limit) params.limit = filters.limit.toString();
    if (filters.offset) params.offset = filters.offset.toString();
  }

  return apiClient.get<{ logs: AccessLog[]; total: number }>(
    `/api/v1/workspaces/${workspaceId}/access-logs`,
    params
  );
}

export async function getAccessLog(workspaceId: string, logId: string) {
  return apiClient.get<AccessLog>(
    `/api/v1/workspaces/${workspaceId}/access-logs/${logId}`
  );
}

export async function getAccessLogStats(
  workspaceId: string,
  period: "today" | "week" | "month"
) {
  return apiClient.get<AccessLogStats>(
    `/api/v1/workspaces/${workspaceId}/access-logs/stats`,
    { period }
  );
}

export async function getAccessLogHourly(
  workspaceId: string,
  date: string
) {
  return apiClient.get<{ hour: string; entries: number; exits: number }[]>(
    `/api/v1/workspaces/${workspaceId}/access-logs/hourly`,
    { date }
  );
}
