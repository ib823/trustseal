"use client";

import { useQuery } from "@tanstack/react-query";
import { MoreHorizontal, Mail, Phone, Shield, ShieldOff, Eye } from "lucide-react";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Card } from "@/components/ui/card";
import { cn, truncateId, formatDate } from "@/lib/utils";

interface Resident {
  id: string;
  name: string;
  email: string;
  phone: string;
  unitNumber: string;
  credentialStatus: "active" | "suspended" | "pending" | "revoked";
  credentialId?: string;
  lastAccess?: string;
  createdAt: string;
}

async function fetchResidents(): Promise<Resident[]> {
  // TODO: Replace with actual API call
  return [
    {
      id: "res_01HQ123ABC",
      name: "Ahmad bin Ismail",
      email: "ahmad@example.com",
      phone: "+60 12-345 6789",
      unitNumber: "A-12-03",
      credentialStatus: "active",
      credentialId: "cred_01HQ789XYZ",
      lastAccess: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
      createdAt: "2024-01-15T08:00:00Z",
    },
    {
      id: "res_01HQ456DEF",
      name: "Sarah Lee",
      email: "sarah.lee@example.com",
      phone: "+60 11-234 5678",
      unitNumber: "B-08-15",
      credentialStatus: "active",
      credentialId: "cred_01HQ012ABC",
      lastAccess: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString(),
      createdAt: "2024-01-20T10:30:00Z",
    },
    {
      id: "res_01HQ789GHI",
      name: "Raj Kumar",
      email: "raj.k@example.com",
      phone: "+60 10-987 6543",
      unitNumber: "C-15-01",
      credentialStatus: "suspended",
      credentialId: "cred_01HQ345DEF",
      lastAccess: new Date(Date.now() - 48 * 60 * 60 * 1000).toISOString(),
      createdAt: "2024-02-01T14:00:00Z",
    },
    {
      id: "res_01HQ012JKL",
      name: "Mei Ling Tan",
      email: "meiling@example.com",
      phone: "+60 13-456 7890",
      unitNumber: "A-05-08",
      credentialStatus: "pending",
      createdAt: "2024-03-01T09:00:00Z",
    },
    {
      id: "res_01HQ345MNO",
      name: "Muthu Rajan",
      email: "muthu.r@example.com",
      phone: "+60 14-567 8901",
      unitNumber: "D-20-02",
      credentialStatus: "revoked",
      credentialId: "cred_01HQ678GHI",
      lastAccess: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
      createdAt: "2023-12-01T11:00:00Z",
    },
  ];
}

const statusVariants = {
  active: "success",
  suspended: "warning",
  pending: "secondary",
  revoked: "error",
} as const;

const statusIcons = {
  active: Shield,
  suspended: ShieldOff,
  pending: Shield,
  revoked: ShieldOff,
};

function getInitials(name: string): string {
  return name
    .split(" ")
    .map((n) => n[0])
    .join("")
    .toUpperCase()
    .slice(0, 2);
}

export function ResidentTable() {
  const { data: residents, isLoading } = useQuery({
    queryKey: ["residents"],
    queryFn: fetchResidents,
    staleTime: 30_000,
  });

  if (isLoading || !residents) {
    return null;
  }

  return (
    <Card>
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b">
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Resident
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Unit
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Contact
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Credential
              </th>
              <th className="px-4 py-3 text-left text-sm font-medium text-muted-foreground">
                Last Access
              </th>
              <th className="px-4 py-3 text-right text-sm font-medium text-muted-foreground">
                Actions
              </th>
            </tr>
          </thead>
          <tbody>
            {residents.map((resident) => {
              const StatusIcon = statusIcons[resident.credentialStatus];
              return (
                <tr
                  key={resident.id}
                  className="border-b last:border-0 hover:bg-muted/50"
                >
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-3">
                      <Avatar className="h-9 w-9">
                        <AvatarFallback className="text-xs">
                          {getInitials(resident.name)}
                        </AvatarFallback>
                      </Avatar>
                      <div>
                        <p className="font-medium">{resident.name}</p>
                        <p className="font-mono text-xs text-muted-foreground">
                          {truncateId(resident.id)}
                        </p>
                      </div>
                    </div>
                  </td>
                  <td className="px-4 py-3">
                    <span className="font-medium">{resident.unitNumber}</span>
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex flex-col gap-1">
                      <span className="flex items-center gap-1 text-sm">
                        <Mail className="h-3 w-3 text-muted-foreground" />
                        {resident.email}
                      </span>
                      <span className="flex items-center gap-1 text-sm text-muted-foreground">
                        <Phone className="h-3 w-3" />
                        {resident.phone}
                      </span>
                    </div>
                  </td>
                  <td className="px-4 py-3">
                    <div className="flex items-center gap-2">
                      <StatusIcon
                        className={cn(
                          "h-4 w-4",
                          resident.credentialStatus === "active" && "text-success",
                          resident.credentialStatus === "suspended" && "text-warning",
                          resident.credentialStatus === "pending" && "text-muted-foreground",
                          resident.credentialStatus === "revoked" && "text-error"
                        )}
                      />
                      <Badge variant={statusVariants[resident.credentialStatus]}>
                        {resident.credentialStatus}
                      </Badge>
                    </div>
                  </td>
                  <td className="px-4 py-3 text-sm text-muted-foreground">
                    {resident.lastAccess
                      ? formatDate(resident.lastAccess)
                      : "Never"}
                  </td>
                  <td className="px-4 py-3 text-right">
                    <div className="flex items-center justify-end gap-1">
                      <Button variant="ghost" size="icon" asChild>
                        <Link href={`/residents/${resident.id}` as `/residents/${string}`}>
                          <Eye className="h-4 w-4" />
                        </Link>
                      </Button>
                      <Button variant="ghost" size="icon">
                        <MoreHorizontal className="h-4 w-4" />
                      </Button>
                    </div>
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </Card>
  );
}
