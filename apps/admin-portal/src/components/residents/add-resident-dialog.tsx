"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { Plus, Loader2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";

interface AddResidentFormData {
  name: string;
  email: string;
  phone: string;
  unitNumber: string;
}

export function AddResidentDialog() {
  const t = useTranslations("residents");
  const tCommon = useTranslations("common");

  const [open, setOpen] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [formData, setFormData] = useState<AddResidentFormData>({
    name: "",
    email: "",
    phone: "",
    unitNumber: "",
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);

    try {
      // TODO: Call API to create resident
      // await createResident(workspaceId, formData);

      // Simulate API call
      await new Promise((resolve) => setTimeout(resolve, 1000));

      setOpen(false);
      setFormData({ name: "", email: "", phone: "", unitNumber: "" });
    } catch (error) {
      console.error("Failed to create resident:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleChange = (field: keyof AddResidentFormData) => (
    e: React.ChangeEvent<HTMLInputElement>
  ) => {
    setFormData((prev) => ({ ...prev, [field]: e.target.value }));
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          {t("addResident")}
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{t("addResident")}</DialogTitle>
            <DialogDescription>
              {t("addResidentDescription")}
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="name">{t("name")}</Label>
              <Input
                id="name"
                value={formData.name}
                onChange={handleChange("name")}
                placeholder={t("namePlaceholder")}
                required
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="email">{t("email")}</Label>
              <Input
                id="email"
                type="email"
                value={formData.email}
                onChange={handleChange("email")}
                placeholder={t("emailPlaceholder")}
                required
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="phone">{t("phone")}</Label>
              <Input
                id="phone"
                type="tel"
                value={formData.phone}
                onChange={handleChange("phone")}
                placeholder={t("phonePlaceholder")}
                required
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="unitNumber">{t("unit")}</Label>
              <Input
                id="unitNumber"
                value={formData.unitNumber}
                onChange={handleChange("unitNumber")}
                placeholder={t("unitPlaceholder")}
                required
              />
            </div>
          </div>
          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => setOpen(false)}
              disabled={isSubmitting}
            >
              {tCommon("cancel")}
            </Button>
            <Button type="submit" disabled={isSubmitting}>
              {isSubmitting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              {tCommon("save")}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
