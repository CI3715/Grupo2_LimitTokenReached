from datetime import datetime, timezone
from decimal import Decimal, InvalidOperation
from typing import Annotated
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, Response, status
from sqlalchemy import and_, or_, select
from sqlalchemy.orm import Session

from cuentas_claras.app.rutas.autenticacion import obtener_usuario_actual
from cuentas_claras.contrato.social import (
    CrearAmistadRequest,
    CrearGrupoRequest,
    InvitarMiembroRequest,
    RegistrarGastoRequest,
    RespuestaRequest,
)
from cuentas_claras.dominio.usuario import Usuario
from cuentas_claras.infraestructura.base_datos import obtener_db
from cuentas_claras.infraestructura.modelos_sociales import (
    AmistadDB,
    GastoDB,
    GrupoDB,
    MiembroGrupoDB,
    ParteGastoDB,
)
from cuentas_claras.infraestructura.modelos_usuario import UsuarioDB

router = APIRouter()
NO_ES_MIEMBRO = "Grupo no encontrado."


def _error(codigo: int, mensaje: str) -> HTTPException:
    return HTTPException(status_code=codigo, detail=mensaje)


def _usuario_por_nombre(db: Session, nombre: str) -> UsuarioDB:
    usuario = db.scalar(select(UsuarioDB).where(UsuarioDB.nombre_usuario == nombre))
    if usuario is None:
        raise _error(status.HTTP_404_NOT_FOUND, "Usuario no encontrado.")
    return usuario


def _grupo_miembro(db: Session, grupo_id: str, usuario_id: str) -> GrupoDB:
    grupo = db.scalar(
        select(GrupoDB)
        .join(MiembroGrupoDB, MiembroGrupoDB.grupo_id == GrupoDB.id)
        .where(
            GrupoDB.id == grupo_id,
            MiembroGrupoDB.usuario_id == usuario_id,
            MiembroGrupoDB.estado == "aceptada",
        )
    )
    if grupo is None:
        raise _error(status.HTTP_404_NOT_FOUND, NO_ES_MIEMBRO)
    return grupo


def _a_monto(centimos: int) -> str:
    signo = "-" if centimos < 0 else ""
    absoluto = abs(centimos)
    return f"{signo}{absoluto // 100}.{absoluto % 100:02d}"


def _maximo_dos_decimales(numero: Decimal) -> bool:
    exponente = numero.as_tuple().exponent
    return isinstance(exponente, int) and exponente >= -2


def _decimal_positivo(valor: str, campo: str) -> Decimal:
    try:
        numero = Decimal(valor)
    except InvalidOperation:
        raise _error(
            status.HTTP_400_BAD_REQUEST, f"{campo} no es un decimal válido."
        ) from None
    if not numero.is_finite() or numero <= 0 or not _maximo_dos_decimales(numero):
        raise _error(
            status.HTTP_400_BAD_REQUEST,
            f"{campo} debe ser positivo y tener como máximo dos decimales.",
        )
    return numero


def _a_centimos(valor: Decimal) -> int:
    return int(valor * 100)


def _validar_valores(
    valores: dict[str, str], participantes: set[str]
) -> dict[str, Decimal]:
    if set(valores) != participantes:
        raise _error(
            status.HTTP_400_BAD_REQUEST,
            "Los valores deben corresponder exactamente a los participantes.",
        )
    resultado: dict[str, Decimal] = {}
    for usuario, valor in valores.items():
        try:
            numero = Decimal(valor)
        except InvalidOperation:
            raise _error(
                status.HTTP_400_BAD_REQUEST,
                f"El valor de {usuario} no es un decimal válido.",
            ) from None
        if not numero.is_finite() or numero < 0 or not _maximo_dos_decimales(numero):
            raise _error(
                status.HTTP_400_BAD_REQUEST,
                f"El valor de {usuario} debe ser no negativo y tener "
                "como máximo dos decimales.",
            )
        resultado[usuario] = numero
    return resultado


