import { getTranslations } from "next-intl/server";
import { notFound } from "next/navigation";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { ArrowLeft, Mail, Phone, Shield, ShieldOff, Key, Ban } from "lucide-react";
import Link from "next/link";

export async function generateMetadata() {
  const t = await getTranslations("residents");
  return {
    title: t("title"),
  };
}

interface ResidentDetailPageProps {
  params: { id: string };
}

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

async function getResident(id: string): Promise<Resident | undefined> {
  // TODO: Replace with actual API call
  const residents: Resident[] = [
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
  ];

  return residents.find(r => r.id === id);
}

export default async function ResidentDetailPage({ params }: ResidentDetailPageProps) {
  const resident = await getResident(params.id);

  if (!resident) {
    notFound();
  }

  const statusVariants = {
    active: "success",
    suspended: "warning",
    pending: "secondary",
    revoked: "error",
  } as const;

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" asChild>
          <Link href="/residents">
            <ArrowLeft className="h-4 w-4" />
          </Link>
        </Button>
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{resident.name}</h1>
          <p className="text-muted-foreground">Unit {resident.unitNumber}</p>
        </div>
      </div>

      <div className="grid gap-6 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">Contact Information</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center gap-3">
              <Mail className="h-4 w-4 text-muted-foreground" />
              <span>{resident.email}</span>
            </div>
            <div className="flex items-center gap-3">
              <Phone className="h-4 w-4 text-muted-foreground" />
              <span>{resident.phone}</span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle className="text-lg">Credential Status</CardTitle>
            <Badge variant={statusVariants[resident.credentialStatus]}>
              {resident.credentialStatus}
            </Badge>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">Credential ID</span>
              <span className="font-mono text-sm">{resident.credentialId}</span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-muted-foreground">Last Access</span>
              <span>{resident.lastAccess ? new Date(resident.lastAccess).toLocaleString() : "Never"}</span>
            </div>
            <div className="flex gap-2 pt-4">
              {resident.credentialStatus === "active" && (
                <>
                  <Button variant="outline" size="sm" className="gap-2">
                    <ShieldOff className="h-4 w-4" />
                    Suspend
                  </Button>
                  <Button variant="destructive" size="sm" className="gap-2">
                    <Ban className="h-4 w-4" />
                    Revoke
                  </Button>
                </>
              )}
              {resident.credentialStatus === "suspended" && (
                <Button variant="default" size="sm" className="gap-2">
                  <Shield className="h-4 w-4" />
                  Reinstate
                </Button>
              )}
              {(resident.credentialStatus === "pending" || resident.credentialStatus === "revoked") && (
                <Button variant="default" size="sm" className="gap-2">
                  <Key className="h-4 w-4" />
                  Issue Credential
                </Button>
              )}
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
