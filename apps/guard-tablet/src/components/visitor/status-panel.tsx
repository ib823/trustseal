"use client";

import { useTranslations } from "next-intl";
import {
  Wifi,
  WifiOff,
  Bluetooth,
  Smartphone,
  CheckCircle,
  XCircle,
  AlertTriangle,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { useOfflineStore, type QueuedAction } from "@/lib/stores/offline-store";
import { useVisitorStore } from "@/lib/stores/visitor-store";
import { cn } from "@/lib/utils";

export function StatusPanel() {
  const t = useTranslations();
  const { isOnline, queue } = useOfflineStore();
  const { visitors } = useVisitorStore();

  // Recent activity from visitors that have been processed
  const recentActivity = visitors
    .filter((v) => v.status !== "pending")
    .sort(
      (a, b) =>
        new Date(b.arrivedAt).getTime() - new Date(a.arrivedAt).getTime()
    )
    .slice(0, 5);

  const formatTime = (timestamp: number) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
  };

  const getActionIcon = (type: QueuedAction["type"]) => {
    switch (type) {
      case "verify":
        return <CheckCircle className="h-4 w-4 text-success" />;
      case "deny":
        return <XCircle className="h-4 w-4 text-error" />;
      case "override":
        return <AlertTriangle className="h-4 w-4 text-warning" />;
    }
  };

  return (
    <div className="flex h-full flex-col gap-4 p-4">
      {/* System Status */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base">{t("status.systemStatus")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              {isOnline ? (
                <Wifi className="h-4 w-4 text-success" />
              ) : (
                <WifiOff className="h-4 w-4 text-error" />
              )}
              <span className="text-sm">{t("status.apiConnection")}</span>
            </div>
            <Badge variant={isOnline ? "success" : "error"}>
              {isOnline ? t("status.connected") : t("status.disconnected")}
            </Badge>
          </div>

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Bluetooth className="h-4 w-4 text-success" />
              <span className="text-sm">{t("status.bleScanner")}</span>
            </div>
            <Badge variant="success">{t("status.active")}</Badge>
          </div>

          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Smartphone className="h-4 w-4 text-success" />
              <span className="text-sm">{t("status.nfcReader")}</span>
            </div>
            <Badge variant="success">{t("status.ready")}</Badge>
          </div>
        </CardContent>
      </Card>

      {/* Queued Actions */}
      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center justify-between text-base">
            <span>{t("status.queuedActions")}</span>
            {queue.length > 0 && (
              <Badge variant="warning">{queue.length}</Badge>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {queue.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t("status.noQueuedActions")}
            </p>
          ) : (
            <div className="space-y-2">
              {queue.map((action) => (
                <div
                  key={`${action.type}-${action.visitorId}-${action.timestamp}`}
                  className="flex items-center justify-between rounded-md bg-muted/50 px-3 py-2"
                >
                  <div className="flex items-center gap-2">
                    {getActionIcon(action.type)}
                    <span className="text-sm capitalize">
                      {t(`status.action.${action.type}`)}
                    </span>
                  </div>
                  <span className="text-xs text-muted-foreground">
                    {formatTime(action.timestamp)}
                  </span>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Recent Activity */}
      <Card className="flex-1 overflow-hidden">
        <CardHeader className="pb-3">
          <CardTitle className="text-base">{t("status.recentActivity")}</CardTitle>
        </CardHeader>
        <CardContent className="overflow-y-auto">
          {recentActivity.length === 0 ? (
            <p className="text-sm text-muted-foreground">
              {t("status.noActivity")}
            </p>
          ) : (
            <div className="space-y-3">
              {recentActivity.map((visitor) => (
                <div
                  key={visitor.id}
                  className="flex items-start gap-3 border-b pb-3 last:border-0 last:pb-0"
                >
                  <div
                    className={cn(
                      "mt-0.5 flex h-6 w-6 items-center justify-center rounded-full",
                      visitor.status === "verified"
                        ? "bg-success/10"
                        : "bg-error/10"
                    )}
                  >
                    {visitor.status === "verified" ? (
                      <CheckCircle className="h-4 w-4 text-success" />
                    ) : (
                      <XCircle className="h-4 w-4 text-error" />
                    )}
                  </div>
                  <div className="flex-1">
                    <p className="text-sm">
                      {visitor.status === "verified"
                        ? t("status.activity.verified", { name: visitor.name })
                        : t("status.activity.denied", { name: visitor.name })}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {new Date(visitor.arrivedAt).toLocaleTimeString([], {
                        hour: "2-digit",
                        minute: "2-digit",
                      })}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
