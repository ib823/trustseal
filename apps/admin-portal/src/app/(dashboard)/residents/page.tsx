import { useTranslations } from "next-intl";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { ResidentTable } from "@/components/residents/resident-table";
import { ResidentTableSkeleton } from "@/components/residents/resident-table-skeleton";
import { AddResidentDialog } from "@/components/residents/add-resident-dialog";

export async function generateMetadata() {
  const t = await getTranslations("residents");
  return {
    title: t("title"),
  };
}

export default function ResidentsPage() {
  const t = useTranslations("residents");

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{t("title")}</h1>
          <p className="text-muted-foreground">{t("description")}</p>
        </div>
        <AddResidentDialog />
      </div>

      <Suspense fallback={<ResidentTableSkeleton />}>
        <ResidentTable />
      </Suspense>
    </div>
  );
}
