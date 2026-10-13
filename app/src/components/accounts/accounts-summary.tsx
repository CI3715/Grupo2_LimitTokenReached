import type { Account, Currency } from "@/types/accounts";
import { formatMoney, minorUnits } from "@/types/account-money";

export function AccountsSummary({
  accounts,
  currencies,
}: {
  accounts: Account[];
  currencies: Currency[];
}) {
  const entries = currencies.filter((currency) =>
    accounts.some((account) => account.moneda === currency.codigo),
  );
  return (
    <section className="accounts-summary" aria-labelledby="summary-title">
      <div className="section-heading">
        <div>
          <h2 id="summary-title">Resumen</h2>
          <p>Los saldos se mantienen separados por moneda.</p>
        </div>
        <span className="local-tag">Local</span>
      </div>
      <div className="summary-grid">
        {entries.map((currency) => {
          const matching = accounts.filter(
            (account) => account.moneda === currency.codigo,
          );
          const total = matching.reduce(
            (sum, account) => sum + minorUnits(account.saldo),
            BigInt(0),
          );
          return (
            <article className="summary-card" key={currency.codigo}>
              <div>
                <span>{currency.nombre}</span>
                <strong>{currency.codigo}</strong>
              </div>
              <p>
                {currency.simbolo} {formatMoney(total)}
              </p>
              <small>
                {matching.length} {matching.length === 1 ? "cuenta" : "cuentas"}
              </small>
            </article>
          );
        })}
      </div>
    </section>
  );
}
