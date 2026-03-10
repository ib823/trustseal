"use client";

import { useQuery } from "@tanstack/react-query";
import {
  Circle,
  MoreVertical,
  Signal,
  SignalLow,
  SignalZero,
  Activity,
  Clock,
  MapPin,
} from "lucide-react";

import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { cn, formatRelativeTime, truncateId } from "@/lib/utils";

interface Verifier {
  id: string;
  name: string;
  location: string;
  model: string;
  firmwareVersion: string;
  status: "online" | "offline" | "degraded";
  signalStrength: "strong" | "medium" | "weak";
  lastSeen: string;
  todayEvents: number;
  todayDenied: number;
  uptime: string;
}

async function fetchVerifiers(): Promise<Verifier[]> {
  // TODO: Replace with actual API call
  return [
    {
      id: "ver_01HQ123ABC",
      name: "Main Lobby",
      location: "Ground Floor, Main Entrance",
      model: "VaultPass Edge V2",
      firmwareVersion: "1.2.3",
      status: "online",
      signalStrength: "strong",
      lastSeen: new Date().toISOString(),
      todayEvents: 342,
      todayDenied: 5,
      uptime: "15d 4h",
    },
    {
      id: "ver_01HQ456DEF",
      name: "Parking B1",
      location: "Basement 1, Car Park Entry",
      model: "VaultPass Edge V2",
      firmwareVersion: "1.2.3",
      status: "online",
      signalStrength: "strong",
      lastSeen: new Date().toISOString(),
      todayEvents: 189,
      todayDenied: 2,
      uptime: "15d 4h",
    },
    {
      id: "ver_01HQ789GHI",
      name: "Side Gate",
      location: "East Wing, Pedestrian Gate",
      model: "VaultPass Edge V1",
      firmwareVersion: "1.1.8",
      status: "degraded",
      signalStrength: "medium",
      lastSeen: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
      todayEvents: 56,
      todayDenied: 1,
      uptime: "3d 12h",
    },
    {
      id: "ver_01HQ012JKL",
      name: "Gym Access",
      location: "Level 3, Amenities Floor",
      model: "VaultPass Edge V2",
      firmwareVersion: "1.2.3",
      status: "online",
      signalStrength: "strong",
      lastSeen: new Date().toISOString(),
      todayEvents: 78,
      todayDenied: 0,
      uptime: "15d 4h",
    },
    {
      id: "ver_01HQ345MNO",
      name: "Pool Gate",
      location: "Level 5, Pool Deck",
      model: "VaultPass Edge V1",
      firmwareVersion: "1.1.8",
      status: "offline",
      signalStrength: "weak",
      lastSeen: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
      todayEvents: 12,
      todayDenied: 0,
      uptime: "0h",
    },
    {
      id: "ver_01HQ678PQR",
      name: "Loading Bay",
      location: "Basement 2, Service Area",
      model: "VaultPass Edge V2",
      firmwareVersion: "1.2.3",
      status: "online",
      signalStrength: "medium",
      lastSeen: new Date().toISOString(),
      todayEvents: 23,
      todayDenied: 3,
      uptime: "8d 19h",
    },
  ];
}

const statusColors = {
  online: "bg-success",
  offline: "bg-error",
  degraded: "bg-warning",
};

const SignalIcons = {
  strong: Signal,
  medium: SignalLow,
  weak: SignalZero,
};

export function VerifierGrid() {
  const { data: verifiers, isLoading } = useQuery({
    queryKey: ["verifiers"],
    queryFn: fetchVerifiers,
    staleTime: 15_000,
    refetchInterval: 30_000,
  });

  if (isLoading || !verifiers) {
    return null;
  }

  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {verifiers.map((verifier) => {
        const SignalIcon = SignalIcons[verifier.signalStrength];
        return (
          <Card
            key={verifier.id}
            className={cn(
              "transition-colors",
              verifier.status === "offline" && "border-error/50",
              verifier.status === "degraded" && "border-warning/50"
            )}
          >
            <CardHeader className="flex flex-row items-start justify-between pb-2">
              <div className="space-y-1">
                <div className="flex items-center gap-2">
                  <Circle
                    className={cn(
                      "h-2 w-2 fill-current",
                      statusColors[verifier.status]
                    )}
                  />
                  <h3 className="font-semibold">{verifier.name}</h3>
                </div>
                <p className="flex items-center gap-1 text-sm text-muted-foreground">
                  <MapPin className="h-3 w-3" />
                  {verifier.location}
                </p>
              </div>
              <Button variant="ghost" size="icon" className="h-8 w-8">
                <MoreVertical className="h-4 w-4" />
              </Button>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-2">
                  <SignalIcon
                    className={cn(
                      "h-4 w-4",
                      verifier.signalStrength === "strong" && "text-success",
                      verifier.signalStrength === "medium" && "text-warning",
                      verifier.signalStrength === "weak" && "text-error"
                    )}
                  />
                  <span className="capitalize">{verifier.signalStrength}</span>
                </div>
                <Badge
                  variant={
                    verifier.status === "online"
                      ? "success"
                      : verifier.status === "offline"
                      ? "error"
                      : "warning"
                  }
                >
                  {verifier.status}
                </Badge>
              </div>

              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <p className="text-muted-foreground">Today</p>
                  <p className="flex items-center gap-1 font-medium">
                    <Activity className="h-3 w-3 text-success" />
                    {verifier.todayEvents} events
                  </p>
                  {verifier.todayDenied > 0 && (
                    <p className="text-xs text-error">
                      {verifier.todayDenied} denied
                    </p>
                  )}
                </div>
                <div>
                  <p className="text-muted-foreground">Uptime</p>
                  <p className="flex items-center gap-1 font-medium">
                    <Clock className="h-3 w-3" />
                    {verifier.uptime}
                  </p>
                </div>
              </div>

              <div className="space-y-1 border-t pt-3 text-xs text-muted-foreground">
                <div className="flex justify-between">
                  <span>Model</span>
                  <span className="font-medium text-foreground">
                    {verifier.model}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>Firmware</span>
                  <span className="font-mono">{verifier.firmwareVersion}</span>
                </div>
                <div className="flex justify-between">
                  <span>Last seen</span>
                  <span>
                    {verifier.status === "online"
                      ? "Now"
                      : formatRelativeTime(verifier.lastSeen)}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span>ID</span>
                  <span className="font-mono">{truncateId(verifier.id)}</span>
                </div>
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}
