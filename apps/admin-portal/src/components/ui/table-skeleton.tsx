import { cn } from "@/lib/utils";

interface TableSkeletonProps {
  rows?: number;
  columns?: number;
}

export function TableSkeleton({ rows = 5, columns = 5 }: TableSkeletonProps) {
  return (
    <div className="rounded-lg border bg-card">
      <div className="border-b">
        <div className="flex h-12 items-center gap-4 px-4">
          {Array.from({ length: columns }).map((_, i) => (
            <div
              key={i}
              className={cn(
                "h-4 animate-pulse rounded bg-muted",
                i === 0 ? "w-32" : "w-24"
              )}
            />
          ))}
        </div>
      </div>
      {Array.from({ length: rows }).map((_, rowIndex) => (
        <div
          key={rowIndex}
          className="flex h-14 items-center gap-4 border-b px-4 last:border-0"
        >
          {Array.from({ length: columns }).map((_, colIndex) => (
            <div
              key={colIndex}
              className={cn(
                "h-4 animate-pulse rounded bg-muted",
                colIndex === 0 ? "w-40" : "w-20"
              )}
            />
          ))}
        </div>
      ))}
    </div>
  );
}
