import { Card, CardContent, CardHeader } from "@/components/ui/card";

export function VerifierGridSkeleton() {
  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      {Array.from({ length: 6 }).map((_, i) => (
        <Card key={i}>
          <CardHeader className="flex flex-row items-start justify-between pb-2">
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <div className="h-2 w-2 animate-pulse rounded-full bg-muted" />
                <div className="h-5 w-24 animate-pulse rounded bg-muted" />
              </div>
              <div className="h-4 w-36 animate-pulse rounded bg-muted" />
            </div>
            <div className="h-8 w-8 animate-pulse rounded bg-muted" />
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="h-4 w-16 animate-pulse rounded bg-muted" />
              <div className="h-5 w-14 animate-pulse rounded-full bg-muted" />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-1">
                <div className="h-3 w-12 animate-pulse rounded bg-muted" />
                <div className="h-4 w-20 animate-pulse rounded bg-muted" />
              </div>
              <div className="space-y-1">
                <div className="h-3 w-12 animate-pulse rounded bg-muted" />
                <div className="h-4 w-16 animate-pulse rounded bg-muted" />
              </div>
            </div>

            <div className="space-y-2 border-t pt-3">
              {Array.from({ length: 4 }).map((_, j) => (
                <div key={j} className="flex justify-between">
                  <div className="h-3 w-16 animate-pulse rounded bg-muted" />
                  <div className="h-3 w-24 animate-pulse rounded bg-muted" />
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
