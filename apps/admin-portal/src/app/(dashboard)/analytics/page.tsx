import { useTranslations } from "next-intl";
import { getTranslations } from "next-intl/server";

import { AnalyticsOverview } from "@/components/analytics/analytics-overview";
import { PeakHoursChart } from "@/components/analytics/peak-hours-chart";
import { DenialReasons } from "@/components/analytics/denial-reasons";
import { EntryTrends } from "@/components/analytics/entry-trends";

export async function generateMetadata() {
  const t = await getTranslations("analytics");
  return {
    title: t("title"),
  };
}

export default function AnalyticsPage() {
  const t = useTranslations("analytics");

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">{t("title")}</h1>
        <p className="text-muted-foreground">{t("description")}</p>
      </div>

      <AnalyticsOverview />

      <div className="grid gap-6 lg:grid-cols-2">
        <PeakHoursChart />
        <DenialReasons />
      </div>

      <EntryTrends />
    </div>
  );
}
