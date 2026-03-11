"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { AlertTriangle, Fingerprint } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useOverrideStore } from "@/lib/stores/override-store";
import { useVisitorStore } from "@/lib/stores/visitor-store";
import { useOfflineStore } from "@/lib/stores/offline-store";
import { requestBiometricAuth } from "@/hooks/use-override";

interface OverrideDialogProps {
  open: boolean;
  onClose: () => void;
}

const OVERRIDE_REASONS = [
  "medical",
  "fire",
  "security",
  "system",
  "other",
] as const;

export function OverrideDialog({ open, onClose }: OverrideDialogProps) {
  const t = useTranslations();
  const { overrideTarget, clearOverrideTarget } = useOverrideStore();
  const { updateVisitorStatus } = useVisitorStore();
  const { isOnline, addToQueue } = useOfflineStore();
  const [reason, setReason] = useState<string>("");
  const [customReason, setCustomReason] = useState("");
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleClose = () => {
    setReason("");
    setCustomReason("");
    setError(null);
    clearOverrideTarget();
    onClose();
  };

  const handleOverride = async () => {
    if (!overrideTarget) return;

    const finalReason = reason === "other" ? customReason : reason;
    if (!finalReason) {
      setError(t("override.reasonRequired"));
      return;
    }

    setIsAuthenticating(true);
    setError(null);

    try {
      const authToken = await requestBiometricAuth();
      if (!authToken) {
        setError(t("override.authFailed"));
        return;
      }

      if (isOnline) {
        await updateVisitorStatus(overrideTarget.id, "verified");
      } else {
        addToQueue({
          type: "override",
          visitorId: overrideTarget.id,
          timestamp: Date.now(),
          reason: finalReason,
        });
      }

      handleClose();
    } catch {
      setError(t("override.failed"));
    } finally {
      setIsAuthenticating(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-warning">
            <AlertTriangle className="h-5 w-5" />
            {t("override.title")}
          </DialogTitle>
          <DialogDescription>{t("override.description")}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 pt-2">
          <div className="rounded-md bg-warning/10 p-3">
            <p className="text-sm text-warning">{t("override.warning")}</p>
          </div>

          <div className="space-y-2">
            <label className="text-sm font-medium">{t("override.reason")}</label>
            <Select value={reason} onValueChange={setReason}>
              <SelectTrigger>
                <SelectValue placeholder={t("override.reasonPlaceholder")} />
              </SelectTrigger>
              <SelectContent>
                {OVERRIDE_REASONS.map((r) => (
                  <SelectItem key={r} value={r}>
                    {t(`override.reasons.${r}`)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {reason === "other" && (
            <div className="space-y-2">
              <textarea
                placeholder={t("override.reasonPlaceholder")}
                value={customReason}
                onChange={(e) => setCustomReason(e.target.value)}
                className="h-24 w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2"
              />
            </div>
          )}

          {error && (
            <p className="text-sm text-error">{error}</p>
          )}

          <div className="flex gap-3 pt-2">
            <Button variant="outline" className="flex-1" onClick={handleClose}>
              {t("common.cancel")}
            </Button>
            <Button
              variant="warning"
              className="flex-1"
              onClick={handleOverride}
              disabled={isAuthenticating || !reason}
            >
              <Fingerprint className="mr-2 h-4 w-4" />
              {isAuthenticating
                ? t("override.authenticating")
                : t("override.authenticate")}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
