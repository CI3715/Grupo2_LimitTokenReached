import type { Account, Currency } from "@/types/accounts";
import { AccountCard } from "./account-card";
import { EmptyAccountsState } from "./empty-accounts-state";

export function AccountsList({
  accounts,
  currencies,
  onCreateAccount,
}: {
  accounts: Account[];
  currencies: Currency[];
  onCreateAccount: () => void;
}) {
  return (
    <section className="accounts-list" aria-labelledby="accounts-list-title">
      <div className="section-heading">
        <div>
          <h2 id="accounts-list-title">Tus cuentas</h2>
          <p>
            {accounts.length} {accounts.length === 1 ? "cuenta" : "cuentas"}
          </p>
        </div>
        <span className="list-labels">
          <span>Moneda</span>
          <span>Saldo</span>
        </span>
      </div>
      {accounts.length ? (
        <ul>
          {accounts.map((account) => (
            <AccountCard
              key={account.id}
              account={account}
              currency={currencies.find(
                (currency) => currency.codigo === account.moneda_codigo,
              )}
            />
          ))}
        </ul>
      ) : (
        <EmptyAccountsState onCreateAccount={onCreateAccount} />
      )}
    </section>
  );
}
