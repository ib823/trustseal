"use client";

import { useQuery } from "@tanstack/react-query";
import { CheckCircle2, XCircle, ArrowUpRight, ArrowDownLeft } from "lucide-react";
import Link from "next/link";

import { Card } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn, formatDateTime, truncateId } from "@/lib/utils";

interface AccessLog {
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

async function fetchAccessLogs(): Promise<AccessLog[]> {
  // TODO: Replace with actual API call
  const now = Date.now();
  return [
    {
      id: "log_01HQ001",
      residentId: "res_01HQ123ABC",
      residentName: "Ahmad bin Ismail",
      unitNumber: "A-12-03",
      verifierId: "ver_01HQ123ABC",
      verifierName: "Main Lobby",
      direction: "entry",
      status: "granted",
      credentialId: "cred_01HQ789XYZ",
      timestamp: new Date(now - 2 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQ002",
      residentId: "res_01HQ456DEF",
      residentName: "Sarah Lee",
      unitNumber: "B-08-15",
      verifierId: "ver_01HQ456DEF",
      verifierName: "Parking B1",
      direction: "entry",
      status: "granted",
      credentialId: "cred_01HQ012ABC",
      timestamp: new Date(now - 5 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQ003",
      residentName: "Unknown",
      unitNumber: "-",
      verifierId: "ver_01HQ789GHI",
      verifierName: "Side Gate",
      direction: "entry",
      status: "denied",
      reason: "Credential revoked",
      timestamp: new Date(now - 8 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQ004",
      residentId: "res_01HQ123ABC",
      residentName: "Ahmad bin Ismail",
      unitNumber: "A-12-03",
      verifierId: "ver_01HQ012JKL",
      verifierName: "Gym Access",
      direction: "exit",
      status: "granted",
      credentialId: "cred_01HQ789XYZ",
      timestamp: new Date(now - 15 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQ005",
      residentId: "res_01HQ789GHI",
      residentName: "Raj Kumar",
      unitNumber: "C-15-01",
      verifierId: "ver_01HQ345MNO",
      verifierName: "Pool Gate",
      direction: "entry",
      status: "denied",
      reason: "Credential suspended",
      timestamp: new Date(now - 22 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQ006",
      residentId: "res_01HQ012JKL",
      residentName: "Mei Ling Tan",
      unitNumber: "A-05-08",
      verifierId: "ver_01HQ123ABC",
      verifierName: "Main Lobby",
      direction: "exit",
      status: "granted",
      credentialId: "cred_01HQ456DEF",
      timestamp: new Date(now - 35 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQ007",
      residentName: "Unknown",
      unitNumber: "-",
      verifierId: "ver_01HQ123ABC",
      verifierName: "Main Lobby",
      direction: "entry",
      status: "denied",
      reason: "Invalid credential format",
      timestamp: new Date(now - 42 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQ008",
      residentId: "res_01HQ456DEF",
      residentName: "Sarah Lee",
      unitNumber: "B-08-15",
      verifierId: "ver_01HQ456DEF",
      verifierName: "Parking B1",
      direction: "exit",
      status: "granted",
      credentialId: "cred_01HQ012ABC",
      timestamp: new Date(now - 55 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQ009",
      residentId: "res_01HQ123ABC",
      residentName: "Ahmad bin Ismail",
      unitNumber: "A-12-03",
      verifierId: "ver_01HQ012JKL",
      verifierName: "Gym Access",
      direction: "entry",
      status: "granted",
      credentialId: "cred_01HQ789XYZ",
      timestamp: new Date(now - 68 * 60 * 1000).toISOString(),
    },
    {
      id: "log_01HQ010",
      residentId: "res_01HQ345MNO",
      residentName: "Muthu Rajan",
      unitNumber: "D-20-02",
      verifierId: "ver_01HQ123ABC",
      verifierName: "Main Lobby",
      direction: "entry",
      status: "denied",
      reason: "Credential expired",
      timestamp: new Date(now - 85 * 60 * 1000).toISOString(),
    },
  ];
}

export function AccessLogTable() {
  const { data: logs, isLoading } = useQuery({
    queryKey: ["access-logs"],
    queryFn: fetchAccessLogs,
    staleTime: 10_000,
    refetchInterval: 30_000,
  });

  if (isLoading || !logs) {
    return null;
  }

  return (
    <Card>
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b">
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Time
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Resident
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Verifier
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Direction
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Status
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Details
              </th>
            </tr>
          </thead>
          <tbody>
            {logs.map((log) => (
              <tr
                key={log.id}
                className={cn(
                  "border-b last:border-0 hover:bg-muted/50",
                  log.status === "denied" && "bg-error/5"
                )}
              >
                <td className="px-4 py-3">
                  <div>
                    <p className="text-sm font-medium">
                      {formatDateTime(log.timestamp)}
                    </p>
                    <p className="font-mono text-xs text-muted-foreground">
                      {truncateId(log.id)}
                    </p>
                  </div>
                </td>
                <td className="px-4 py-3">
                  <div>
                    <p className="font-medium">
                      {log.residentId ? (
                        <Link
                          href={`/residents/${log.residentId}` as `/residents/${string}`}
                          className="hover:underline"
                        >
                          {log.residentName}
                        </Link>
                      ) : (
                        <span className="text-muted-foreground">
                          {log.residentName}
                        </span>
                      )}
                    </p>
                    <p className="text-sm text-muted-foreground">
                      {log.unitNumber}
                    </p>
                  </div>
                </td>
                <td className="px-4 py-3">
                  <div>
                    <p className="text-sm font-medium">{log.verifierName}</p>
                    <p className="font-mono text-xs text-muted-foreground">
                      {truncateId(log.verifierId)}
                    </p>
                  </div>
                </td>
                <td className="px-4 py-3">
                  <div className="flex items-center gap-1.5">
                    {log.direction === "entry" ? (
                      <ArrowDownLeft className="h-4 w-4 text-success" />
                    ) : (
                      <ArrowUpRight className="h-4 w-4 text-primary" />
                    )}
                    <span className="text-sm capitalize">{log.direction}</span>
                  </div>
                </td>
                <td className="px-4 py-3">
                  <div className="flex items-center gap-2">
                    {log.status === "granted" ? (
                      <CheckCircle2 className="h-4 w-4 text-success" />
                    ) : (
                      <XCircle className="h-4 w-4 text-error" />
                    )}
                    <Badge
                      variant={log.status === "granted" ? "success" : "error"}
                    >
                      {log.status}
                    </Badge>
                  </div>
                </td>
                <td className="px-4 py-3 text-sm">
                  {log.reason ? (
                    <span className="text-error">{log.reason}</span>
                  ) : log.credentialId ? (
                    <span className="font-mono text-xs text-muted-foreground">
                      {truncateId(log.credentialId)}
                    </span>
                  ) : (
                    <span className="text-muted-foreground">-</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <div className="flex items-center justify-between border-t px-4 py-3">
        <p className="text-sm text-muted-foreground">
          Showing 10 of 847 entries
        </p>
        <div className="flex gap-2">
          <Button variant="outline" size="sm" disabled>
            Previous
          </Button>
          <Button variant="outline" size="sm">
            Next
          </Button>
        </div>
      </div>
    </Card>
  );
}
