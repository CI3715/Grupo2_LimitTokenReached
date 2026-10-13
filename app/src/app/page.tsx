"use client";
import { useState } from "react";
import { AccountsHeader } from "@/components/accounts/accounts-header";
import { AccountsList } from "@/components/accounts/accounts-list";
import { AccountsSidebar } from "@/components/accounts/accounts-sidebar";
import { AccountsSummary } from "@/components/accounts/accounts-summary";
import { NewAccountModal } from "@/components/accounts/new-account-modal";
import { useAccounts } from "@/features/accounts/useAccounts";
import "./accounts.css";
export default function HomePage() {
  const { accounts, currencies, loading, error, preview, reload, create } =
    useAccounts();
  const [createOpen, setCreateOpen] = useState(false);
  const [announcement, setAnnouncement] = useState("");
  return (
    <div className="accounts-page">
      <div className="accounts-shell">
        <AccountsSidebar />
        <main className="accounts-workspace">
          <AccountsHeader
            onCreateAccount={() => setCreateOpen(true)}
            disabled={loading || !!error || !currencies.length}
          />
          {preview && (
            <p className="preview-notice">
              Vista previa · Datos de muestra. Las cuentas creadas aquí se
              pierden al recargar.
            </p>
          )}
          <span className="sr-only" role="status">
            {announcement}
          </span>
          {loading ? (
            <div className="accounts-feedback" role="status">
              Cargando tus cuentas…
            </div>
          ) : error ? (
            <div className="accounts-feedback">
              <p className="account-error" role="alert">
                {error}
              </p>
              <button
                className="account-secondary"
                onClick={() => void reload()}
              >
                Reintentar
              </button>
            </div>
          ) : (
            <>
              <AccountsSummary accounts={accounts} currencies={currencies} />
              <AccountsList
                accounts={accounts}
                currencies={currencies}
                onCreateAccount={() => setCreateOpen(true)}
              />
              <p className="accounts-footnote">
                Tú tienes el control de tus cuentas.
              </p>
            </>
          )}
        </main>
      </div>
      {createOpen && (
        <NewAccountModal
          currencies={currencies}
          preview={preview}
          onClose={() => setCreateOpen(false)}
          onCreate={async (input) => {
            const result = await create(input);
            if (!result) setAnnouncement(`Cuenta ${input.nombre} creada.`);
            return result;
          }}
        />
      )}
    </div>
  );
}
