"use client";

import { useEffect, useMemo, useState, type FormEvent } from "react";

import type {
  CrearGrupoRequestDTO,
  MiembroGrupoDTO,
  ParteGastoDTO,
  RegistrarGastoRequestDTO,
  ReglaDivisionGrupo,
  UsuarioResumenDTO,
} from "@/types/social";

type CreateGroupFormProps = {
  pending: boolean;
  onCreate: (datos: CrearGrupoRequestDTO) => Promise<boolean>;
};

export function CreateGroupForm({ pending, onCreate }: CreateGroupFormProps) {
  const [nombre, setNombre] = useState("");
  const [descripcion, setDescripcion] = useState("");
  const [moneda, setMoneda] = useState("");
  const [actividad, setActividad] = useState("");
  const [error, setError] = useState("");

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const codigo = moneda.trim().toUpperCase();
    if (!/^[A-Z]{3}$/.test(codigo)) {
      setError("La moneda debe ser un código ISO de tres letras.");
      return;
    }
    if (!nombre.trim() || !actividad.trim()) {
      setError("Completa el nombre y la actividad del grupo.");
      return;
    }
    setError("");
    const creado = await onCreate({
      nombre: nombre.trim(),
      descripcion: descripcion.trim() || null,
      moneda: codigo,
      actividad: actividad.trim(),
    });
    if (creado) {
      setNombre("");
      setDescripcion("");
      setMoneda("");
      setActividad("");
    }
  }

  return (
    <section className="group-panel" aria-labelledby="create-group-title">
      <div className="group-section-heading">
        <h2 id="create-group-title">Crear grupo</h2>
        <p>Organiza gastos compartidos con tus amigos.</p>
      </div>
      <form className="group-form" onSubmit={(event) => void submit(event)}>
        <label>
          Nombre
          <input
            required
            maxLength={120}
            value={nombre}
            onChange={(event) => setNombre(event.target.value)}
            placeholder="Viaje a Mérida"
          />
        </label>
        <label>
          Actividad o fecha
          <input
            required
            maxLength={200}
            value={actividad}
            onChange={(event) => setActividad(event.target.value)}
            placeholder="Viaje de marzo"
          />
        </label>
        <label>
          Moneda (ISO)
          <input
            required
            maxLength={3}
            value={moneda}
            onChange={(event) =>
              setMoneda(event.target.value.replace(/[^a-z]/gi, "").slice(0, 3))
            }
            placeholder="USD"
          />
        </label>
        <label>
          Descripción <span className="group-optional">(opcional)</span>
          <textarea
            maxLength={1000}
            rows={2}
            value={descripcion}
            onChange={(event) => setDescripcion(event.target.value)}
            placeholder="Detalles del viaje"
          />
        </label>
        {error && <p className="group-error" role="alert">{error}</p>}
        <button className="account-primary" type="submit" disabled={pending}>
          {pending ? "Creando…" : "Crear grupo"}
        </button>
      </form>
    </section>
  );
}

type InviteMemberFormProps = {
  amigos: UsuarioResumenDTO[];
  excluidos: string[];
  pending: boolean;
  onInvite: (nombreUsuario: string) => Promise<boolean>;
};

export function InviteMemberForm({
  amigos,
  excluidos,
  pending,
  onInvite,
}: InviteMemberFormProps) {
  const disponibles = amigos.filter(
    (amigo) => !excluidos.includes(amigo.nombre_usuario),
  );
  const [usuario, setUsuario] = useState("");
  const usuarioSeleccionado = disponibles.some(
    (amigo) => amigo.nombre_usuario === usuario,
  )
    ? usuario
    : disponibles[0]?.nombre_usuario ?? "";

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (usuarioSeleccionado && (await onInvite(usuarioSeleccionado))) {
      setUsuario("");
    }
  }

  if (disponibles.length === 0) {
    return <p className="group-muted">No tienes amigos disponibles para invitar.</p>;
  }

  return (
    <form className="group-inline-form" onSubmit={(event) => void submit(event)}>
      <label>
        Invitar a un amigo
        <select
          value={usuarioSeleccionado}
          onChange={(event) => setUsuario(event.target.value)}
          required
        >
          {disponibles.map((amigo) => (
            <option key={amigo.nombre_usuario} value={amigo.nombre_usuario}>
              {amigo.nombre_visible} (@{amigo.nombre_usuario})
            </option>
          ))}
        </select>
      </label>
      <button
        className="account-primary"
        type="submit"
        disabled={pending || !usuarioSeleccionado}
      >
        Invitar
      </button>
    </form>
  );
}

