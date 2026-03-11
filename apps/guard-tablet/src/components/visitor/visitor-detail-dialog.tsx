"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import {
  User,
  Hash,
  Target,
  Home,
  Building2,
  CreditCard,
  Clock,
  Timer,
  AlertTriangle,
} from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useVisitorStore, type Visitor } from "@/lib/stores/visitor-store";
import { useOverrideStore } from "@/lib/stores/override-store";
import { useOfflineStore } from "@/lib/stores/offline-store";
import { OverrideDialog } from "@/components/override/override-dialog";

interface VisitorDetailDialogProps {
  visitor: Visitor | null;
  onClose: () => void;
}

export function VisitorDetailDialog({
  visitor,
  onClose,
}: VisitorDetailDialogProps) {
  const t = useTranslations();
  const { updateVisitorStatus } = useVisitorStore();
  const { isOnline, addToQueue } = useOfflineStore();
  const { setOverrideTarget } = useOverrideStore();
  const [isVerifying, setIsVerifying] = useState(false);
  const [isDenying, setIsDenying] = useState(false);
  const [showOverride, setShowOverride] = useState(false);

  if (!visitor) return null;

  const handleVerify = async () => {
    setIsVerifying(true);
    try {
      if (isOnline) {
        await updateVisitorStatus(visitor.id, "verified");
      } else {
        addToQueue({
          type: "verify",
          visitorId: visitor.id,
          timestamp: Date.now(),
        });
      }
      onClose();
    } finally {
      setIsVerifying(false);
    }
  };

  const handleDeny = async () => {
    setIsDenying(true);
    try {
      if (isOnline) {
        await updateVisitorStatus(visitor.id, "denied");
      } else {
        addToQueue({
          type: "deny",
          visitorId: visitor.id,
          timestamp: Date.now(),
        });
      }
      onClose();
    } finally {
      setIsDenying(false);
    }
  };

  const handleOverride = () => {
    setOverrideTarget(visitor);
    setShowOverride(true);
  };

  const formatDateTime = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleString([], {
      dateStyle: "medium",
      timeStyle: "short",
    });
  };

  const DetailRow = ({
    icon: Icon,
    label,
    value,
  }: {
    icon: React.ComponentType<{ className?: string }>;
    label: string;
    value: string;
  }) => (
    <div className="flex items-center gap-3 py-2">
      <Icon className="h-5 w-5 text-muted-foreground" />
      <div className="flex-1">
        <p className="text-xs text-muted-foreground">{label}</p>
        <p className="font-medium">{value}</p>
      </div>
    </div>
  );

  return (
    <>
      <Dialog open={!!visitor} onOpenChange={() => onClose()}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center justify-between">
              <span>{t("visitor.details")}</span>
              <Badge
                variant={
                  visitor.status === "pending"
                    ? "warning"
                    : visitor.status === "verified"
                    ? "success"
                    : "error"
                }
              >
                {t(`visitor.status.${visitor.status}`)}
              </Badge>
            </DialogTitle>
            <DialogDescription className="sr-only">
              {t("visitor.details")} for {visitor.name}
            </DialogDescription>
          </DialogHeader>

          <div className="space-y-1">
            <DetailRow icon={User} label={t("visitor.name")} value={visitor.name} />
            <DetailRow
              icon={Hash}
              label={t("visitor.idHash")}
              value={visitor.idHash}
            />
            <DetailRow
              icon={Target}
              label={t("visitor.purpose")}
              value={visitor.purpose}
            />
            <DetailRow
              icon={Home}
              label={t("visitor.host")}
              value={visitor.hostResident}
            />
            <DetailRow
              icon={Building2}
              label={t("visitor.unit")}
              value={visitor.hostUnit}
            />
            <DetailRow
              icon={CreditCard}
              label={t("visitor.credential")}
              value={t(`visitor.credentialType.${visitor.credentialType}`)}
            />
            <DetailRow
              icon={Clock}
              label={t("visitor.arrived")}
              value={formatDateTime(visitor.arrivedAt)}
            />
            <DetailRow
              icon={Timer}
              label={t("visitor.expires")}
              value={
                visitor.expiresAt
                  ? formatDateTime(visitor.expiresAt)
                  : t("visitor.noExpiry")
              }
            />
          </div>

          {visitor.status === "pending" && (
            <div className="flex flex-col gap-3 pt-4">
              <div className="flex gap-3">
                <Button
                  variant="success"
                  size="lg"
                  className="flex-1"
                  onClick={handleVerify}
                  disabled={isVerifying || isDenying}
                >
                  {isVerifying ? t("visitor.verifying") : t("visitor.verify")}
                </Button>
                <Button
                  variant="destructive"
                  size="lg"
                  className="flex-1"
                  onClick={handleDeny}
                  disabled={isVerifying || isDenying}
                >
                  {isDenying ? t("visitor.denying") : t("visitor.deny")}
                </Button>
              </div>
              <Button
                variant="outline"
                size="lg"
                onClick={handleOverride}
                className="w-full"
              >
                <AlertTriangle className="mr-2 h-4 w-4" />
                {t("override.title")}
              </Button>
            </div>
          )}

          {visitor.status !== "pending" && (
            <div className="pt-4">
              <Button variant="outline" className="w-full" onClick={onClose}>
                {t("common.close")}
              </Button>
            </div>
          )}
        </DialogContent>
      </Dialog>

      <OverrideDialog
        open={showOverride}
        onClose={() => {
          setShowOverride(false);
          onClose();
        }}
      />
    </>
  );
}
