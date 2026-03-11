"use client";

import * as React from "react";
import { Check } from "lucide-react";

import { cn } from "@/lib/utils";

export interface CheckboxProps
  extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
}

const Checkbox = React.forwardRef<HTMLInputElement, CheckboxProps>(
  ({ className, label, id, ...props }, ref) => {
    const generatedId = React.useId();
    const checkboxId = id ?? generatedId;

    return (
      <div className="flex items-start gap-3">
        <div className="relative flex items-center">
          <input
            type="checkbox"
            id={checkboxId}
            ref={ref}
            className="peer sr-only"
            {...props}
          />
          <div
            className={cn(
              "h-5 w-5 shrink-0 rounded border border-input ring-offset-background",
              "peer-focus-visible:outline-none peer-focus-visible:ring-2 peer-focus-visible:ring-ring peer-focus-visible:ring-offset-2",
              "peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
              "peer-checked:bg-primary peer-checked:border-primary",
              className
            )}
          >
            <Check
              className={cn(
                "h-4 w-4 text-primary-foreground absolute top-0.5 left-0.5",
                "opacity-0 peer-checked:opacity-100 transition-opacity"
              )}
            />
          </div>
        </div>
        {label && (
          <label
            htmlFor={checkboxId}
            className="text-sm leading-relaxed cursor-pointer"
          >
            {label}
          </label>
        )}
      </div>
    );
  }
);
Checkbox.displayName = "Checkbox";

export { Checkbox };
