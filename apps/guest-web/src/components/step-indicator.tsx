"use client";

import { useTranslations } from "next-intl";
import { Check } from "lucide-react";
import { cn } from "@/lib/utils";

interface StepIndicatorProps {
  currentStep: number;
  totalSteps: number;
}

export function StepIndicator({ currentStep, totalSteps }: StepIndicatorProps) {
  const t = useTranslations();

  const steps = [
    { key: 1, label: t("steps.info") },
    { key: 2, label: t("steps.verify") },
    { key: 3, label: t("steps.complete") },
  ];

  return (
    <div className="w-full">
      {/* Progress bar */}
      <div className="relative mb-4">
        <div className="absolute top-4 left-0 right-0 h-0.5 bg-muted" />
        <div
          className="absolute top-4 left-0 h-0.5 bg-primary transition-all duration-300"
          style={{ width: `${((currentStep - 1) / (totalSteps - 1)) * 100}%` }}
        />

        {/* Step circles */}
        <div className="relative flex justify-between">
          {steps.map((step) => (
            <div key={step.key} className="flex flex-col items-center">
              <div
                className={cn(
                  "w-8 h-8 rounded-full flex items-center justify-center text-sm font-medium transition-colors",
                  step.key < currentStep &&
                    "bg-success text-success-foreground",
                  step.key === currentStep &&
                    "bg-primary text-primary-foreground",
                  step.key > currentStep && "bg-muted text-muted-foreground"
                )}
              >
                {step.key < currentStep ? (
                  <Check className="h-4 w-4" />
                ) : (
                  step.key
                )}
              </div>
              <span
                className={cn(
                  "mt-2 text-xs text-center max-w-[80px]",
                  step.key === currentStep
                    ? "text-foreground font-medium"
                    : "text-muted-foreground"
                )}
              >
                {step.label}
              </span>
            </div>
          ))}
        </div>
      </div>

      {/* Step counter */}
      <p className="text-center text-sm text-muted-foreground">
        {t("steps.step", { current: currentStep, total: totalSteps })}
      </p>
    </div>
  );
}
