import { useTranslations } from "next-intl";
import { getTranslations } from "next-intl/server";

import { SettingsTabs } from "@/components/settings/settings-tabs";

export async function generateMetadata() {
  const t = await getTranslations("settings");
  return {
    title: t("title"),
  };
}

export default function SettingsPage() {
  const t = useTranslations("settings");

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">{t("title")}</h1>
        <p className="text-muted-foreground">{t("description")}</p>
      </div>

      <SettingsTabs />
    </div>
  );
}
