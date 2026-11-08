from datetime import timezone

from sqlalchemy import select
from sqlalchemy.exc import IntegrityError
from sqlalchemy.orm import Session

from cuentas_claras.dominio.usuario import Usuario
from cuentas_claras.dominio.validacion_usuario import ErrorValidacionUsuario
from cuentas_claras.infraestructura.modelos_usuario import UsuarioDB


class RepositorioUsuariosSqlAlchemy:
    def __init__(self, sesion: Session) -> None:
        self._sesion = sesion

    def existe_nombre_usuario(self, nombre_usuario: str) -> bool:
        consulta = select(UsuarioDB.id).where(
            UsuarioDB.nombre_usuario == nombre_usuario
        )
        return self._sesion.scalar(consulta) is not None

    def existe_correo(self, correo_normalizado: str) -> bool:
        consulta = select(UsuarioDB.id).where(UsuarioDB.correo == correo_normalizado)
        return self._sesion.scalar(consulta) is not None

    def buscar_por_correo(self, correo: str) -> Usuario | None:
        fila = self._sesion.scalar(select(UsuarioDB).where(UsuarioDB.correo == correo))
        return self._convertir(fila) if fila is not None else None

    def buscar_por_id(self, usuario_id: str) -> Usuario | None:
        fila = self._sesion.get(UsuarioDB, usuario_id)
        return self._convertir(fila) if fila is not None else None

    def guardar(self, usuario: Usuario) -> Usuario:
        fila = UsuarioDB(
            id=usuario.id,
            nombre_usuario=usuario.nombre_usuario,
            nombre_visible=usuario.nombre_visible,
            correo=usuario.correo,
            hash_contrasena=usuario.hash_contrasena,
            fecha_alta=usuario.fecha_alta,
        )
        self._sesion.add(fila)
        try:
            self._sesion.commit()
        except IntegrityError:
            self._sesion.rollback()
            if self.existe_nombre_usuario(usuario.nombre_usuario):
                raise ErrorValidacionUsuario(
                    "nombre_usuario", "usuario_duplicado"
                ) from None
            if self.existe_correo(usuario.correo):
                raise ErrorValidacionUsuario("correo", "correo_duplicado") from None
            raise
        return usuario

    @staticmethod
    def _convertir(fila: UsuarioDB) -> Usuario:
        fecha_alta = fila.fecha_alta
        if fecha_alta.tzinfo is None:
            fecha_alta = fecha_alta.replace(tzinfo=timezone.utc)
        return Usuario(
            id=fila.id,
            nombre_usuario=fila.nombre_usuario,
            nombre_visible=fila.nombre_visible,
            correo=fila.correo,
            hash_contrasena=fila.hash_contrasena,
            fecha_alta=fecha_alta,
        )
