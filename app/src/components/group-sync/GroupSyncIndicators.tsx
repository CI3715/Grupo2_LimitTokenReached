import styles from "./GroupSyncIndicators.module.css";

export type PendingGroupExpense = {
  id: string;
  grupo: string;
  concepto: string;
  fecha: string;
  monto: string;
  moneda: string;
};

export type QueuedGroupExpense = {
  id: string;
  concepto: string;
  monto: string;
  moneda: string;
  estado: "pendiente" | "rechazado";
  motivo?: string;
};

export function PendingGroupExpenses({ items }: { items: PendingGroupExpense[] }) {
  if (!items.length) return null;
  return (
    <section className={styles.panel} aria-label="Gastos pendientes de aplicar">
      <h3>Gastos pendientes de aplicar</h3>
      <p>Elige una cuenta para cada grupo para aplicar estos gastos.</p>
      <ul className={styles.list}>
        {items.map((item) => (
          <li key={item.id} className={styles.item}>
            <div><strong>{item.grupo} · {item.concepto}</strong><small>{item.fecha}</small></div>
            <strong>{item.monto} {item.moneda}</strong>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function QueuedGroupExpenses({ items }: { items: QueuedGroupExpense[] }) {
  if (!items.length) return null;
  return (
    <section className={styles.panel} aria-label="Gastos pendientes de sincronización">
      <h3>Gastos pendientes de sincronización</h3>
      <ul className={styles.list}>
        {items.map((item) => (
          <li key={item.id} className={styles.item}>
            <div>
              <strong>{item.concepto}</strong>
              <small className={item.estado === "rechazado" ? styles.error : styles.pending}>
                {item.estado === "rechazado" ? "Rechazado al sincronizar" : "Pendiente de envío"}
              </small>
              {item.estado === "rechazado" && item.motivo && <small role="alert" className={styles.error}>{item.motivo}</small>}
            </div>
            <strong>{item.monto} {item.moneda}</strong>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function GroupMovementLabel() {
  return <small className={styles.badge}>Gasto de grupo</small>;
}

export type EligibleAccount = { id: string; nombre: string; moneda: string; activa: boolean };

export function GroupAccountSelector({
  cuentas, moneda, seleccion, guardando, onSelect,
}: {
  cuentas: EligibleAccount[];
  moneda: string;
  seleccion: string;
  guardando: boolean;
  onSelect: (id: string) => void;
}) {
  const elegibles = cuentas.filter((cuenta) => cuenta.activa && cuenta.moneda === moneda);
  return (
    <section className={styles.panel} aria-label="Cuenta personal del grupo">
      <h3>Cuenta personal del grupo</h3>
      <p>Elige una cuenta activa en {moneda} para aplicar tus gastos.</p>
      <label className={styles.control}>
        Cuenta
        <select value={seleccion} disabled={guardando || elegibles.length === 0} onChange={(event) => onSelect(event.target.value)}>
          <option value="">Selecciona una cuenta</option>
          {elegibles.map((cuenta) => <option key={cuenta.id} value={cuenta.id}>{cuenta.nombre}</option>)}
        </select>
      </label>
      {!elegibles.length && <p>No tienes cuentas activas en {moneda}.</p>}
    </section>
  );
}

export function SyncButton({ ultimaSincronizacion, sincronizando, sesionVencida, onSync }: {
  ultimaSincronizacion: string | null;
  sincronizando: boolean;
  sesionVencida: boolean;
  onSync: () => void;
}) {
  return (
    <section className={styles.panel} aria-label="Sincronización">
      <h3>Sincronización</h3>
      <p>{ultimaSincronizacion ? `Última sincronización: ${ultimaSincronizacion}` : "Aún no se ha sincronizado."}</p>
      {sesionVencida && <p className={styles.error} role="alert">Tu sesión venció, inicia sesión de nuevo.</p>}
      <button className={styles.button} type="button" disabled={sincronizando || sesionVencida} onClick={onSync}>
        {sincronizando ? "Sincronizando..." : "Sincronizar"}
      </button>
    </section>
  );
}
