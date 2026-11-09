// Solo comprobación de tipos: comparte los casos de contrato de Python y Rust.
import type contratos from "../../qa/fixtures/autenticacion.json";
import type {
  ErrorAutenticacionDTO,
  IniciarSesionRequest,
  RegistrarUsuarioRequest,
  SesionDTO,
  UsuarioDTO,
} from "../src/types/autenticacion";

type CumpleContrato<Esperado, Recibido extends Esperado> = Recibido;

export type RegistroCompartido = CumpleContrato<
  RegistrarUsuarioRequest,
  typeof contratos.registro
>;
export type InicioCompartido = CumpleContrato<
  IniciarSesionRequest,
  typeof contratos.inicio_sesion
>;
export type UsuarioCompartido = CumpleContrato<
  UsuarioDTO,
  typeof contratos.usuario
>;
export type SesionCompartida = CumpleContrato<SesionDTO, typeof contratos.sesion>;
export type ErrorCompartido = CumpleContrato<
  ErrorAutenticacionDTO,
  typeof contratos.credenciales_invalidas
>;
