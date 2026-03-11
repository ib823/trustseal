"use client";

import { useEffect, useState } from "react";
import { useParams } from "next/navigation";
import { useTranslations } from "next-intl";
import { AlertCircle } from "lucide-react";
import { Header } from "@/components/header";
import { Footer } from "@/components/footer";
import { StepIndicator } from "@/components/step-indicator";
import { InviteCard } from "@/components/invite-card";
import { RegistrationForm } from "@/components/registration-form";
import { VerificationStep } from "@/components/verification-step";
import { CompletionScreen } from "@/components/completion-screen";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { useRegistrationStore } from "@/lib/stores/registration-store";
import { invitesApi } from "@/lib/api/invites";
import { hashIdNumber } from "@/lib/utils";

const TOTAL_STEPS = 3;

export default function InvitePage() {
  const params = useParams();
  const inviteId = params.inviteId as string;
  const t = useTranslations();

  const {
    currentStep,
    invite,
    registration,
    setInvite,
    setStep,
    nextStep,
    prevStep,
    setCredential,
    setSubmitting,
    setError,
    isSubmitting,
    error,
  } = useRegistrationStore();

  const [isLoading, setIsLoading] = useState(true);
  const [inviteError, setInviteError] = useState<string | null>(null);

  // Load invite on mount
  useEffect(() => {
    async function loadInvite() {
      try {
        setIsLoading(true);
        const response = await invitesApi.get(inviteId);

        if (!response.isValid) {
          if (response.isExpired) {
            setInviteError(t("error.inviteExpired"));
          } else {
            setInviteError(t("error.inviteNotFound"));
          }
          return;
        }

        setInvite(response.invite);
      } catch {
        setInviteError(t("error.inviteNotFound"));
      } finally {
        setIsLoading(false);
      }
    }

    loadInvite();
  }, [inviteId, setInvite, t]);

  // Handle step 0 -> 1 transition (Get Started)
  const handleGetStarted = () => {
    setStep(1);
  };

  // Handle form submission and credential issuance
  const handleSubmitRegistration = async () => {
    setSubmitting(true);
    setError(null);

    try {
      const response = await invitesApi.register({
        inviteId,
        fullName: registration.fullName,
        idType: registration.idType!,
        idHash: hashIdNumber(registration.idNumber),
        purpose: registration.purpose === "other"
          ? registration.purposeOther
          : registration.purpose!,
        vehicleNumber: registration.vehicleNumber || undefined,
        livenessVerified: registration.livenessVerified,
      });

      if (response.success) {
        setCredential(response.credential);
        nextStep();
      } else {
        setError(t("error.registrationFailed"));
      }
    } catch {
      setError(t("error.registrationFailed"));
    } finally {
      setSubmitting(false);
    }
  };

  // Loading state
  if (isLoading) {
    return (
      <div className="min-h-screen flex flex-col">
        <Header />
        <main className="flex-1 flex items-center justify-center">
          <div className="animate-pulse text-muted-foreground">
            {t("common.loading")}
          </div>
        </main>
        <Footer />
      </div>
    );
  }

  // Error state
  if (inviteError) {
    return (
      <div className="min-h-screen flex flex-col">
        <Header />
        <main className="flex-1 flex items-center justify-center p-4">
          <Card className="max-w-sm w-full">
            <CardContent className="pt-6 text-center space-y-4">
              <div className="w-16 h-16 bg-error/10 rounded-full flex items-center justify-center mx-auto">
                <AlertCircle className="h-8 w-8 text-error" />
              </div>
              <h2 className="text-xl font-bold">{t("invite.invalid")}</h2>
              <p className="text-muted-foreground">{inviteError}</p>
              <p className="text-sm text-muted-foreground">
                {t("invite.invalidDescription")}
              </p>
              <Button
                variant="outline"
                onClick={() => window.history.back()}
                className="w-full"
              >
                {t("common.back")}
              </Button>
            </CardContent>
          </Card>
        </main>
        <Footer />
      </div>
    );
  }

  // Step 0: Invite welcome (before starting)
  if (currentStep === 0 || (currentStep === 1 && !invite)) {
    return (
      <div className="min-h-screen flex flex-col">
        <Header />
        <main className="flex-1 flex items-center justify-center py-8">
          {invite && (
            <InviteCard invite={invite} onGetStarted={handleGetStarted} />
          )}
        </main>
        <Footer />
      </div>
    );
  }

  return (
    <div className="min-h-screen flex flex-col">
      <Header />
      <main className="flex-1 py-6">
        <div className="container mx-auto px-4 max-w-lg">
          {/* Step indicator (only for steps 1-3) */}
          {currentStep >= 1 && currentStep <= TOTAL_STEPS && (
            <div className="mb-8">
              <StepIndicator currentStep={currentStep} totalSteps={TOTAL_STEPS} />
            </div>
          )}

          {/* Error display */}
          {error && (
            <div className="mb-6 p-4 bg-error/10 border border-error/20 rounded-lg text-error text-sm text-center">
              {error}
            </div>
          )}

          {/* Step content */}
          {currentStep === 1 && (
            <RegistrationForm
              onNext={nextStep}
              onBack={() => setStep(0)}
            />
          )}

          {currentStep === 2 && (
            <VerificationStep
              onNext={handleSubmitRegistration}
              onBack={prevStep}
            />
          )}

          {currentStep === 3 && <CompletionScreen />}

          {/* Loading overlay */}
          {isSubmitting && (
            <div className="fixed inset-0 bg-background/80 flex items-center justify-center z-50">
              <div className="text-center">
                <div className="h-8 w-8 rounded-full border-4 border-primary border-t-transparent animate-spin mx-auto" />
                <p className="mt-4 text-sm text-muted-foreground">
                  {t("common.loading")}
                </p>
              </div>
            </div>
          )}
        </div>
      </main>
      <Footer />
    </div>
  );
}
