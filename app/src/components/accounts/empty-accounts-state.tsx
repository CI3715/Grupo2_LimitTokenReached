export function EmptyAccountsState({
  onCreateAccount,
}: {
  onCreateAccount: () => void;
}) {
  return (
    <div className="empty-accounts">
      <span aria-hidden="true">＋</span>
      <h3>Todavía no tienes cuentas</h3>
      <p>
        Crea tu primera cuenta para comenzar a organizar tu dinero por moneda.
      </p>
      <button className="account-primary" onClick={onCreateAccount}>
        Crear mi primera cuenta
      </button>
    </div>
  );
}