type ExpenseFormProps = {
  miembros: MiembroGrupoDTO[];
  moneda: string;
  pending: boolean;
  onSubmit: (datos: RegistrarGastoRequestDTO) => Promise<boolean>;
  onPreview: (
    datos: RegistrarGastoRequestDTO,
  ) => Promise<ParteGastoDTO[]>;
};

type SolicitudPreview = {
  solicitud: RegistrarGastoRequestDTO | null;
  error: string;
};

function minorUnits(value: string): bigint | null {
  const normalized = value.replace(",", ".");
  if (!/^\d+(?:\.\d{1,2})?$/.test(normalized)) return null;
  const [whole, fraction = ""] = normalized.split(".");
  return BigInt(whole) * BigInt(100) + BigInt(fraction.padEnd(2, "0"));
}

function percentageUnits(value: string): bigint | null {
  const normalized = value.replace(",", ".");
  if (!/^\d+(?:\.\d{1,2})?$/.test(normalized)) return null;
  const [whole, fraction = ""] = normalized.split(".");
  return BigInt(whole) * BigInt(100) + BigInt(fraction.padEnd(2, "0"));
}

function formatMinorUnits(value: bigint, trimZeros = false): string {
  const formatted = `${value / BigInt(100)}.${(value % BigInt(100))
    .toString()
    .padStart(2, "0")}`;
  return trimZeros ? formatted.replace(/\.?0+$/, "") : formatted;
}

function buildPreviewRequest(
  concepto: string,
  monto: string,
  fecha: string,
  pagador: string,
  regla: ReglaDivisionGrupo,
  participantes: string[],
  valores: Record<string, string>,
): SolicitudPreview {
  if (!concepto.trim()) return { solicitud: null, error: "Escribe el concepto." };
  const total = minorUnits(monto);
  if (total === null || total <= BigInt(0)) {
    return {
      solicitud: null,
      error: "El monto debe ser positivo y tener como máximo dos decimales.",
    };
  }
  if (!fecha) return { solicitud: null, error: "Selecciona la fecha del gasto." };
  if (!pagador) return { solicitud: null, error: "Selecciona quién pagó." };
  if (participantes.length === 0) {
    return { solicitud: null, error: "Selecciona al menos un participante." };
  }

  const valoresSeleccionados: Record<string, string> = {};
  if (regla !== "iguales") {
    let suma = BigInt(0);
    for (const usuario of participantes) {
      const entrada = valores[usuario] ?? "";
      const unidades =
        regla === "porcentajes"
          ? percentageUnits(entrada)
          : minorUnits(entrada);
      if (unidades === null) {
        return {
          solicitud: null,
          error: `Ingresa un valor no negativo, con hasta dos decimales, para ${usuario}.`,
        };
      }
      suma += unidades;
      valoresSeleccionados[usuario] = entrada.replace(",", ".");
    }

    const esperado =
      regla === "porcentajes" ? BigInt(10000) : total;
    if (suma !== esperado) {
      const diferencia = suma < esperado ? esperado - suma : suma - esperado;
      const faltanOSobran = suma < esperado ? "faltan" : "sobran";
      const totalFormateado =
        regla === "porcentajes"
          ? formatMinorUnits(suma, true)
          : formatMinorUnits(suma);
      const diferenciaFormateada =
        regla === "porcentajes"
          ? formatMinorUnits(diferencia, true)
          : formatMinorUnits(diferencia);
      return {
        solicitud: null,
        error:
          regla === "porcentajes"
            ? `Los porcentajes suman ${totalFormateado}; ${faltanOSobran} ${diferenciaFormateada}.`
            : `Los montos suman ${totalFormateado}; ${faltanOSobran} ${diferenciaFormateada}.`,
      };
    }
  }

  return {
    solicitud: {
      concepto: concepto.trim(),
      monto: monto.replace(",", "."),
      fecha,
      pagador,
      regla,
      participantes,
      valores: valoresSeleccionados,
    },
    error: "",
  };
}

