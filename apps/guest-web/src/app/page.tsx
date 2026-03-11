import { redirect } from "next/navigation";

export default function HomePage() {
  // Root page redirects to an example invite
  // In production, this would show an error or landing page
  redirect("/invite/demo");
}
