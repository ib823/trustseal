import { apiClient } from "./client";

export interface AnalyticsOverview {
  totalEntries: number;
  totalEntriesChange: number;
  uniqueResidents: number;
  uniqueResidentsChange: number;
  denialRate: number;
  denialRateChange: number;
  avgResponseTime: number;
  avgResponseTimeChange: number;
}

export interface PeakHoursData {
  hour: string;
  entries: number;
}

export interface DenialReason {
  reason: string;
  count: number;
  percentage: number;
}

export interface EntryTrend {
  date: string;
  entries: number;
  exits: number;
}

export async function getAnalyticsOverview(
  workspaceId: string,
  period: "month" | "quarter" | "year" = "month"
) {
  return apiClient.get<AnalyticsOverview>(
    `/api/v1/workspaces/${workspaceId}/analytics/overview`,
    { period }
  );
}

export async function getPeakHours(
  workspaceId: string,
  date?: string
) {
  const params: Record<string, string> = {};
  if (date) params.date = date;

  return apiClient.get<PeakHoursData[]>(
    `/api/v1/workspaces/${workspaceId}/analytics/peak-hours`,
    params
  );
}

export async function getDenialReasons(
  workspaceId: string,
  period: "week" | "month" | "quarter" = "month"
) {
  return apiClient.get<DenialReason[]>(
    `/api/v1/workspaces/${workspaceId}/analytics/denial-reasons`,
    { period }
  );
}

export async function getEntryTrends(
  workspaceId: string,
  days: number = 30
) {
  return apiClient.get<EntryTrend[]>(
    `/api/v1/workspaces/${workspaceId}/analytics/entry-trends`,
    { days: days.toString() }
  );
}
