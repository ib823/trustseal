"use client";

import { useTranslations } from "next-intl";
import { WifiOff, CloudOff } from "lucide-react";
import { useOfflineStore } from "@/lib/stores/offline-store";
import { cn } from "@/lib/utils";

export function OfflineIndicator() {
  const t = useTranslations();
  const { isOnline, queue } = useOfflineStore();

  if (isOnline && queue.length === 0) {
    return null;
  }

  return (
    <div
      className={cn(
        "flex items-center justify-between px-6 py-3",
        !isOnline ? "bg-error/10" : "bg-warning/10"
      )}
    >
      <div className="flex items-center gap-3">
        {!isOnline ? (
          <WifiOff className="h-5 w-5 text-error" />
        ) : (
          <CloudOff className="h-5 w-5 text-warning" />
        )}
        <div>
          <p
            className={cn(
              "text-sm font-medium",
              !isOnline ? "text-error" : "text-warning"
            )}
          >
            {!isOnline ? t("offline.banner") : t("offline.syncPending")}
          </p>
          <p className="text-xs text-muted-foreground">
            {!isOnline
              ? t("offline.description")
              : t("offline.queuedCount", { count: queue.length })}
          </p>
        </div>
      </div>

      {queue.length > 0 && (
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">
            {t("offline.queuedCount", { count: queue.length })}
          </span>
        </div>
      )}
    </div>
  );
}
