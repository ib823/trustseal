"use client";

import { useTranslations } from "next-intl";
import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { useRegistrationStore, type IdType, type VisitPurpose } from "@/lib/stores/registration-store";
import { validateMalaysianIC, validatePassport } from "@/lib/utils";
import { cn } from "@/lib/utils";

interface RegistrationFormProps {
  onNext: () => void;
  onBack: () => void;
}

export function RegistrationForm({ onNext, onBack }: RegistrationFormProps) {
  const t = useTranslations();
  const { registration, updateRegistration } = useRegistrationStore();
  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!registration.fullName.trim()) {
      newErrors.fullName = t("form.validation.nameRequired");
    } else if (registration.fullName.trim().length < 2) {
      newErrors.fullName = t("form.validation.nameMinLength");
    }

    if (!registration.idType) {
      newErrors.idType = t("form.validation.idTypeRequired");
    }

    if (!registration.idNumber.trim()) {
      newErrors.idNumber = t("form.validation.idNumberRequired");
    } else if (registration.idType === "ic" && !validateMalaysianIC(registration.idNumber)) {
      newErrors.idNumber = t("form.validation.idNumberInvalid");
    } else if (registration.idType === "passport" && !validatePassport(registration.idNumber)) {
      newErrors.idNumber = t("form.validation.idNumberInvalid");
    }

    if (!registration.purpose) {
      newErrors.purpose = t("form.validation.purposeRequired");
    }

    if (registration.purpose === "other" && !registration.purposeOther.trim()) {
      newErrors.purposeOther = t("form.validation.purposeOtherRequired");
    }

    if (!registration.termsAgreed) {
      newErrors.terms = t("form.termsRequired");
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (validate()) {
      onNext();
    }
  };

  const purposeOptions: { value: VisitPurpose; label: string }[] = [
    { value: "social", label: t("form.purposeOptions.social") },
    { value: "delivery", label: t("form.purposeOptions.delivery") },
    { value: "service", label: t("form.purposeOptions.service") },
    { value: "business", label: t("form.purposeOptions.business") },
    { value: "other", label: t("form.purposeOptions.other") },
  ];

  return (
    <Card className="w-full max-w-md mx-auto">
      <CardHeader>
        <CardTitle>{t("form.title")}</CardTitle>
        <CardDescription>{t("form.description")}</CardDescription>
      </CardHeader>
      <CardContent>
        <form onSubmit={handleSubmit} className="space-y-5">
          {/* Full Name */}
          <div className="space-y-2">
            <Label htmlFor="fullName">{t("form.fullName")} *</Label>
            <Input
              id="fullName"
              value={registration.fullName}
              onChange={(e) => updateRegistration({ fullName: e.target.value })}
              placeholder={t("form.fullNamePlaceholder")}
              className={cn(errors.fullName && "border-error")}
            />
            {errors.fullName && (
              <p className="text-sm text-error">{errors.fullName}</p>
            )}
          </div>

          {/* ID Type */}
          <div className="space-y-2">
            <Label htmlFor="idType">{t("form.idType")} *</Label>
            <Select
              value={registration.idType ?? ""}
              onValueChange={(v) => updateRegistration({ idType: v as IdType })}
            >
              <SelectTrigger className={cn(errors.idType && "border-error")}>
                <SelectValue placeholder={t("form.idType")} />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="ic">{t("form.idTypeIc")}</SelectItem>
                <SelectItem value="passport">{t("form.idTypePassport")}</SelectItem>
              </SelectContent>
            </Select>
            {errors.idType && (
              <p className="text-sm text-error">{errors.idType}</p>
            )}
          </div>

          {/* ID Number */}
          <div className="space-y-2">
            <Label htmlFor="idNumber">{t("form.idNumber")} *</Label>
            <Input
              id="idNumber"
              value={registration.idNumber}
              onChange={(e) => updateRegistration({ idNumber: e.target.value })}
              placeholder={t("form.idNumberPlaceholder")}
              className={cn(errors.idNumber && "border-error")}
            />
            <p className="text-xs text-muted-foreground">{t("form.idNumberHint")}</p>
            {errors.idNumber && (
              <p className="text-sm text-error">{errors.idNumber}</p>
            )}
          </div>

          {/* Purpose */}
          <div className="space-y-2">
            <Label htmlFor="purpose">{t("form.purpose")} *</Label>
            <Select
              value={registration.purpose ?? ""}
              onValueChange={(v) => updateRegistration({ purpose: v as VisitPurpose })}
            >
              <SelectTrigger className={cn(errors.purpose && "border-error")}>
                <SelectValue placeholder={t("form.purposePlaceholder")} />
              </SelectTrigger>
              <SelectContent>
                {purposeOptions.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            {errors.purpose && (
              <p className="text-sm text-error">{errors.purpose}</p>
            )}
          </div>

          {/* Other Purpose */}
          {registration.purpose === "other" && (
            <div className="space-y-2">
              <Label htmlFor="purposeOther">{t("form.purposeOther")} *</Label>
              <Input
                id="purposeOther"
                value={registration.purposeOther}
                onChange={(e) => updateRegistration({ purposeOther: e.target.value })}
                placeholder={t("form.purposeOtherPlaceholder")}
                className={cn(errors.purposeOther && "border-error")}
              />
              {errors.purposeOther && (
                <p className="text-sm text-error">{errors.purposeOther}</p>
              )}
            </div>
          )}

          {/* Vehicle Number */}
          <div className="space-y-2">
            <Label htmlFor="vehicleNumber">
              {t("form.vehicleNumber")} <span className="text-muted-foreground">({t("common.optional")})</span>
            </Label>
            <Input
              id="vehicleNumber"
              value={registration.vehicleNumber}
              onChange={(e) => updateRegistration({ vehicleNumber: e.target.value.toUpperCase() })}
              placeholder={t("form.vehicleNumberPlaceholder")}
            />
            <p className="text-xs text-muted-foreground">{t("form.vehicleNumberHint")}</p>
          </div>

          {/* Terms Agreement */}
          <div className="space-y-2">
            <label className="flex items-start gap-3 cursor-pointer">
              <input
                type="checkbox"
                checked={registration.termsAgreed}
                onChange={(e) => updateRegistration({ termsAgreed: e.target.checked })}
                className="mt-1 h-4 w-4 rounded border-input"
              />
              <span className="text-sm">{t("form.termsAgree")}</span>
            </label>
            {errors.terms && (
              <p className="text-sm text-error">{errors.terms}</p>
            )}
          </div>

          {/* Actions */}
          <div className="flex gap-3 pt-4">
            <Button type="button" variant="outline" onClick={onBack} className="flex-1">
              {t("common.back")}
            </Button>
            <Button type="submit" className="flex-1">
              {t("common.next")}
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}
