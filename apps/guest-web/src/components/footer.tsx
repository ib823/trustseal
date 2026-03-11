"use client";

import { useTranslations } from "next-intl";

export function Footer() {
  const t = useTranslations();

  return (
    <footer className="mt-auto py-6 border-t">
      <div className="container mx-auto px-4">
        <div className="flex flex-col sm:flex-row items-center justify-center gap-4 text-sm text-muted-foreground">
          <a href="#privacy" className="hover:text-foreground transition-colors">
            {t("footer.privacy")}
          </a>
          <span className="hidden sm:inline">|</span>
          <a href="#terms" className="hover:text-foreground transition-colors">
            {t("footer.terms")}
          </a>
          <span className="hidden sm:inline">|</span>
          <a href="#help" className="hover:text-foreground transition-colors">
            {t("footer.help")}
          </a>
        </div>
        <p className="text-center text-xs text-muted-foreground mt-4">
          {t("header.poweredBy")}
        </p>
      </div>
    </footer>
  );
}
