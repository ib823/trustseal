import { getTranslations } from "next-intl/server";

import { StatsCards } from "@/components/dashboard/stats-cards";
import { RecentActivity } from "@/components/dashboard/recent-activity";
import { AccessChart } from "@/components/dashboard/access-chart";
import { VerifierStatus } from "@/components/dashboard/verifier-status";

export async function generateMetadata() {
  const t = await getTranslations("dashboard");
  return {
    title: t("title"),
  };
}

export default function DashboardPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Dashboard</h1>
        <p className="text-muted-foreground">
          Overview of your property access control
        </p>
      </div>

      <StatsCards />

      <div className="grid gap-6 lg:grid-cols-7">
        <div className="lg:col-span-4">
          <AccessChart />
        </div>
        <div className="lg:col-span-3">
          <VerifierStatus />
        </div>
      </div>

      <RecentActivity />
    </div>
  );
}
