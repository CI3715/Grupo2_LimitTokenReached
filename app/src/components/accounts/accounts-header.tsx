export function AccountsHeader({
  onCreateAccount,
  disabled,
}: {
  onCreateAccount: () => void;
  disabled: boolean;
}) {
  return (
    <header className="accounts-header">
      <div>
        <p className="breadcrumb">
          Inicio <span>/</span> Mis cuentas
        </p>
        <h1>Mis cuentas</h1>
        <p>Todo lo que tienes, organizado por moneda.</p>
      </div>
      <button
        className="account-primary"
        onClick={onCreateAccount}
        disabled={disabled}
      >
        <span aria-hidden="true">＋</span> Nueva cuenta
      </button>
    </header>
  );
}
