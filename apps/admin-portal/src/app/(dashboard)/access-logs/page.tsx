import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { AccessLogTable } from "@/components/access-logs/access-log-table";
import { AccessLogFilters } from "@/components/access-logs/access-log-filters";
import { TableSkeleton } from "@/components/ui/table-skeleton";

export async function generateMetadata() {
  const t = await getTranslations("accessLogs");
  return {
    title: t("title"),
  };
}

export default function AccessLogsPage() {
  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold tracking-tight">Access Logs</h1>
        <p className="text-muted-foreground">
          View and analyze entry and exit events
        </p>
      </div>

      <AccessLogFilters />

      <Suspense fallback={<TableSkeleton rows={10} />}>
        <AccessLogTable />
      </Suspense>
    </div>
  );
}
