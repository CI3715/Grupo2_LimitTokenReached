export interface UsuarioResumenDTO {
  nombre_usuario: string;
  nombre_visible: string;
}

export interface CrearAmistadRequestDTO {
  usuario: string;
}

export interface AmistadCreadaDTO {
  id: string;
}

export interface RespuestaInvitacionDTO {
  estado: string;
}

export interface InvitacionAmistadDTO {
  id: string;
  nombre_usuario: string;
  nombre_visible: string;
}

export interface ListaAmistadesDTO {
  amigos: UsuarioResumenDTO[];
  enviadas: InvitacionAmistadDTO[];
  recibidas: InvitacionAmistadDTO[];
}

export interface CrearGrupoRequestDTO {
  nombre: string;
  descripcion: string | null;
  moneda: string;
  actividad: string | null;
}

export interface GrupoDTO {
  id: string;
  nombre: string;
  descripcion: string | null;
  moneda: string;
  actividad: string | null;
}

export interface ListaGruposDTO {
  miembros: GrupoDTO[];
  invitaciones: GrupoDTO[];
}

export interface InvitarMiembroGrupoRequestDTO {
  usuario: string;
}

export interface InvitacionGrupoCreadaDTO {
  id: string;
  invitacion_id: string;
  usuario: string;
}

export interface RespuestaInvitacionGrupoDTO {
  id: string;
  estado: string;
}

export type ReglaDivisionGrupo = "iguales" | "porcentajes" | "fijos";

export interface RegistrarGastoRequestDTO {
  concepto: string;
  monto: string;
  fecha: string;
  pagador: string;
  regla: ReglaDivisionGrupo;
  participantes: string[];
  valores: Record<string, string>;
}

export interface ParteGastoDTO {
  usuario: string;
  monto: string;
  residuo: boolean;
}

export interface GastoCreadoDTO {
  id: string;
  partes: ParteGastoDTO[];
}

export interface GastoGrupoDTO {
  id: string;
  concepto: string;
  monto: string;
  fecha: string;
  pagador: string;
  regla: ReglaDivisionGrupo;
  partes: ParteGastoDTO[];
}

export interface MiembroGrupoDTO {
  nombre_usuario: string;
  nombre_visible: string;
  rol: string;
}

export interface DetalleGrupoDTO extends GrupoDTO {
  miembros: MiembroGrupoDTO[];
  gastos: GastoGrupoDTO[];
  balances: Record<string, string>;
  partes: ParteGastoDTO[];
}
