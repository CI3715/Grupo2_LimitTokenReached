export function AccountsHeader({
  onCreateAccount,
  onCreateCategory,
  disabled,
}: {
  onCreateAccount: () => void;
  onCreateCategory: () => void;
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
      <div className="accounts-header-actions">
        <button
          type="button"
          className="account-secondary"
          onClick={onCreateCategory}
        >
          <span aria-hidden="true">＋</span> Nueva categoría
        </button>
        <button
          className="account-primary"
          onClick={onCreateAccount}
          disabled={disabled}
        >
          <span aria-hidden="true">＋</span> Nueva cuenta
        </button>
      </div>
    </header>
  );
}
