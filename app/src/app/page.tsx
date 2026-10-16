"use client";
import type { ErrorValidacionDTO } from "@/types/error_validacion";
import { useState } from "react";
import { AccountsHeader } from "@/components/accounts/accounts-header";
import { AccountsList } from "@/components/accounts/accounts-list";
import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { AccountsSummary } from "@/components/accounts/accounts-summary";
import { NewAccountModal } from "@/components/accounts/new-account-modal";
import { useAccounts } from "@/features/accounts/useAccounts";
import "./accounts.css";
export default function HomePage() {
  const { accounts, currencies, create } = useAccounts();
  const [createOpen, setCreateOpen] = useState(false);
  const [announcement, setAnnouncement] = useState("");
  const [validationError, setValidationError] =
  useState<ErrorValidacionDTO | null>(null);
  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar />
        <main className="accounts-workspace">
          <AccountsHeader
            onCreateAccount={() => setCreateOpen(true)}
            disabled={false}
          />
          <p className="accounts-guidance">
            Agrega tus cuentas y mantén tus saldos a la vista.
          </p>
          <span className="sr-only" role="status">
            {announcement}
          </span>
          <AccountsSummary accounts={accounts} currencies={currencies} />
          <AccountsList
            accounts={accounts}
            currencies={currencies}
            onCreateAccount={() => setCreateOpen(true)}
          />
          <p className="accounts-footnote">
            Tú tienes el control de tus cuentas.
          </p>
        </main>
      </div>
      {createOpen && (
        <NewAccountModal
          validationError={validationError}
          currencies={currencies}
          onClose={() => {
            setCreateOpen(false);
            setValidationError(null);
}}
          onCreate={async (input) => {
            try{
              setValidationError(null);
              await create(input);

              setAnnouncement(`Cuenta ${input.nombre} agregada a la vista previa`);
            } catch (error) {
              const validationError = error as ErrorValidacionDTO;
              setValidationError(validationError);
              throw error;
            }
            
          }}
        />
      )}
    </div>
  );
}
