import type { Account, Currency } from "@/types/accounts";
import { minorUnits } from "@/types/account-money";

export function AccountsSummary({
  accounts,
  currencies,
}: {
  accounts: Account[];
  currencies: Currency[];
}) {
  const entries = currencies.filter((currency) =>
    accounts.some((account) => account.moneda_codigo === currency.codigo),
  );
  return (
    <section className="accounts-summary" aria-labelledby="summary-title">
      <div className="section-heading">
        <div>
          <h2 id="summary-title">Resumen</h2>
          <p>Los saldos se mantienen separados por moneda.</p>
        </div>
        <span className="local-tag">Disponible</span>
      </div>
      <div className="summary-grid">
        {entries.map((currency) => {
          const matching = accounts.filter(
            (account) => account.moneda_codigo === currency.codigo,
          );
          const total = matching.reduce(
            (sum, account) => sum.plus(minorUnits(account.saldo)),
            minorUnits("0"),
          );
          return (
            <article className="summary-card" key={currency.codigo}>
              <div>
                <span>{currency.nombre}</span>
                <strong>{currency.codigo}</strong>
              </div>
              <p>
                <span className="summary-money">
                  {currency.simbolo} {total.toString()}
                </span>
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
