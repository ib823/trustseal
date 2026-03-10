"use client";

import { useState } from "react";
import { useTranslations } from "next-intl";
import { Check, ChevronDown, Building2 } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { useWorkspaceStore, type Workspace } from "@/lib/stores/workspace-store";
import { cn } from "@/lib/utils";

// Mock workspaces - will be replaced with API call
const mockWorkspaces: Workspace[] = [
  {
    id: "ws_01HQ123ABC",
    name: "Sunway Geo Residences",
    code: "SGR",
    address: "Jalan Lagoon Selatan, Bandar Sunway",
    totalUnits: 1284,
    createdAt: "2024-01-01T00:00:00Z",
  },
  {
    id: "ws_01HQ456DEF",
    name: "The Maple Residences",
    code: "TMR",
    address: "Jalan Ampang, Kuala Lumpur",
    totalUnits: 856,
    createdAt: "2024-02-01T00:00:00Z",
  },
  {
    id: "ws_01HQ789GHI",
    name: "Vista Heights",
    code: "VH",
    address: "Jalan Sultan Ismail, Kuala Lumpur",
    totalUnits: 542,
    createdAt: "2024-03-01T00:00:00Z",
  },
];

export function WorkspaceSwitcher() {
  const t = useTranslations("common");
  const [open, setOpen] = useState(false);
  const { currentWorkspace, setCurrentWorkspace } = useWorkspaceStore();

  // Initialize with first workspace if none selected
  const activeWorkspace = currentWorkspace ?? mockWorkspaces[0];

  const handleSelect = (workspace: Workspace) => {
    setCurrentWorkspace(workspace);
    setOpen(false);
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button
          variant="outline"
          className="w-full justify-between text-left font-normal"
        >
          <div className="flex flex-col items-start">
            <span className="text-xs text-muted-foreground">{t("workspace")}</span>
            <span className="font-medium truncate max-w-[180px]">
              {activeWorkspace.name}
            </span>
          </div>
          <ChevronDown className="h-4 w-4 opacity-50" />
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>{t("workspace")}</DialogTitle>
        </DialogHeader>
        <div className="space-y-2 pt-4">
          {mockWorkspaces.map((workspace) => (
            <button
              key={workspace.id}
              onClick={() => handleSelect(workspace)}
              className={cn(
                "flex w-full items-center gap-3 rounded-lg border p-3 text-left transition-colors hover:bg-muted",
                workspace.id === activeWorkspace.id && "border-primary bg-primary/5"
              )}
            >
              <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
                <Building2 className="h-5 w-5 text-primary" />
              </div>
              <div className="flex-1 min-w-0">
                <p className="font-medium truncate">{workspace.name}</p>
                <p className="text-sm text-muted-foreground truncate">
                  {workspace.totalUnits} units
                </p>
              </div>
              {workspace.id === activeWorkspace.id && (
                <Check className="h-4 w-4 text-primary" />
              )}
            </button>
          ))}
        </div>
      </DialogContent>
    </Dialog>
  );
}
