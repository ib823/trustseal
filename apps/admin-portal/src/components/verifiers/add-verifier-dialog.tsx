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

interface AddVerifierFormData {
  name: string;
  location: string;
  model: string;
}

export function AddVerifierDialog() {
  const t = useTranslations("verifiers");
  const tCommon = useTranslations("common");

  const [open, setOpen] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [formData, setFormData] = useState<AddVerifierFormData>({
    name: "",
    location: "",
    model: "VaultPass Edge V2",
  });

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);

    try {
      // TODO: Call API to create verifier
      // await createVerifier(workspaceId, formData);

      // Simulate API call
      await new Promise((resolve) => setTimeout(resolve, 1000));

      setOpen(false);
      setFormData({ name: "", location: "", model: "VaultPass Edge V2" });
    } catch (error) {
      console.error("Failed to create verifier:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleChange = (field: keyof AddVerifierFormData) => (
    e: React.ChangeEvent<HTMLInputElement>
  ) => {
    setFormData((prev) => ({ ...prev, [field]: e.target.value }));
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          {t("addVerifier")}
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <form onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>{t("addVerifier")}</DialogTitle>
            <DialogDescription>
              {t("addVerifierDescription")}
            </DialogDescription>
          </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="name">{t("deviceName")}</Label>
              <Input
                id="name"
                value={formData.name}
                onChange={handleChange("name")}
                placeholder={t("deviceNamePlaceholder")}
                required
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="location">{t("location")}</Label>
              <Input
                id="location"
                value={formData.location}
                onChange={handleChange("location")}
                placeholder={t("locationPlaceholder")}
                required
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor="model">{t("model")}</Label>
              <Input
                id="model"
                value={formData.model}
                onChange={handleChange("model")}
                disabled
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
