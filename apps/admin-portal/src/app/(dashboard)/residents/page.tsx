import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { ResidentTable } from "@/components/residents/resident-table";
import { ResidentTableSkeleton } from "@/components/residents/resident-table-skeleton";
import { AddResidentButton } from "@/components/residents/add-resident-button";

export async function generateMetadata() {
  const t = await getTranslations("residents");
  return {
    title: t("title"),
  };
}

export default function ResidentsPage() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Residents</h1>
          <p className="text-muted-foreground">
            Manage resident credentials and access
          </p>
        </div>
        <AddResidentButton />
      </div>

      <Suspense fallback={<ResidentTableSkeleton />}>
        <ResidentTable />
      </Suspense>
    </div>
  );
}
