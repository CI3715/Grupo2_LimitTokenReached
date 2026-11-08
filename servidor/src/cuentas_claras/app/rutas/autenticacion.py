from typing import Annotated

from fastapi import APIRouter, Depends, HTTPException, status
from fastapi.responses import JSONResponse
from fastapi.security import HTTPAuthorizationCredentials, HTTPBearer
from jwt.exceptions import InvalidTokenError
from sqlalchemy.orm import Session

from cuentas_claras.aplicacion.reloj import Reloj
from cuentas_claras.aplicacion.servicios_autenticacion import (
    CredencialesInvalidas,
    ServicioAutenticacion,
)
from cuentas_claras.contrato.autenticacion import (
    ErrorAutenticacionDTO,
    IniciarSesionRequest,
    RegistrarUsuarioRequest,
    SesionDTO,
    UsuarioDTO,
    error_credenciales_invalidas,
)
from cuentas_claras.dominio.usuario import Usuario
from cuentas_claras.dominio.validacion_usuario import ErrorValidacionUsuario
from cuentas_claras.infraestructura.base_datos import obtener_db
from cuentas_claras.infraestructura.reloj_sistema import RelojSistema
from cuentas_claras.infraestructura.repositorio_usuarios_sqlalchemy import (
    RepositorioUsuariosSqlAlchemy,
)
from cuentas_claras.infraestructura.seguridad_auth import (
    crear_hash,
    crear_token,
    verificar_contrasena,
    verificar_token,
)

router = APIRouter()
_bearer = HTTPBearer(auto_error=False)


def obtener_reloj_auth() -> Reloj:
    return RelojSistema()


def obtener_repositorio(
    sesion: Annotated[Session, Depends(obtener_db)],
) -> RepositorioUsuariosSqlAlchemy:
    return RepositorioUsuariosSqlAlchemy(sesion)


def obtener_servicio_autenticacion(
    repositorio: Annotated[RepositorioUsuariosSqlAlchemy, Depends(obtener_repositorio)],
    reloj: Annotated[Reloj, Depends(obtener_reloj_auth)],
) -> ServicioAutenticacion:
    return ServicioAutenticacion(
        repositorio=repositorio,
        crear_hash=crear_hash,
        verificar_clave=verificar_contrasena,
        crear_token=crear_token,
        ahora=reloj.ahora,
    )


def _usuario_dto(usuario: Usuario) -> UsuarioDTO:
    return UsuarioDTO(
        id=usuario.id,
        nombre_usuario=usuario.nombre_usuario,
        nombre_visible=usuario.nombre_visible,
        correo=usuario.correo,
        fecha_alta=usuario.fecha_alta,
    )


@router.post(
    "/usuarios",
    response_model=UsuarioDTO,
    status_code=status.HTTP_201_CREATED,
    responses={
        status.HTTP_400_BAD_REQUEST: {"model": ErrorAutenticacionDTO},
        status.HTTP_409_CONFLICT: {"model": ErrorAutenticacionDTO},
    },
)
def registrar_usuario(
    datos: RegistrarUsuarioRequest,
    servicio: Annotated[ServicioAutenticacion, Depends(obtener_servicio_autenticacion)],
) -> UsuarioDTO | JSONResponse:
    try:
        usuario = servicio.registrar(
            nombre_usuario=datos.nombre_usuario,
            nombre_visible=datos.nombre_visible,
            correo=datos.correo,
            contrasena=datos.contrasena.get_secret_value(),
        )
    except ErrorValidacionUsuario as error:
        codigo = (
            status.HTTP_409_CONFLICT
            if error.codigo.endswith("_duplicado")
            else status.HTTP_400_BAD_REQUEST
        )
        return JSONResponse(
            status_code=codigo,
            content=ErrorAutenticacionDTO(
                campo=error.campo, mensaje=error.mensaje
            ).model_dump(),
        )
    return _usuario_dto(usuario)


@router.post(
    "/sesiones",
    response_model=SesionDTO,
    responses={
        status.HTTP_400_BAD_REQUEST: {"model": ErrorAutenticacionDTO},
        status.HTTP_401_UNAUTHORIZED: {"model": ErrorAutenticacionDTO},
    },
)
def iniciar_sesion(
    datos: IniciarSesionRequest,
    servicio: Annotated[ServicioAutenticacion, Depends(obtener_servicio_autenticacion)],
) -> SesionDTO | JSONResponse:
    try:
        token, vence = servicio.iniciar_sesion(
            correo=datos.correo,
            contrasena=datos.contrasena.get_secret_value(),
        )
    except CredencialesInvalidas:
        return JSONResponse(
            status_code=status.HTTP_401_UNAUTHORIZED,
            content=error_credenciales_invalidas().model_dump(),
            headers={"WWW-Authenticate": "Bearer"},
        )
    except ErrorValidacionUsuario as error:
        return JSONResponse(
            status_code=status.HTTP_400_BAD_REQUEST,
            content=ErrorAutenticacionDTO(
                campo=error.campo, mensaje=error.mensaje
            ).model_dump(),
        )
    return SesionDTO(token=token, vence_en=vence)


def obtener_usuario_actual(
    credenciales: Annotated[HTTPAuthorizationCredentials | None, Depends(_bearer)],
    repositorio: Annotated[RepositorioUsuariosSqlAlchemy, Depends(obtener_repositorio)],
    reloj: Annotated[Reloj, Depends(obtener_reloj_auth)],
) -> Usuario:
    if credenciales is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Autenticación requerida.",
            headers={"WWW-Authenticate": "Bearer"},
        )
    try:
        usuario_id = verificar_token(credenciales.credentials, reloj.ahora())
    except InvalidTokenError:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Token inválido o vencido.",
            headers={"WWW-Authenticate": "Bearer"},
        ) from None

    usuario = repositorio.buscar_por_id(usuario_id)
    if usuario is None:
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Token inválido o vencido.",
            headers={"WWW-Authenticate": "Bearer"},
        )
    return usuario


@router.get(
    "/usuarios/yo",
    response_model=UsuarioDTO,
    responses={
        status.HTTP_401_UNAUTHORIZED: {
            "description": "Token ausente, inválido o vencido."
        }
    },
)
def perfil_propio(
    usuario: Annotated[Usuario, Depends(obtener_usuario_actual)],
) -> UsuarioDTO:
    return _usuario_dto(usuario)
