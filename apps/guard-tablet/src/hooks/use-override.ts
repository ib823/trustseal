"use client";

import { useQuery } from "@tanstack/react-query";
import { getOverrideReasonCodes } from "@/lib/api/visitors";

const TENANT_ID = "TEN_01HQ123ABC"; // TODO: Get from auth context

export function useOverrideReasonCodes() {
  return useQuery({
    queryKey: ["override-codes"],
    queryFn: () => getOverrideReasonCodes(TENANT_ID),
    staleTime: 5 * 60 * 1000, // Cache for 5 minutes
  });
}

export async function requestBiometricAuth(): Promise<string | null> {
  // In production, this would use the Web Authentication API
  // For now, simulate with a confirm dialog
  return new Promise((resolve) => {
    const confirmed = window.confirm(
      "Biometric authentication required.\n\nClick OK to simulate successful authentication."
    );
    if (confirmed) {
      resolve(`bio_${Date.now()}_${Math.random().toString(36).slice(2)}`);
    } else {
      resolve(null);
    }
  });
}
