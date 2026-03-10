"use client";

import { useState } from "react";
import { Plus } from "lucide-react";

import { Button } from "@/components/ui/button";

export function AddVerifierButton() {
  const [_isOpen, setIsOpen] = useState(false);

  // TODO: Implement modal dialog for adding verifiers
  return (
    <Button onClick={() => setIsOpen(true)}>
      <Plus className="mr-2 h-4 w-4" />
      Add Verifier
    </Button>
  );
}