def _dividir(
    monto_centimos: int,
    regla: str,
    valores: dict[str, Decimal],
    participantes: list[UsuarioDB],
) -> dict[str, tuple[int, bool]]:
    nombres = sorted(usuario.nombre_usuario for usuario in participantes)
    partes: dict[str, int] = {}

    if regla == "iguales":
        if valores:
            raise _error(
                status.HTTP_400_BAD_REQUEST,
                "La división en partes iguales no recibe valores adicionales.",
            )
        base, residuo = divmod(monto_centimos, len(nombres))
        partes = {nombre: base for nombre in nombres}
    elif regla == "porcentajes":
        suma = sum(valores.values(), Decimal(0))
        diferencia_porcentual = Decimal(100) - suma
        if suma != Decimal(100):
            verbo = "faltan" if diferencia_porcentual > 0 else "sobran"
            raise _error(
                status.HTTP_400_BAD_REQUEST,
                f"Los porcentajes suman {suma}; {verbo} "
                f"{abs(diferencia_porcentual)}.",
            )
        partes = {
            nombre: int((Decimal(monto_centimos) * valores[nombre]) / Decimal(100))
            for nombre in nombres
        }
        residuo = monto_centimos - sum(partes.values())
    else:
        montos = {nombre: _a_centimos(valores[nombre]) for nombre in nombres}
        suma_centimos = sum(montos.values())
        diferencia_centimos = monto_centimos - suma_centimos
        if diferencia_centimos != 0:
            verbo = "faltan" if diferencia_centimos > 0 else "sobran"
            raise _error(
                status.HTTP_400_BAD_REQUEST,
                f"Los montos suman {_a_monto(suma_centimos)}; {verbo} "
                f"{_a_monto(abs(diferencia_centimos))}.",
            )
        partes = montos
        residuo = 0

    resultado = {nombre: (parte, False) for nombre, parte in partes.items()}
    for nombre in nombres[:residuo]:
        monto_parte, _ = resultado[nombre]
        resultado[nombre] = (monto_parte + 1, True)
    return {
        nombre: (monto_parte, recibio_residuo)
        for nombre, (monto_parte, recibio_residuo) in resultado.items()
    }


@router.get("/usuarios")
def buscar_usuario(
    usuario: str,
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, str]:
    del actor
    if not usuario:
        raise _error(
            status.HTTP_400_BAD_REQUEST, "El nombre de usuario es obligatorio."
        )
    encontrado = _usuario_por_nombre(db, usuario)
    return {
        "nombre_usuario": encontrado.nombre_usuario,
        "nombre_visible": encontrado.nombre_visible,
    }


@router.post("/amistades", status_code=status.HTTP_201_CREATED)
def invitar_amigo(
    datos: CrearAmistadRequest,
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, str]:
    destino = _usuario_por_nombre(db, datos.usuario)
    if destino.id == actor.id:
        raise _error(status.HTTP_400_BAD_REQUEST, "No puedes invitarte a ti mismo.")

    relacion = db.scalar(
        select(AmistadDB).where(
            or_(
                and_(
                    AmistadDB.emisor_id == actor.id,
                    AmistadDB.destinatario_id == destino.id,
                ),
                and_(
                    AmistadDB.emisor_id == destino.id,
                    AmistadDB.destinatario_id == actor.id,
                ),
            ),
            AmistadDB.estado.in_(("pendiente", "aceptada")),
        )
    )
    if relacion is not None:
        raise _error(
            status.HTTP_409_CONFLICT,
            "La invitación ya existe o ya son amigos.",
        )

    invitacion = AmistadDB(
        id=str(uuid4()),
        emisor_id=actor.id,
        destinatario_id=destino.id,
        estado="pendiente",
        creada_en=datetime.now(timezone.utc),
    )
    db.add(invitacion)
    db.commit()
    return {"id": invitacion.id}


