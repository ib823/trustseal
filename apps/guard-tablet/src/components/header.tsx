"use client";

import { useTranslations } from "next-intl";
import { RefreshCw, Shield, Wifi, WifiOff } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useVisitorStore } from "@/lib/stores/visitor-store";
import { useOfflineStore } from "@/lib/stores/offline-store";
import { locales, localeNames, type Locale } from "@/i18n/config";
import { cn } from "@/lib/utils";

export function Header() {
  const t = useTranslations();
  const { lastFetch, isLoading, fetchVisitors } = useVisitorStore();
  const { isOnline } = useOfflineStore();

  const formatLastSync = () => {
    if (!lastFetch) return t("header.never");
    const seconds = Math.floor((Date.now() - lastFetch) / 1000);
    if (seconds < 60) return t("header.secondsAgo", { seconds });
    const minutes = Math.floor(seconds / 60);
    return t("header.minutesAgo", { minutes });
  };

  const handleRefresh = () => {
    fetchVisitors();
  };

  const handleLocaleChange = (locale: string) => {
    document.cookie = `locale=${locale};path=/;max-age=31536000`;
    window.location.reload();
  };

  return (
    <header className="flex h-16 items-center justify-between border-b bg-card px-6">
      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2">
          <Shield className="h-8 w-8 text-primary" />
          <div>
            <h1 className="text-lg font-semibold">{t("header.title")}</h1>
            <p className="text-xs text-muted-foreground">
              {t("header.checkpoint")} A1
            </p>
          </div>
        </div>
      </div>

      <div className="flex items-center gap-4">
        <div className="flex items-center gap-2 text-sm">
          {isOnline ? (
            <Wifi className="h-4 w-4 text-success" />
          ) : (
            <WifiOff className="h-4 w-4 text-error" />
          )}
          <span
            className={cn(
              "font-medium",
              isOnline ? "text-success" : "text-error"
            )}
          >
            {isOnline ? t("common.online") : t("common.offline")}
          </span>
        </div>

        <div className="h-6 w-px bg-border" />

        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <span>{t("header.lastSync")}:</span>
          <span className="font-medium text-foreground">{formatLastSync()}</span>
        </div>

        <Button
          variant="outline"
          size="icon"
          onClick={handleRefresh}
          disabled={isLoading || !isOnline}
        >
          <RefreshCw className={cn("h-4 w-4", isLoading && "animate-spin")} />
        </Button>

        <div className="h-6 w-px bg-border" />

        <Select
          defaultValue="en"
          onValueChange={handleLocaleChange}
        >
          <SelectTrigger className="w-[140px]">
            <SelectValue placeholder={t("header.language")} />
          </SelectTrigger>
          <SelectContent>
            {locales.map((locale) => (
              <SelectItem key={locale} value={locale}>
                {localeNames[locale as Locale]}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </header>
  );
}
