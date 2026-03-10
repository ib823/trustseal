"use client";

import { useQuery } from "@tanstack/react-query";
import { Circle, WifiOff, MoreVertical } from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { cn, formatRelativeTime } from "@/lib/utils";

interface Verifier {
  id: string;
  name: string;
  location: string;
  status: "online" | "offline" | "degraded";
  lastSeen: string;
  todayEvents: number;
}

async function fetchVerifiers(): Promise<Verifier[]> {
  // TODO: Replace with actual API call
  return [
    {
      id: "ver_01HQ123ABC",
      name: "Main Lobby",
      location: "Ground Floor",
      status: "online",
      lastSeen: new Date().toISOString(),
      todayEvents: 342,
    },
    {
      id: "ver_01HQ456DEF",
      name: "Parking B1",
      location: "Basement 1",
      status: "online",
      lastSeen: new Date().toISOString(),
      todayEvents: 189,
    },
    {
      id: "ver_01HQ789GHI",
      name: "Side Gate",
      location: "East Wing",
      status: "degraded",
      lastSeen: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
      todayEvents: 56,
    },
    {
      id: "ver_01HQ012JKL",
      name: "Gym Access",
      location: "Level 3",
      status: "online",
      lastSeen: new Date().toISOString(),
      todayEvents: 78,
    },
    {
      id: "ver_01HQ345MNO",
      name: "Pool Gate",
      location: "Level 5",
      status: "offline",
      lastSeen: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
      todayEvents: 12,
    },
  ];
}

function VerifierSkeleton() {
  return (
    <div className="space-y-3">
      {Array.from({ length: 5 }).map((_, i) => (
        <div
          key={i}
          className="flex items-center gap-3 rounded-lg border p-3"
        >
          <div className="h-2 w-2 animate-pulse rounded-full bg-muted" />
          <div className="flex-1 space-y-1">
            <div className="h-4 w-24 animate-pulse rounded bg-muted" />
            <div className="h-3 w-16 animate-pulse rounded bg-muted" />
          </div>
          <div className="h-4 w-12 animate-pulse rounded bg-muted" />
        </div>
      ))}
    </div>
  );
}

const statusColors = {
  online: "bg-success",
  offline: "bg-error",
  degraded: "bg-warning",
};

const statusLabels = {
  online: "Online",
  offline: "Offline",
  degraded: "Degraded",
};

export function VerifierStatus() {
  const { data: verifiers, isLoading } = useQuery({
    queryKey: ["verifier-status"],
    queryFn: fetchVerifiers,
    staleTime: 15_000,
    refetchInterval: 30_000,
  });

  const onlineCount = verifiers?.filter((v) => v.status === "online").length ?? 0;
  const totalCount = verifiers?.length ?? 0;

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <div>
          <CardTitle>Verifier Status</CardTitle>
          <p className="text-sm text-muted-foreground">
            {onlineCount}/{totalCount} online
          </p>
        </div>
        {verifiers?.some((v) => v.status !== "online") && (
          <WifiOff className="h-4 w-4 text-warning" />
        )}
      </CardHeader>
      <CardContent>
        {isLoading ? (
          <VerifierSkeleton />
        ) : (
          <div className="space-y-3">
            {verifiers?.map((verifier) => (
              <div
                key={verifier.id}
                className={cn(
                  "flex items-center gap-3 rounded-lg border p-3 transition-colors",
                  verifier.status === "offline" && "border-error/50 bg-error/5",
                  verifier.status === "degraded" && "border-warning/50 bg-warning/5"
                )}
              >
                <Circle
                  className={cn(
                    "h-2 w-2 fill-current",
                    statusColors[verifier.status]
                  )}
                />
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">{verifier.name}</p>
                  <p className="text-xs text-muted-foreground truncate">
                    {verifier.location}
                  </p>
                </div>
                <div className="text-right">
                  <p className="text-xs font-medium">
                    {verifier.todayEvents} events
                  </p>
                  <p className="text-xs text-muted-foreground">
                    {verifier.status === "online"
                      ? statusLabels[verifier.status]
                      : formatRelativeTime(verifier.lastSeen)}
                  </p>
                </div>
                <Button variant="ghost" size="icon" className="h-8 w-8">
                  <MoreVertical className="h-4 w-4" />
                </Button>
              </div>
            ))}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
