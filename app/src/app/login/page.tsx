"use client";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { useAuthSession } from "@/features/auth/useAuthSession";
import { LoginForm } from "@/features/auth/LoginForm";

export default function LoginPage() {
  const { usuario } = useAuthSession();

  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar active="auth" sesionActiva={Boolean(usuario)} />
        <main className="accounts-workspace">
          <LoginForm />
        </main>
      </div>
    </div>
  );
}
