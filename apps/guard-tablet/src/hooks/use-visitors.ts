"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  getVisitors,
  getVisitor,
  submitEntryDecision,
  getDailyStats,
  type VisitorListParams,
  type EntryDecision,
} from "@/lib/api/visitors";

const TENANT_ID = "TEN_01HQ123ABC"; // TODO: Get from auth context

export function useVisitorQueue(params?: VisitorListParams) {
  return useQuery({
    queryKey: ["visitors", params],
    queryFn: async () => {
      const result = await getVisitors(TENANT_ID, params);
      return result;
    },
    refetchInterval: 30_000, // Auto-refresh every 30 seconds
  });
}

export function useVisitor(visitorId: string | null) {
  return useQuery({
    queryKey: ["visitor", visitorId],
    queryFn: () =>
      visitorId ? getVisitor(TENANT_ID, visitorId) : Promise.resolve(null),
    enabled: !!visitorId,
  });
}

export function useEntryDecision() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (decision: EntryDecision) =>
      submitEntryDecision(TENANT_ID, decision),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["visitors"] });
      queryClient.invalidateQueries({ queryKey: ["daily-stats"] });
    },
  });
}

export function useDailyStats() {
  return useQuery({
    queryKey: ["daily-stats"],
    queryFn: () => getDailyStats(TENANT_ID),
    refetchInterval: 60_000, // Refresh every minute
  });
}
