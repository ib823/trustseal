"use client";

import { useTranslations } from "next-intl";
import { Globe } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { locales, localeNames, type Locale } from "@/i18n/config";

export function Header() {
  const t = useTranslations();

  const handleLocaleChange = (locale: string) => {
    document.cookie = `locale=${locale};path=/;max-age=31536000`;
    window.location.reload();
  };

  return (
    <header className="sticky top-0 z-10 bg-background border-b">
      <div className="container mx-auto px-4 h-14 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <div className="w-8 h-8 bg-primary rounded-lg flex items-center justify-center">
            <span className="text-primary-foreground font-bold text-sm">VP</span>
          </div>
          <span className="font-semibold text-sm hidden sm:inline">
            {t("header.title")}
          </span>
        </div>

        <Select defaultValue="en" onValueChange={handleLocaleChange}>
          <SelectTrigger className="w-[140px] h-10">
            <Globe className="h-4 w-4 mr-2 opacity-50" />
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
