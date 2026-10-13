import {
  accountTypeLabels,
  type Account,
  type Currency,
} from "@/types/accounts";
import { formatMoney, minorUnits } from "@/types/account-money";

export function AccountCard({
  account,
  currency,
}: {
  account: Account;
  currency?: Currency;
}) {
  return (
    <li className="account-row">
      <div className={`account-icon ${account.tipo}`} aria-hidden="true">
        {account.tipo === "efectivo"
          ? "▤"
          : account.tipo === "banco"
            ? "▥"
            : "▱"}
      </div>
      <div className="account-name">
        <h3>{account.nombre}</h3>
        <p>{accountTypeLabels[account.tipo]}</p>
      </div>
      <span className="currency-chip">{account.moneda}</span>
      <div className="account-balance">
        <strong>
          {currency?.simbolo ?? account.moneda}{" "}
          {formatMoney(minorUnits(account.saldo))}
        </strong>
        <small>Saldo actual</small>
      </div>
    </li>
  );
}