@router.post("/amistades/{id}/respuesta")
def responder_amistad(
    id: str,
    datos: RespuestaRequest,
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, str]:
    invitacion = db.get(AmistadDB, id)
    if invitacion is None:
        raise _error(status.HTTP_404_NOT_FOUND, "Invitación no encontrada.")
    if invitacion.destinatario_id != actor.id:
        raise _error(status.HTTP_403_FORBIDDEN, "Solo el destinatario puede responder.")
    if invitacion.estado != "pendiente":
        raise _error(status.HTTP_409_CONFLICT, "La invitación ya fue respondida.")
    invitacion.estado = "aceptada" if datos.aceptar else "rechazada"
    db.commit()
    return {"estado": invitacion.estado}


@router.get("/amistades/{nombre_usuario}/id")
def obtener_id_amistad(
    nombre_usuario: str,
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, str]:
    destino = _usuario_por_nombre(db, nombre_usuario)
    amistad = db.scalar(
        select(AmistadDB).where(
            AmistadDB.estado == "aceptada",
            or_(
                and_(
                    AmistadDB.emisor_id == actor.id,
                    AmistadDB.destinatario_id == destino.id,
                ),
                and_(
                    AmistadDB.emisor_id == destino.id,
                    AmistadDB.destinatario_id == actor.id,
                ),
            ),
        )
    )
    if amistad is None:
        raise _error(status.HTTP_404_NOT_FOUND, "Amistad no encontrada.")
    return {"id": amistad.id}


@router.get("/amistades")
def listar_amistades(
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, list[dict[str, str]]]:
    relaciones = db.scalars(
        select(AmistadDB).where(
            or_(AmistadDB.emisor_id == actor.id, AmistadDB.destinatario_id == actor.id)
        )
    ).all()
    amigos: list[dict[str, str]] = []
    enviadas: list[dict[str, str]] = []
    recibidas: list[dict[str, str]] = []
    for relacion in relaciones:
        if relacion.estado == "aceptada":
            otro_id = (
                relacion.destinatario_id
                if relacion.emisor_id == actor.id
                else relacion.emisor_id
            )
            otro = db.get(UsuarioDB, otro_id)
            if otro is not None:
                amigos.append(
                    {
                        "nombre_usuario": otro.nombre_usuario,
                        "nombre_visible": otro.nombre_visible,
                    }
                )
        elif relacion.estado == "pendiente":
            otro_id = (
                relacion.destinatario_id
                if relacion.emisor_id == actor.id
                else relacion.emisor_id
            )
            otro = db.get(UsuarioDB, otro_id)
            if otro is not None:
                elemento = {
                    "id": relacion.id,
                    "nombre_usuario": otro.nombre_usuario,
                    "nombre_visible": otro.nombre_visible,
                }
                (enviadas if relacion.emisor_id == actor.id else recibidas).append(
                    elemento
                )
    amigos.sort(key=lambda usuario: usuario["nombre_usuario"])
    enviadas.sort(key=lambda invitacion: invitacion["nombre_usuario"])
    recibidas.sort(key=lambda invitacion: invitacion["nombre_usuario"])
    return {"amigos": amigos, "enviadas": enviadas, "recibidas": recibidas}


