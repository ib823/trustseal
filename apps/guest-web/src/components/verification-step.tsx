"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";
import { Camera, CheckCircle, XCircle, SkipForward } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { useRegistrationStore } from "@/lib/stores/registration-store";
import { cn } from "@/lib/utils";

interface VerificationStepProps {
  onNext: () => void;
  onBack: () => void;
}

type VerificationStatus = "idle" | "processing" | "success" | "failed";

export function VerificationStep({ onNext, onBack }: VerificationStepProps) {
  const t = useTranslations();
  const { updateRegistration } = useRegistrationStore();
  const [status, setStatus] = useState<VerificationStatus>("idle");

  const handleStartVerification = async () => {
    setStatus("processing");

    // Simulate liveness check - in production this would use a real biometric SDK
    await new Promise((resolve) => setTimeout(resolve, 2000));

    // Simulate 80% success rate
    const success = Math.random() > 0.2;

    if (success) {
      setStatus("success");
      updateRegistration({ livenessVerified: true });
      // Auto-proceed after success
      setTimeout(() => {
        onNext();
      }, 1500);
    } else {
      setStatus("failed");
    }
  };

  const handleSkip = () => {
    updateRegistration({ livenessVerified: false });
    onNext();
  };

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardHeader>
        <CardTitle>{t("verify.title")}</CardTitle>
        <CardDescription>{t("verify.description")}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Verification area */}
        <div
          className={cn(
            "aspect-square max-w-[280px] mx-auto rounded-2xl border-2 border-dashed flex flex-col items-center justify-center gap-4 transition-colors",
            status === "idle" && "border-muted-foreground/30 bg-muted/30",
            status === "processing" && "border-primary bg-primary/5",
            status === "success" && "border-success bg-success/5",
            status === "failed" && "border-error bg-error/5"
          )}
        >
          {status === "idle" && (
            <>
              <Camera className="h-16 w-16 text-muted-foreground/50" />
              <p className="text-sm text-muted-foreground text-center px-4">
                {t("verify.livenessDescription")}
              </p>
            </>
          )}

          {status === "processing" && (
            <>
              <div className="h-16 w-16 rounded-full border-4 border-primary border-t-transparent animate-spin" />
              <p className="text-sm text-primary font-medium">
                {t("verify.processing")}
              </p>
            </>
          )}

          {status === "success" && (
            <>
              <CheckCircle className="h-16 w-16 text-success" />
              <p className="text-sm text-success font-medium">
                {t("verify.livenessSuccess")}
              </p>
            </>
          )}

          {status === "failed" && (
            <>
              <XCircle className="h-16 w-16 text-error" />
              <p className="text-sm text-error font-medium text-center px-4">
                {t("verify.livenessFailed")}
              </p>
            </>
          )}
        </div>

        {/* Hint */}
        <p className="text-xs text-muted-foreground text-center">
          {t("verify.livenessHint")}
        </p>

        {/* Actions */}
        <div className="space-y-3">
          {(status === "idle" || status === "failed") && (
            <Button
              size="lg"
              className="w-full"
              onClick={handleStartVerification}
            >
              <Camera className="mr-2 h-5 w-5" />
              {t("verify.livenessButton")}
            </Button>
          )}

          {status === "processing" && (
            <Button size="lg" className="w-full" disabled>
              {t("verify.processing")}
            </Button>
          )}

          <div className="flex gap-3">
            <Button
              type="button"
              variant="outline"
              onClick={onBack}
              className="flex-1"
              disabled={status === "processing"}
            >
              {t("common.back")}
            </Button>
            <Button
              type="button"
              variant="ghost"
              onClick={handleSkip}
              className="flex-1"
              disabled={status === "processing"}
            >
              <SkipForward className="mr-2 h-4 w-4" />
              {t("verify.skip")}
            </Button>
          </div>
        </div>

        {/* Skip hint */}
        <p className="text-xs text-muted-foreground text-center">
          {t("verify.skipHint")}
        </p>
      </CardContent>
    </Card>
  );
}
