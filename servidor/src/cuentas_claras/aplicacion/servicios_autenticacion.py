from collections.abc import Callable
from datetime import datetime
from typing import Protocol
from uuid import uuid4

from cuentas_claras.aplicacion.validar_registro_usuario import (
    ConsultaUsuarios,
    validar_unicidad_usuario,
)
from cuentas_claras.dominio.usuario import Usuario
from cuentas_claras.dominio.validacion_usuario import (
    normalizar_correo,
    validar_contrasena_inicio,
    validar_contrasena_registro,
    validar_nombre_usuario,
    validar_nombre_visible,
)


class RepositorioUsuariosAuth(ConsultaUsuarios, Protocol):
    def guardar(self, usuario: Usuario) -> Usuario: ...

    def buscar_por_correo(self, correo: str) -> Usuario | None: ...

    def buscar_por_id(self, usuario_id: str) -> Usuario | None: ...


class CredencialesInvalidas(Exception):
    pass


class ServicioAutenticacion:
    def __init__(
        self,
        repositorio: RepositorioUsuariosAuth,
        crear_hash: Callable[[str], str],
        verificar_clave: Callable[[str | None, str], bool],
        crear_token: Callable[[str, datetime], tuple[str, datetime]],
        ahora: Callable[[], datetime],
    ) -> None:
        self._repositorio = repositorio
        self._crear_hash = crear_hash
        self._verificar_clave = verificar_clave
        self._crear_token = crear_token
        self._ahora = ahora

    def registrar(
        self,
        nombre_usuario: str,
        nombre_visible: str,
        correo: str,
        contrasena: str,
    ) -> Usuario:
        nombre_usuario = validar_nombre_usuario(nombre_usuario)
        nombre_visible = validar_nombre_visible(nombre_visible)
        correo = normalizar_correo(correo)
        contrasena = validar_contrasena_registro(contrasena)
        validar_unicidad_usuario(nombre_usuario, correo, self._repositorio)

        usuario = Usuario(
            id=str(uuid4()),
            nombre_usuario=nombre_usuario,
            nombre_visible=nombre_visible,
            correo=correo,
            hash_contrasena=self._crear_hash(contrasena),
            fecha_alta=self._ahora(),
        )
        return self._repositorio.guardar(usuario)

    def iniciar_sesion(self, correo: str, contrasena: str) -> tuple[str, datetime]:
        correo = normalizar_correo(correo)
        contrasena = validar_contrasena_inicio(contrasena)
        usuario = self._repositorio.buscar_por_correo(correo)
        hash_guardado = usuario.hash_contrasena if usuario is not None else None

        if not self._verificar_clave(hash_guardado, contrasena):
            raise CredencialesInvalidas
        if usuario is None:
            raise CredencialesInvalidas

        return self._crear_token(usuario.id, self._ahora())

    def obtener_usuario(self, usuario_id: str) -> Usuario | None:
        return self._repositorio.buscar_por_id(usuario_id)
