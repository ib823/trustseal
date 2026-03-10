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
  ChevronDown,
} from "lucide-react";

import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

const navigation = [
  { name: "Dashboard", href: "/" as const, icon: Home },
  { name: "Residents", href: "/residents" as const, icon: Users },
  { name: "Access Logs", href: "/access-logs" as const, icon: ClipboardList },
  { name: "Verifiers", href: "/verifiers" as const, icon: Cpu },
  { name: "Analytics", href: "/analytics" as const, icon: BarChart3 },
  { name: "Settings", href: "/settings" as const, icon: Settings },
] as const;

export function Sidebar() {
  const pathname = usePathname();

  return (
    <aside className="flex w-64 flex-col border-r bg-card">
      {/* Logo */}
      <div className="flex h-16 items-center gap-2 border-b px-6">
        <Shield className="h-6 w-6 text-primary" />
        <span className="text-lg font-semibold">VaultPass</span>
      </div>

      {/* Workspace Selector */}
      <div className="border-b p-4">
        <Button
          variant="outline"
          className="w-full justify-between text-left font-normal"
        >
          <div className="flex flex-col items-start">
            <span className="text-xs text-muted-foreground">Workspace</span>
            <span className="font-medium">Sunway Geo Residences</span>
          </div>
          <ChevronDown className="h-4 w-4 opacity-50" />
        </Button>
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 p-4">
        {navigation.map((item) => {
          const isActive =
            pathname === item.href ||
            (item.href !== "/" && pathname.startsWith(item.href));

          return (
            <Link
              key={item.name}
              href={item.href}
              className={cn(
                "flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-primary/10 text-primary"
                  : "text-muted-foreground hover:bg-muted hover:text-foreground"
              )}
            >
              <item.icon className="h-4 w-4" />
              {item.name}
            </Link>
          );
        })}
      </nav>

      {/* Footer */}
      <div className="border-t p-4">
        <div className="flex items-center gap-3 rounded-lg bg-muted/50 px-3 py-2">
          <div className="flex h-8 w-8 items-center justify-center rounded-full bg-primary/10 text-xs font-medium text-primary">
            SG
          </div>
          <div className="flex-1 overflow-hidden">
            <p className="truncate text-sm font-medium">Sunway Geo</p>
            <p className="truncate text-xs text-muted-foreground">
              Active since Jan 2024
            </p>
          </div>
        </div>
      </div>
    </aside>
  );
}
