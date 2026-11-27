"use client";

import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { BackupWorkspace } from "@/components/backup/BackupWorkspace";
import { useBackup } from "@/features/backup/useBackup";
import { useAuthSession } from "@/features/auth/useAuthSession";
import "./respaldo.css";

export default function BackupPage() {
  const { usuario } = useAuthSession();
  const backup = useBackup();

  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar active="backup" sesionActiva={Boolean(usuario)} />
        <main className="accounts-workspace">
          <header className="accounts-header">
            <div>
              <p className="breadcrumb">
                Sistema <span>/</span> Respaldo
              </p>
              <h1>Respaldo</h1>
              <p>
                Guarda copias de seguridad y sincroniza tus cambios locales en otros dispositivos
              </p>
            </div>
          </header>
          <BackupWorkspace backup={backup} />
        </main>
      </div>
    </div>
  );
}
