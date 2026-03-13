"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  Home,
  Users,
  ClipboardList,
  Cpu,
  BarChart3,
  Settings,
  Shield,
} from "lucide-react";
import { useTranslations } from "next-intl";

import { cn } from "@/lib/utils";
import { WorkspaceSwitcher } from "./workspace-switcher";
import { useWorkspaceStore } from "@/lib/stores/workspace-store";

const navigationItems = [
  { labelKey: "dashboard.title", href: "/" as const, icon: Home },
  { labelKey: "residents.title", href: "/residents" as const, icon: Users },
  { labelKey: "accessLogs.title", href: "/access-logs" as const, icon: ClipboardList },
  { labelKey: "verifiers.title", href: "/verifiers" as const, icon: Cpu },
  { labelKey: "analytics.title", href: "/analytics" as const, icon: BarChart3 },
  { labelKey: "settings.title", href: "/settings" as const, icon: Settings },
] as const;

export function Sidebar() {
  const pathname = usePathname();
  const t = useTranslations();
  const tCommon = useTranslations("common");
  const { currentWorkspace } = useWorkspaceStore();

  // Get initials from workspace name
  const initials = currentWorkspace?.name
    .split(" ")
    .map((w) => w[0])
    .join("")
    .slice(0, 2)
    .toUpperCase() ?? "VP";

  const shortName = currentWorkspace?.name.split(" ").slice(0, 2).join(" ") ?? tCommon("productName");

  return (
    <aside className="flex w-64 flex-col border-r bg-card">
      {/* Logo */}
      <div className="flex h-16 items-center gap-2 border-b px-6">
        <Shield className="h-6 w-6 text-primary" />
        <span className="text-lg font-semibold">{tCommon("productName")}</span>
      </div>

      {/* Workspace Selector */}
      <div className="border-b p-4">
        <WorkspaceSwitcher />
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 p-4">
        {navigationItems.map((item) => {
          const isActive =
            pathname === item.href ||
            (item.href !== "/" && pathname.startsWith(item.href));

          return (
            <Link
              key={item.labelKey}
              href={item.href}
              className={cn(
                "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-primary/10 text-primary"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground"
              )}
            >
              <item.icon className="h-4 w-4" />
              {t(item.labelKey)}
            </Link>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="border-t p-4">
        <div className="flex items-center gap-3 rounded-lg bg-muted/50 px-3 py-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-xs font-medium text-primary">
            {initials}
          </div>
          <div className="flex-1 overflow-hidden">
            <p className="truncate text-sm font-medium">{shortName}</p>
            <p className="truncate text-xs text-muted-foreground">
              {currentWorkspace?.totalUnits ?? 0} {tCommon("units")}
            </p>
          </div>
        </div>
      </div>
    </aside>
  );
}
