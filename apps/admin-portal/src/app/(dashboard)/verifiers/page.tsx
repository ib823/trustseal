import { getTranslations } from "next-intl/server";
import { Suspense } from "react";

import { VerifierGrid } from "@/components/verifiers/verifier-grid";
import { VerifierGridSkeleton } from "@/components/verifiers/verifier-grid-skeleton";
import { AddVerifierButton } from "@/components/verifiers/add-verifier-button";

export async function generateMetadata() {
  const t = await getTranslations("verifiers");
  return {
    title: t("title"),
  };
}

export default function VerifiersPage() {
  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Verifiers</h1>
          <p className="text-muted-foreground">
            Monitor and configure edge verifier devices
          </p>
        </div>
        <AddVerifierButton />
      </div>

      <Suspense fallback={<VerifierGridSkeleton />}>
        <VerifierGrid />
      </Suspense>
    </div>
  );
}