function fechaLocal(): string {
  const hoy = new Date();
  return `${hoy.getFullYear()}-${String(hoy.getMonth() + 1).padStart(2, "0")}-${String(hoy.getDate()).padStart(2, "0")}`;
}

export function GroupExpenseForm({
  miembros,
  moneda,
  pending,
  onSubmit,
  onPreview,
}: ExpenseFormProps) {
  const nombres = useMemo(
    () => miembros.map((miembro) => miembro.nombre_usuario),
    [miembros],
  );
  const [concepto, setConcepto] = useState("");
  const [monto, setMonto] = useState("");
  const [fecha, setFecha] = useState(fechaLocal);
  const [pagador, setPagador] = useState(nombres[0] ?? "");
  const [regla, setRegla] = useState<ReglaDivisionGrupo>("iguales");
  const [participantes, setParticipantes] = useState(nombres);
  const [valores, setValores] = useState<Record<string, string>>({});
  const [preview, setPreview] = useState<{
    clave: string;
    partes: ParteGastoDTO[];
    error: string;
  }>({ clave: "", partes: [], error: "" });

  const resultado = useMemo(
    () =>
      buildPreviewRequest(
        concepto,
        monto,
        fecha,
        pagador,
        regla,
        participantes,
        valores,
      ),
    [concepto, monto, fecha, pagador, regla, participantes, valores],
  );
  const claveSolicitud = resultado.solicitud
    ? JSON.stringify(resultado.solicitud)
    : "";
  const previewActual =
    preview.clave === claveSolicitud ? preview : null;
  const partes = previewActual?.partes ?? [];
  const errorPreview = resultado.error || previewActual?.error || "";
  const cargandoPreview = Boolean(resultado.solicitud && !previewActual);
  const clavePreview = previewActual?.error ? "" : previewActual?.clave ?? "";

  useEffect(() => {
    let vigente = true;
    if (!resultado.solicitud) {
      return () => {
        vigente = false;
      };
    }

    void onPreview(resultado.solicitud)
      .then((partesCalculadas) => {
        if (!vigente) return;
        setPreview({
          clave: claveSolicitud,
          partes: partesCalculadas,
          error: "",
        });
      })
      .catch((cause: unknown) => {
        if (!vigente) return;
        setPreview({
          clave: claveSolicitud,
          partes: [],
          error:
            typeof cause === "string"
              ? cause
              : "No se pudo calcular la vista previa.",
        });
      });

    return () => {
      vigente = false;
    };
  }, [claveSolicitud, onPreview, resultado.solicitud]);

  function toggleParticipante(nombreUsuario: string) {
    setParticipantes((actuales) =>
      actuales.includes(nombreUsuario)
        ? actuales.filter((nombre) => nombre !== nombreUsuario)
        : [...actuales, nombreUsuario],
    );
    setValores((actuales) => ({
      ...actuales,
      [nombreUsuario]: actuales[nombreUsuario] ?? "",
    }));
  }

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (
      !resultado.solicitud ||
      clavePreview !== claveSolicitud ||
      partes.length === 0 ||
      errorPreview
    ) {
      return;
    }
    const guardado = await onSubmit(resultado.solicitud);
    if (guardado) {
      setConcepto("");
      setMonto("");
      setRegla("iguales");
      setValores({});
    }
  }

  return (
    <section className="group-panel" aria-labelledby="new-group-expense-title">
      <div className="group-section-heading">
        <h2 id="new-group-expense-title">Registrar gasto</h2>
        <p>El servidor volverá a calcular y validar la división.</p>
      </div>
      <form className="group-form" onSubmit={(event) => void submit(event)}>
        <label>
          Concepto
          <input
            required
            maxLength={200}
            value={concepto}
            onChange={(event) => setConcepto(event.target.value)}
            placeholder="Comida"
          />
        </label>
        <div className="group-form-row">
          <label>
            Monto ({moneda})
            <input
              required
              inputMode="decimal"
              value={monto}
              onChange={(event) => setMonto(event.target.value)}
              placeholder="100.00"
            />
          </label>
          <label>
            Fecha
            <input
              required
              type="date"
              value={fecha}
              onChange={(event) => setFecha(event.target.value)}
            />
          </label>
        </div>
        <label>
          Pagó
          <select
            required
            value={pagador}
            onChange={(event) => setPagador(event.target.value)}
          >
            {miembros.map((miembro) => (
              <option
                key={miembro.nombre_usuario}
                value={miembro.nombre_usuario}
              >
                {miembro.nombre_visible} (@{miembro.nombre_usuario})
              </option>
            ))}
          </select>
        </label>
        <label>
          División
          <select
            value={regla}
            onChange={(event) =>
              setRegla(event.target.value as ReglaDivisionGrupo)
            }
          >
            <option value="iguales">Partes iguales</option>
            <option value="porcentajes">Porcentajes</option>
            <option value="fijos">Montos fijos</option>
          </select>
        </label>
        <fieldset className="group-participant-list">
          <legend>Participantes</legend>
          {miembros.map((miembro) => (
            <label key={miembro.nombre_usuario} className="group-checkbox-row">
              <input
                type="checkbox"
                checked={participantes.includes(miembro.nombre_usuario)}
                onChange={() => toggleParticipante(miembro.nombre_usuario)}
              />
              <span>
                {miembro.nombre_visible} (@{miembro.nombre_usuario})
              </span>
            </label>
          ))}
        </fieldset>
        {regla !== "iguales" && (
          <div className="group-split-values">
            {participantes.map((nombreUsuario) => (
              <label key={nombreUsuario}>
                {nombreUsuario}{" "}
                {regla === "porcentajes" ? "(%)" : `(${moneda})`}
                <input
                  inputMode="decimal"
                  value={valores[nombreUsuario] ?? ""}
                  onChange={(event) =>
                    setValores((actuales) => ({
                      ...actuales,
                      [nombreUsuario]: event.target.value,
                    }))
                  }
                  placeholder={regla === "porcentajes" ? "0" : "0.00"}
                />
              </label>
            ))}
          </div>
        )}
        <div className="group-preview" aria-live="polite">
          <h3>Vista previa</h3>
          {cargandoPreview ? (
            <p className="group-muted">Calculando división…</p>
          ) : errorPreview ? (
            <p className="group-error">{errorPreview}</p>
          ) : (
            <ul>
              {partes.map((parte) => (
                <li key={parte.usuario}>
                  <span>{parte.usuario}</span>
                  <strong>
                    {parte.monto} {moneda}
                  </strong>
                  {parte.residuo && (
                    <small>Incluye 0,01 de redondeo</small>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
        <button
          className="account-primary"
          type="submit"
          disabled={
            pending ||
            cargandoPreview ||
            !resultado.solicitud ||
            clavePreview !== claveSolicitud ||
            partes.length === 0
          }
        >
          {pending ? "Registrando…" : "Registrar gasto"}
        </button>
      </form>
    </section>
  );
}
