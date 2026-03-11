"use client";

import { useTranslations } from "next-intl";
import { useEffect, useState } from "react";
import Image from "next/image";
import { CheckCircle, Download, Share2, Clock, MapPin } from "lucide-react";
import QRCode from "qrcode";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { useRegistrationStore } from "@/lib/stores/registration-store";
import { formatDateTime } from "@/lib/utils";

export function CompletionScreen() {
  const t = useTranslations();
  const { credential, invite } = useRegistrationStore();
  const [qrCodeUrl, setQrCodeUrl] = useState<string>("");

  useEffect(() => {
    if (credential?.qrCodeData) {
      QRCode.toDataURL(credential.qrCodeData, {
        width: 256,
        margin: 2,
        color: {
          dark: "#000000",
          light: "#ffffff",
        },
      }).then(setQrCodeUrl);
    }
  }, [credential?.qrCodeData]);

  const handleDownloadQR = async () => {
    if (!qrCodeUrl) return;

    const link = document.createElement("a");
    link.download = "visitor-pass-qr.png";
    link.href = qrCodeUrl;
    link.click();
  };

  const handleShare = async () => {
    if (navigator.share && credential) {
      try {
        await navigator.share({
          title: t("complete.passReady"),
          text: t("complete.shareText", { propertyName: invite?.propertyName ?? "" }),
          url: credential.deepLink,
        });
      } catch {
        // User cancelled or share failed
      }
    }
  };

  const handleOpenWallet = () => {
    if (credential?.deepLink) {
      window.location.href = credential.deepLink;
    }
  };

  if (!credential) {
    return null;
  }

  return (
    <div className="space-y-6 px-4">
      {/* Success header */}
      <div className="text-center">
        <div className="w-16 h-16 bg-success/10 rounded-full flex items-center justify-center mx-auto mb-4">
          <CheckCircle className="h-8 w-8 text-success" />
        </div>
        <h1 className="text-2xl font-bold">{t("complete.title")}</h1>
        <p className="text-muted-foreground mt-1">{t("complete.description")}</p>
      </div>

      {/* QR Code Card */}
      <Card className="max-w-sm mx-auto">
        <CardHeader className="text-center pb-2">
          <CardTitle className="text-lg">{t("complete.passReady")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* QR Code */}
          <div className="qr-container mx-auto w-fit">
            {qrCodeUrl ? (
              <Image
                src={qrCodeUrl}
                alt={t("complete.qrAlt")}
                width={256}
                height={256}
                className="w-64 h-64"
                unoptimized
              />
            ) : (
              <div className="w-64 h-64 bg-muted animate-pulse rounded" />
            )}
          </div>

          <p className="text-sm text-muted-foreground text-center">
            {t("complete.qrHint")}
          </p>

          {/* Actions */}
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              className="flex-1"
              onClick={handleDownloadQR}
            >
              <Download className="h-4 w-4 mr-2" />
              {t("complete.saveQr")}
            </Button>
            {"share" in navigator && (
              <Button
                variant="outline"
                size="sm"
                className="flex-1"
                onClick={handleShare}
              >
                <Share2 className="h-4 w-4 mr-2" />
                {t("complete.sharePass")}
              </Button>
            )}
          </div>
        </CardContent>
      </Card>

      {/* Pass Details */}
      <Card className="max-w-sm mx-auto">
        <CardHeader className="pb-2">
          <CardTitle className="text-base">{t("complete.passDetails")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex items-center gap-3">
            <Clock className="h-4 w-4 text-muted-foreground" />
            <div className="text-sm">
              <p className="text-muted-foreground">{t("complete.validFrom")}</p>
              <p className="font-medium">{formatDateTime(credential.validFrom)}</p>
            </div>
          </div>
          <div className="flex items-center gap-3">
            <Clock className="h-4 w-4 text-muted-foreground" />
            <div className="text-sm">
              <p className="text-muted-foreground">{t("complete.validUntil")}</p>
              <p className="font-medium">{formatDateTime(credential.validUntil)}</p>
            </div>
          </div>
          <div className="flex items-start gap-3">
            <MapPin className="h-4 w-4 text-muted-foreground mt-0.5" />
            <div className="text-sm">
              <p className="text-muted-foreground">{t("complete.accessPoints")}</p>
              <p className="font-medium">{t("complete.mainEntrance")}</p>
              <p className="font-medium">{t("complete.visitorParking")}</p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Download Wallet CTA */}
      <Card className="max-w-sm mx-auto bg-primary/5 border-primary/20">
        <CardContent className="pt-6 text-center space-y-3">
          <p className="text-sm font-medium">{t("complete.downloadWallet")}</p>
          <p className="text-xs text-muted-foreground">
            {t("complete.downloadWalletHint")}
          </p>
          <Button onClick={handleOpenWallet} className="w-full">
            {t("complete.appStoreButton")}
          </Button>
        </CardContent>
      </Card>

      {/* Done button */}
      <div className="max-w-sm mx-auto">
        <Button
          variant="outline"
          size="lg"
          className="w-full"
          onClick={() => window.close()}
        >
          {t("complete.done")}
        </Button>
      </div>
    </div>
  );
}