@router.delete("/amistades/{id}", status_code=status.HTTP_204_NO_CONTENT)
def eliminar_amistad(
    id: str,
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> Response:
    amistad = db.get(AmistadDB, id)
    if amistad is None or amistad.estado != "aceptada":
        raise _error(status.HTTP_404_NOT_FOUND, "Amistad no encontrada.")
    if actor.id not in (amistad.emisor_id, amistad.destinatario_id):
        raise _error(status.HTTP_403_FORBIDDEN, "No puedes eliminar esta amistad.")
    db.delete(amistad)
    db.commit()
    return Response(status_code=status.HTTP_204_NO_CONTENT)


@router.post("/grupos", status_code=status.HTTP_201_CREATED)
def crear_grupo(
    datos: CrearGrupoRequest,
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, str | None]:
    nombre = datos.nombre.strip()
    if not nombre:
        raise _error(status.HTTP_400_BAD_REQUEST, "El nombre del grupo es obligatorio.")
    grupo = GrupoDB(
        id=str(uuid4()),
        nombre=nombre,
        descripcion=datos.descripcion,
        moneda=datos.moneda,
        actividad=datos.actividad,
        creador_id=actor.id,
        creado_en=datetime.now(timezone.utc),
    )
    db.add(grupo)
    db.flush()
    db.add(
        MiembroGrupoDB(
            id=str(uuid4()),
            grupo_id=grupo.id,
            usuario_id=actor.id,
            rol="administrador",
            estado="aceptada",
            invitado_por_id=actor.id,
            creada_en=datetime.now(timezone.utc),
        )
    )
    db.commit()
    return {
        "id": grupo.id,
        "nombre": grupo.nombre,
        "descripcion": grupo.descripcion,
        "moneda": grupo.moneda,
        "actividad": grupo.actividad,
    }


@router.get("/grupos")
def listar_grupos(
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, list[dict[str, str | None]]]:
    filas = db.execute(
        select(MiembroGrupoDB, GrupoDB)
        .join(GrupoDB, GrupoDB.id == MiembroGrupoDB.grupo_id)
        .where(MiembroGrupoDB.usuario_id == actor.id)
    ).all()
    miembros: list[dict[str, str | None]] = []
    invitaciones: list[dict[str, str | None]] = []
    for membresia, grupo in filas:
        if membresia.estado not in ("aceptada", "pendiente"):
            continue
        elemento = {
            "id": grupo.id,
            "nombre": grupo.nombre,
            "descripcion": grupo.descripcion,
            "moneda": grupo.moneda,
            "actividad": grupo.actividad,
        }
        (miembros if membresia.estado == "aceptada" else invitaciones).append(elemento)
    miembros.sort(key=lambda grupo: grupo["nombre"] or "")
    invitaciones.sort(key=lambda grupo: grupo["nombre"] or "")
    return {"miembros": miembros, "invitaciones": invitaciones}


@router.post("/grupos/{id}/miembros", status_code=status.HTTP_201_CREATED)
def invitar_miembro(
    id: str,
    datos: InvitarMiembroRequest,
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, str]:
    grupo = _grupo_miembro(db, id, actor.id)
    admin = db.scalar(
        select(MiembroGrupoDB).where(
            MiembroGrupoDB.grupo_id == id,
            MiembroGrupoDB.usuario_id == actor.id,
            MiembroGrupoDB.estado == "aceptada",
            MiembroGrupoDB.rol == "administrador",
        )
    )
    if admin is None:
        raise _error(status.HTTP_403_FORBIDDEN, "Solo el administrador puede invitar.")
    destino = _usuario_por_nombre(db, datos.usuario)
    if destino.id == actor.id:
        raise _error(status.HTTP_409_CONFLICT, "El usuario ya es miembro.")
    amistad = db.scalar(
        select(AmistadDB.id).where(
            AmistadDB.estado == "aceptada",
            or_(
                and_(
                    AmistadDB.emisor_id == actor.id,
                    AmistadDB.destinatario_id == destino.id,
                ),
                and_(
                    AmistadDB.emisor_id == destino.id,
                    AmistadDB.destinatario_id == actor.id,
                ),
            ),
        )
    )
    if amistad is None:
        raise _error(status.HTTP_403_FORBIDDEN, "Solo puedes invitar a tus amigos.")
    existente = db.scalar(
        select(MiembroGrupoDB).where(
            MiembroGrupoDB.grupo_id == id, MiembroGrupoDB.usuario_id == destino.id
        )
    )
    if existente is not None:
        if existente.estado in ("pendiente", "aceptada"):
            raise _error(
                status.HTTP_409_CONFLICT, "Ya existe una invitación o membresía."
            )
        existente.estado = "pendiente"
        existente.rol = "miembro"
        existente.invitado_por_id = actor.id
        existente.creada_en = datetime.now(timezone.utc)
    else:
        existente = MiembroGrupoDB(
            id=str(uuid4()),
            grupo_id=id,
            usuario_id=destino.id,
            rol="miembro",
            estado="pendiente",
            invitado_por_id=actor.id,
            creada_en=datetime.now(timezone.utc),
        )
        db.add(existente)
    db.commit()
    return {
        "id": grupo.id,
        "invitacion_id": existente.id,
        "usuario": destino.nombre_usuario,
    }


@router.post("/grupos/{id}/miembros/yo/respuesta")
def responder_invitacion_grupo(
    id: str,
    datos: RespuestaRequest,
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, str]:
    grupo = db.get(GrupoDB, id)
    invitacion = db.scalar(
        select(MiembroGrupoDB).where(
            MiembroGrupoDB.grupo_id == id,
            MiembroGrupoDB.usuario_id == actor.id,
        )
    )
    if grupo is None or invitacion is None:
        raise _error(status.HTTP_404_NOT_FOUND, NO_ES_MIEMBRO)
    if invitacion.estado != "pendiente":
        raise _error(status.HTTP_409_CONFLICT, "La invitación ya fue respondida.")
    invitacion.estado = "aceptada" if datos.aceptar else "rechazada"
    db.commit()
    return {"id": grupo.id, "estado": invitacion.estado}


@router.post("/grupos/{id}/gastos", status_code=status.HTTP_201_CREATED)
def registrar_gasto(
    id: str,
    datos: RegistrarGastoRequest,
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, object]:
    _grupo_miembro(db, id, actor.id)
    miembros = db.scalars(
        select(UsuarioDB)
        .join(MiembroGrupoDB, MiembroGrupoDB.usuario_id == UsuarioDB.id)
        .where(
            MiembroGrupoDB.grupo_id == id,
            MiembroGrupoDB.estado == "aceptada",
        )
    ).all()
    por_nombre = {miembro.nombre_usuario: miembro for miembro in miembros}
    if len(set(datos.participantes)) != len(datos.participantes):
        raise _error(status.HTTP_400_BAD_REQUEST, "Hay participantes duplicados.")
    pagador = por_nombre.get(datos.pagador)
    participantes = [por_nombre.get(nombre) for nombre in datos.participantes]
    if pagador is None or any(participante is None for participante in participantes):
        raise _error(
            status.HTTP_400_BAD_REQUEST,
            "El pagador y todos los participantes deben ser miembros aceptados.",
        )
    assert all(participante is not None for participante in participantes)
    participantes_validos = [p for p in participantes if p is not None]
    monto = _decimal_positivo(datos.monto, "El monto")
    monto_centimos = _a_centimos(monto)
    concepto = datos.concepto.strip()
    if not concepto:
        raise _error(status.HTTP_400_BAD_REQUEST, "El concepto es obligatorio.")
    valores = (
        {}
        if datos.regla == "iguales"
        else _validar_valores(datos.valores, set(datos.participantes))
    )
    if datos.regla == "iguales" and datos.valores:
        raise _error(
            status.HTTP_400_BAD_REQUEST,
            "Valores no permitidos para partes iguales.",
        )
    asignaciones = _dividir(monto_centimos, datos.regla, valores, participantes_validos)

    gasto = GastoDB(
        id=str(uuid4()),
        grupo_id=id,
        concepto=concepto,
        monto_centimos=monto_centimos,
        fecha=datos.fecha,
        pagador_id=pagador.id,
        regla=datos.regla,
        creado_en=datetime.now(timezone.utc),
    )
    db.add(gasto)
    db.flush()
    nombre_por_id = {
        usuario.id: usuario.nombre_usuario for usuario in participantes_validos
    }
    partes: list[dict[str, object]] = []
    for participante in participantes_validos:
        monto_parte, recibio_residuo = asignaciones[participante.nombre_usuario]
        db.add(
            ParteGastoDB(
                id=str(uuid4()),
                gasto_id=gasto.id,
                usuario_id=participante.id,
                monto_centimos=monto_parte,
                recibio_residuo=recibio_residuo,
            )
        )
        partes.append(
            {
                "usuario": nombre_por_id[participante.id],
                "monto": _a_monto(monto_parte),
                "residuo": recibio_residuo,
            }
        )
    db.commit()
    partes.sort(key=lambda parte: str(parte["usuario"]))
    return {"id": gasto.id, "partes": partes}


@router.get("/grupos/{id}")
def ver_grupo(
    id: str,
    actor: Annotated[Usuario, Depends(obtener_usuario_actual)],
    db: Annotated[Session, Depends(obtener_db)],
) -> dict[str, object]:
    grupo = _grupo_miembro(db, id, actor.id)
    miembros = db.execute(
        select(UsuarioDB, MiembroGrupoDB)
        .join(MiembroGrupoDB, MiembroGrupoDB.usuario_id == UsuarioDB.id)
        .where(
            MiembroGrupoDB.grupo_id == id,
            MiembroGrupoDB.estado == "aceptada",
        )
        .order_by(UsuarioDB.nombre_usuario)
    ).all()
    gastos = db.scalars(
        select(GastoDB)
        .where(GastoDB.grupo_id == id)
        .order_by(GastoDB.fecha, GastoDB.creado_en, GastoDB.id)
    ).all()
    balances = {usuario.nombre_usuario: 0 for usuario, _ in miembros}
    gastos_dto: list[dict[str, object]] = []
    ultima_parte: list[dict[str, object]] = []
    for gasto in gastos:
        pagador = db.get(UsuarioDB, gasto.pagador_id)
        if pagador is not None:
            balances[pagador.nombre_usuario] += gasto.monto_centimos
        filas_partes = db.execute(
            select(ParteGastoDB, UsuarioDB)
            .join(UsuarioDB, UsuarioDB.id == ParteGastoDB.usuario_id)
            .where(ParteGastoDB.gasto_id == gasto.id)
            .order_by(UsuarioDB.nombre_usuario)
        ).all()
        partes_dto = []
        for parte, usuario in filas_partes:
            balances[usuario.nombre_usuario] -= parte.monto_centimos
            partes_dto.append(
                {
                    "usuario": usuario.nombre_usuario,
                    "monto": _a_monto(parte.monto_centimos),
                    "residuo": parte.recibio_residuo,
                }
            )
        pagador_nombre = pagador.nombre_usuario if pagador is not None else ""
        gastos_dto.append(
            {
                "id": gasto.id,
                "concepto": gasto.concepto,
                "monto": _a_monto(gasto.monto_centimos),
                "fecha": gasto.fecha.isoformat(),
                "pagador": pagador_nombre,
                "regla": gasto.regla,
                "partes": partes_dto,
            }
        )
        ultima_parte = partes_dto
    return {
        "id": grupo.id,
        "nombre": grupo.nombre,
        "descripcion": grupo.descripcion,
        "moneda": grupo.moneda,
        "actividad": grupo.actividad,
        "miembros": [
            {
                "nombre_usuario": usuario.nombre_usuario,
                "nombre_visible": usuario.nombre_visible,
                "rol": membresia.rol,
            }
            for usuario, membresia in miembros
        ],
        "gastos": gastos_dto,
        "balances": {
            nombre: _a_monto(monto) for nombre, monto in sorted(balances.items())
        },
        "partes": ultima_parte,
    }
