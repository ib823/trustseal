"use client";

import { useQuery } from "@tanstack/react-query";
import { CheckCircle2, XCircle, Clock, ArrowRight } from "lucide-react";
import Link from "next/link";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn, formatRelativeTime } from "@/lib/utils";

interface AccessLog {
  id: string;
  residentName: string;
  unitNumber: string;
  verifierName: string;
  status: "granted" | "denied" | "pending";
  timestamp: string;
  reason?: string;
}

async function fetchRecentActivity(): Promise<AccessLog[]> {
  // TODO: Replace with actual API call
  return [
    {
      id: "log_01HQXYZ123ABC",
      residentName: "Ahmad bin Ismail",
      unitNumber: "A-12-03",
      verifierName: "Main Lobby",
      status: "granted",
      timestamp: new Date(Date.now() - 2 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQXYZ456DEF",
      residentName: "Sarah Lee",
      unitNumber: "B-08-15",
      verifierName: "Parking B1",
      status: "granted",
      timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQXYZ789GHI",
      residentName: "Unknown",
      unitNumber: "-",
      verifierName: "Side Gate",
      status: "denied",
      reason: "Credential revoked",
      timestamp: new Date(Date.now() - 8 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQXYZ012JKL",
      residentName: "Raj Kumar",
      unitNumber: "C-15-01",
      verifierName: "Gym",
      status: "granted",
      timestamp: new Date(Date.now() - 12 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQXYZ345MNO",
      residentName: "Mei Ling",
      unitNumber: "A-05-08",
      verifierName: "Pool Access",
      status: "granted",
      timestamp: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
    },
  ];
}

function RecentActivitySkeleton() {
  return (
    <div className="space-y-4">
      {Array.from({ length: 5 }).map((_, i) => (
        <div key={i} className="flex items-center gap-4">
          <div className="h-8 w-8 animate-pulse rounded-full bg-muted" />
          <div className="flex-1 space-y-1">
            <div className="h-4 w-32 animate-pulse rounded bg-muted" />
            <div className="h-3 w-48 animate-pulse rounded bg-muted" />
          </div>
          <div className="h-5 w-16 animate-pulse rounded-full bg-muted" />
        </div>
      ))}
    </div>
  );
}

const statusIcons = {
  granted: CheckCircle2,
  denied: XCircle,
  pending: Clock,
};

const statusColors = {
  granted: "text-success",
  denied: "text-error",
  pending: "text-warning",
};

export function RecentActivity() {
  const { data: logs, isLoading } = useQuery({
    queryKey: ["recent-activity"],
    queryFn: fetchRecentActivity,
    staleTime: 10_000,
    refetchInterval: 30_000,
  });

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>Recent Activity</CardTitle>
        <Button variant="ghost" size="sm" asChild>
          <Link href="/access-logs" className="gap-1">
            View all
            <ArrowRight className="h-4 w-4" />
          </Link>
        </Button>
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <RecentActivitySkeleton />
        ) : (
          <div className="space-y-4">
            {logs?.map((log) => {
              const StatusIcon = statusIcons[log.status];
              return (
                <div key={log.id} className="flex items-center gap-4">
                  <div
                    className={cn(
                      "flex h-8 w-8 items-center justify-center rounded-full",
                      log.status === "granted" && "bg-success/10",
                      log.status === "denied" && "bg-error/10",
                      log.status === "pending" && "bg-warning/10"
                    )}
                  >
                    <StatusIcon
                      className={cn("h-4 w-4", statusColors[log.status])}
                    />
                  </div>
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium truncate">
                      {log.residentName}
                      <span className="ml-2 text-muted-foreground font-normal">
                        {log.unitNumber}
                      </span>
                    </p>
                    <p className="text-xs text-muted-foreground truncate">
                      {log.verifierName}
                      {log.reason && (
                        <span className="text-error"> - {log.reason}</span>
                      )}
                    </p>
                  </div>
                  <div className="text-right">
                    <Badge
                      variant={
                        log.status === "granted"
                          ? "success"
                          : log.status === "denied"
                          ? "error"
                          : "warning"
                      }
                    >
                      {log.status}
                    </Badge>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {formatRelativeTime(log.timestamp)}
                    </p>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
