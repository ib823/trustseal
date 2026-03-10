import { Card } from "@/components/ui/card";

export function ResidentTableSkeleton() {
  return (
    <Card>
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="border-b">
              <th className="px-4 py-3 text-left">
                <div className="h-4 w-16 animate-pulse rounded bg-muted" />
              </th>
              <th className="px-4 py-3 text-left">
                <div className="h-4 w-12 animate-pulse rounded bg-muted" />
              </th>
              <th className="px-4 py-3 text-left">
                <div className="h-4 w-16 animate-pulse rounded bg-muted" />
              </th>
              <th className="px-4 py-3 text-left">
                <div className="h-4 w-20 animate-pulse rounded bg-muted" />
              </th>
              <th className="px-4 py-3 text-left">
                <div className="h-4 w-24 animate-pulse rounded bg-muted" />
              </th>
              <th className="px-4 py-3 text-right">
                <div className="ml-auto h-4 w-16 animate-pulse rounded bg-muted" />
              </th>
            </tr>
          </thead>
          <tbody>
            {Array.from({ length: 5 }).map((_, i) => (
              <tr key={i} className="border-b last:border-0">
                <td className="px-4 py-3">
                  <div className="flex items-center gap-3">
                    <div className="h-9 w-9 animate-pulse rounded-full bg-muted" />
                    <div className="space-y-1">
                      <div className="h-4 w-32 animate-pulse rounded bg-muted" />
                      <div className="h-3 w-20 animate-pulse rounded bg-muted" />
                    </div>
                  </div>
                </td>
                <td className="px-4 py-3">
                  <div className="h-4 w-16 animate-pulse rounded bg-muted" />
                </td>
                <td className="px-4 py-3">
                  <div className="space-y-1">
                    <div className="h-4 w-36 animate-pulse rounded bg-muted" />
                    <div className="h-3 w-28 animate-pulse rounded bg-muted" />
                  </div>
                </td>
                <td className="px-4 py-3">
                  <div className="h-5 w-16 animate-pulse rounded-full bg-muted" />
                </td>
                <td className="px-4 py-3">
                  <div className="h-4 w-24 animate-pulse rounded bg-muted" />
                </td>
                <td className="px-4 py-3 text-right">
                  <div className="ml-auto h-8 w-16 animate-pulse rounded bg-muted" />
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </Card>
  );
}
