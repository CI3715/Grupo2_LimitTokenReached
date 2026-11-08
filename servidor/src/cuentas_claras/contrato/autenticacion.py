"""Contrato E5 propuesto para integrar servidor, Tauri y frontend.

Registro: nombre_usuario, nombre_visible, correo, contrasena.
Sesión: correo, contrasena -> token, vence_en (ISO 8601 con zona horaria).
Las rutas y la traducción HTTP 400/401/409 siguen a cargo del backend.
"""

from collections.abc import Callable, Mapping, Sequence
from typing import Any

from pydantic import (
    AwareDatetime,
    BaseModel,
    ConfigDict,
    Field,
    SecretStr,
    StrictStr,
    field_validator,
)
from pydantic_core import PydanticCustomError

from cuentas_claras.dominio.validacion_usuario import (
    MENSAJE_CREDENCIALES_INVALIDAS,
    MENSAJES_VALIDACION,
    ErrorValidacionUsuario,
    normalizar_correo,
    validar_contrasena_inicio,
    validar_contrasena_registro,
    validar_nombre_usuario,
    validar_nombre_visible,
)


def _validar(valor: str, regla: Callable[[str], str]) -> str:
    try:
        return regla(valor)
    except ErrorValidacionUsuario as error:
        raise PydanticCustomError(error.codigo, error.mensaje) from None


class _ContratoAuth(BaseModel):
    model_config = ConfigDict(extra="forbid", hide_input_in_errors=True)


class RegistrarUsuarioRequest(_ContratoAuth):
    nombre_usuario: StrictStr
    nombre_visible: StrictStr
    correo: StrictStr
    # El backend accede al valor explícitamente con get_secret_value().
    contrasena: SecretStr = Field(exclude=True, repr=False)

    @field_validator("nombre_usuario")
    @classmethod
    def nombre_valido(cls, valor: str) -> str:
        return _validar(valor, validar_nombre_usuario)

    @field_validator("nombre_visible")
    @classmethod
    def visible_valido(cls, valor: str) -> str:
        return _validar(valor, validar_nombre_visible)

    @field_validator("correo")
    @classmethod
    def correo_valido(cls, valor: str) -> str:
        return _validar(valor, normalizar_correo)

    @field_validator("contrasena")
    @classmethod
    def contrasena_valida(cls, valor: SecretStr) -> SecretStr:
        _validar(valor.get_secret_value(), validar_contrasena_registro)
        return valor


class IniciarSesionRequest(_ContratoAuth):
    correo: StrictStr
    contrasena: SecretStr = Field(exclude=True, repr=False)

    @field_validator("correo")
    @classmethod
    def correo_valido(cls, valor: str) -> str:
        return _validar(valor, normalizar_correo)

    @field_validator("contrasena")
    @classmethod
    def contrasena_valida(cls, valor: SecretStr) -> SecretStr:
        # Una contraseña corta pero no vacía llega al autenticador: si es
        # incorrecta, la respuesta debe ser el mismo 401 genérico que cualquier otra.
        _validar(valor.get_secret_value(), validar_contrasena_inicio)
        return valor


class UsuarioDTO(_ContratoAuth):
    nombre_usuario: StrictStr
    nombre_visible: StrictStr
    correo: StrictStr
    fecha_alta: AwareDatetime


class SesionDTO(_ContratoAuth):
    token: StrictStr = Field(min_length=1, repr=False)
    vence_en: AwareDatetime


class ErrorAutenticacionDTO(_ContratoAuth):
    campo: StrictStr
    mensaje: StrictStr


def error_datos_invalidos(
    errores: Sequence[Mapping[str, Any]],
) -> ErrorAutenticacionDTO:
    """Adaptar exc.errors() sin exponer input, body, contexto ni trazas.

    Sirve para ValidationError y RequestValidationError. El backend devuelve
    este DTO con HTTP 400; no debe devolver ni registrar exc.errors() directamente.
    """
    error = errores[0] if errores else {}
    ubicacion = error.get("loc", ())
    campos = {"nombre_usuario", "nombre_visible", "correo", "contrasena"}
    campo = next((str(parte) for parte in ubicacion if parte in campos), "general")
    codigo = str(error.get("type", ""))
    mensaje = MENSAJES_VALIDACION.get(codigo)
    if mensaje is None:
        mensaje = {
            "missing": "El campo es obligatorio.",
            "extra_forbidden": "La solicitud contiene campos no permitidos.",
            "string_type": "El campo debe ser texto.",
        }.get(codigo, "Los datos enviados no son válidos.")
    return ErrorAutenticacionDTO(campo=campo, mensaje=mensaje)


def error_credenciales_invalidas() -> ErrorAutenticacionDTO:
    return ErrorAutenticacionDTO(
        campo="credenciales", mensaje=MENSAJE_CREDENCIALES_INVALIDAS
    )
