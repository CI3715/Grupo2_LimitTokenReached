"""Reglas de entradas de usuario de E5; no registra usuarios ni crea sesiones."""

import re

from email_validator import EmailNotValidError, validate_email

MENSAJES_VALIDACION = {
    "usuario_longitud": "El nombre de usuario debe tener entre 3 y 30 caracteres.",
    "usuario_formato": (
        "El nombre de usuario solo admite letras minúsculas, dígitos, "
        "punto y guion bajo."
    ),
    "nombre_visible_vacio": "El nombre visible es obligatorio.",
    "correo_invalido": "El correo debe tener un formato válido.",
    "contrasena_vacia": "La contraseña es obligatoria.",
    "contrasena_corta": "La contraseña debe tener al menos 10 caracteres.",
    "usuario_duplicado": "El nombre de usuario ya está registrado.",
    "correo_duplicado": "El correo ya está registrado.",
}
MENSAJE_CREDENCIALES_INVALIDAS = "Correo o contraseña incorrectos."


class ErrorValidacionUsuario(ValueError):
    """Solo conserva el campo y un mensaje fijo, nunca el valor recibido."""

    def __init__(self, campo: str, codigo: str) -> None:
        self.campo = campo
        self.codigo = codigo
        self.mensaje = MENSAJES_VALIDACION[codigo]
        super().__init__(self.mensaje)


def validar_nombre_usuario(valor: str) -> str:
    if not 3 <= len(valor) <= 30:
        raise ErrorValidacionUsuario("nombre_usuario", "usuario_longitud")
    if re.fullmatch(r"[a-z0-9._]+", valor) is None:
        raise ErrorValidacionUsuario("nombre_usuario", "usuario_formato")
    return valor


def validar_nombre_visible(valor: str) -> str:
    limpio = valor.strip()
    if not limpio:
        raise ErrorValidacionUsuario("nombre_visible", "nombre_visible_vacio")
    return limpio


def normalizar_correo(valor: str) -> str:
    try:
        # Solo formato: registrar e iniciar sesión no dependen de DNS ni de la red.
        correo = validate_email(valor.strip(), check_deliverability=False)
    except EmailNotValidError:
        raise ErrorValidacionUsuario("correo", "correo_invalido") from None
    # E5 compara el correo completo sin distinguir mayúsculas.
    return str(correo.normalized).casefold()


def validar_contrasena_inicio(valor: str) -> str:
    if not valor.strip():
        raise ErrorValidacionUsuario("contrasena", "contrasena_vacia")
    # Nunca recortar ni normalizar una contraseña.
    return valor


def validar_contrasena_registro(valor: str) -> str:
    validar_contrasena_inicio(valor)
    if len(valor) < 10:
        raise ErrorValidacionUsuario("contrasena", "contrasena_corta")
    return valor
