import { useTranslations } from "next-intl";
import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { VerifierGrid } from "@/components/verifiers/verifier-grid";
import { VerifierGridSkeleton } from "@/components/verifiers/verifier-grid-skeleton";
import { AddVerifierDialog } from "@/components/verifiers/add-verifier-dialog";

export async function generateMetadata() {
  const t = await getTranslations("verifiers");
  return {
    title: t("title"),
  };
}

export default function VerifiersPage() {
  const t = useTranslations("verifiers");

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">{t("title")}</h1>
          <p className="text-muted-foreground">{t("description")}</p>
        </div>
        <AddVerifierDialog />
      </div>

      <Suspense fallback={<VerifierGridSkeleton />}>
        <VerifierGrid />
      </Suspense>
    </div>
  );
}
