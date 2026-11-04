"""Validación de unicidad reutilizable antes de persistir un registro de E5."""

from typing import Protocol

from cuentas_claras.dominio.validacion_usuario import (
    ErrorValidacionUsuario,
    normalizar_correo,
    validar_nombre_usuario,
)


class ConsultaUsuarios(Protocol):
    def existe_nombre_usuario(self, nombre_usuario: str) -> bool: ...

    def existe_correo(self, correo_normalizado: str) -> bool: ...


def validar_unicidad_usuario(
    nombre_usuario: str, correo: str, usuarios: ConsultaUsuarios
) -> None:
    """El adaptador debe reforzar la unicidad al guardar para evitar carreras."""
    nombre = validar_nombre_usuario(nombre_usuario)
    correo_normalizado = normalizar_correo(correo)
    if usuarios.existe_nombre_usuario(nombre):
        raise ErrorValidacionUsuario("nombre_usuario", "usuario_duplicado")
    if usuarios.existe_correo(correo_normalizado):
        raise ErrorValidacionUsuario("correo", "correo_duplicado")
