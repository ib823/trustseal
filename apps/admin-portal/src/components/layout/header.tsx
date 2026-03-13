"use client";

import { Bell, Search, Moon, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { useTranslations } from "next-intl";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { useUserStore } from "@/lib/stores/user-store";

export function Header() {
  const { theme, setTheme } = useTheme();
  const t = useTranslations("common");
  const tResidents = useTranslations("residents");
  const user = useUserStore((state) => state.user);

  return (
    <header className="flex h-16 items-center justify-between border-b bg-card px-6">
      {/* Search */}
      <div className="relative w-full max-w-md">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          type="search"
          placeholder={tResidents("searchPlaceholder")}
          className="pl-9"
        />
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2">
        {/* Theme Toggle */}
        <Button
          variant="ghost"
          size="icon"
          onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
          className="text-muted-foreground"
        >
          <Sun className="h-4 w-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
          <Moon className="absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
          <span className="sr-only">{t("toggleTheme")}</span>
        </Button>

        {/* Notifications */}
        <Button variant="ghost" size="icon" className="relative text-muted-foreground">
          <Bell className="h-4 w-4" />
          <span className="absolute right-1.5 top-1.5 h-2 w-2 rounded-full bg-error" />
          <span className="sr-only">{t("notifications")}</span>
        </Button>

        {/* User Menu */}
        <Button variant="ghost" className="gap-2 px-2">
          <Avatar className="h-8 w-8">
            <AvatarImage src="/avatar.png" alt={t("profileAvatar")} />
            <AvatarFallback>AD</AvatarFallback>
          </Avatar>
          <div className="hidden flex-col items-start text-left md:flex">
            <span className="text-sm font-medium">{user?.name ?? ""}</span>
            <span className="text-xs text-muted-foreground">{user?.role ?? ""}</span>
          </div>
        </Button>
      </div>
    </header>
  );
}
