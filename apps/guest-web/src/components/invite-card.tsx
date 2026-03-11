"use client";

import { useTranslations } from "next-intl";
import { Building2, User, Calendar, Clock } from "lucide-react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import type { InviteData } from "@/lib/stores/registration-store";
import { formatDate, formatRelativeTime } from "@/lib/utils";

interface InviteCardProps {
  invite: InviteData;
  onGetStarted: () => void;
}

export function InviteCard({ invite, onGetStarted }: InviteCardProps) {
  const t = useTranslations();

  const isExpired = new Date(invite.expiresAt) < new Date();
  const expiresIn = formatRelativeTime(invite.expiresAt);

  return (
    <div className="flex flex-col items-center gap-6 px-4">
      <div className="text-center">
        <h1 className="text-2xl font-bold">{t("invite.welcome")}</h1>
        <p className="text-muted-foreground mt-1">
          {t("invite.youAreInvited")}
        </p>
      </div>

      <Card className="w-full max-w-sm">
        <CardContent className="pt-6 space-y-4">
          <div className="flex items-start gap-3">
            <Building2 className="h-5 w-5 text-muted-foreground shrink-0 mt-0.5" />
            <div>
              <p className="text-xs text-muted-foreground">
                {t("invite.propertyName")}
              </p>
              <p className="font-medium">{invite.propertyName}</p>
            </div>
          </div>

          <div className="flex items-start gap-3">
            <User className="h-5 w-5 text-muted-foreground shrink-0 mt-0.5" />
            <div>
              <p className="text-xs text-muted-foreground">
                {t("invite.hostName")}
              </p>
              <p className="font-medium">
                {invite.hostName} ({invite.hostUnit})
              </p>
            </div>
          </div>

          <div className="flex items-start gap-3">
            <Calendar className="h-5 w-5 text-muted-foreground shrink-0 mt-0.5" />
            <div>
              <p className="text-xs text-muted-foreground">
                {t("invite.visitDate")}
              </p>
              <p className="font-medium">{formatDate(invite.visitDate)}</p>
            </div>
          </div>

          {!isExpired && (
            <div className="flex items-center gap-2 text-sm text-muted-foreground pt-2 border-t">
              <Clock className="h-4 w-4" />
              <span>{t("invite.expiresIn", { time: expiresIn })}</span>
            </div>
          )}
        </CardContent>
      </Card>

      {isExpired ? (
        <div className="text-center">
          <p className="text-error font-medium">{t("invite.expired")}</p>
        </div>
      ) : (
        <Button size="lg" className="w-full max-w-sm" onClick={onGetStarted}>
          {t("invite.getStarted")}
        </Button>
      )}
    </div>
  );
}
