import Link from "next/link";

export function AccountsSidebar({
  active = "accounts",
}: {
  active?: "accounts" | "connection";
}) {
  return (
    <aside className="accounts-sidebar">
      <Link href="/" className="accounts-brand">
        <span className="brand-mark" aria-hidden="true">
          C
        </span>
        <span>
          Cuentas Claras<small>finanzas locales</small>
        </span>
      </Link>
      <p className="eyebrow">Principal</p>
      <nav aria-label="Principal">
        <Link
          href="/"
          aria-current={active === "accounts" ? "page" : undefined}
          className={active === "accounts" ? "nav-current" : undefined}
        >
          <span aria-hidden="true">▦</span> Mis cuentas
        </Link>
        <button disabled>
          <span aria-hidden="true">◎</span> Grupos <small>Pronto</small>
        </button>
      </nav>
      <div className="sidebar-divider" />
      <p className="eyebrow">Sistema</p>
      <nav aria-label="Sistema">
        <button disabled>
          <span aria-hidden="true">⚙</span> Preferencias <small>Pronto</small>
        </button>
        <Link
          href="/conexion/"
          aria-current={active === "connection" ? "page" : undefined}
          className={active === "connection" ? "nav-current" : undefined}
        >
          {" "}
          <span aria-hidden="true">⌁</span> Conexión
        </Link>
      </nav>
      <div className="local-note">
        <strong>Tus cuentas, en un solo lugar</strong>
        <p>Organiza tu dinero y consulta tus saldos fácilmente.</p>
        <span>
          <i /> Tus finanzas importan
        </span>
      </div>
    </aside>
  );
}
