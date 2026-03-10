import { getTranslations } from "next-intl/server";

import { SettingsTabs } from "@/components/settings/settings-tabs";

export async function generateMetadata() {
  const t = await getTranslations("settings");
  return {
    title: t("title"),
  };
}

export default function SettingsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">
          Configure property and access control settings
        </p>
      </div>

      <SettingsTabs />
    </div>
  );
}
