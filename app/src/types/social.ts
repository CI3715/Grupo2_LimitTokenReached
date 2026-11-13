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
