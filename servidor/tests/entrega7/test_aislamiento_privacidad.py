"""Aislamiento, idempotencia y privacidad de GET /cambios y de los gastos (E7)."""

from typing import Any
from uuid import uuid4

from fastapi import status
from qa.entrega6.escenario import EscenarioSocial
from sqlalchemy import inspect

PROHIBIDOS = {
    "cuenta",
    "cuentas",
    "saldo",
    "saldos",
    "movimiento",
    "movimientos",
    "tasa",
    "tasas",
}


def _crear_grupo(escenario: EscenarioSocial) -> str:
    respuesta = escenario.cliente.post(
        "/grupos",
        json={
            "nombre": "Viaje",
            "descripcion": None,
            "moneda": "USD",
            "actividad": None,
        },
        headers=escenario.cabecera("ana"),
    )
    assert respuesta.status_code == status.HTTP_201_CREATED
    return str(respuesta.json()["id"])


def _invitar(
    escenario: EscenarioSocial, grupo_id: str, usuario: str, aceptar: bool
) -> None:
    amistad = escenario.cliente.post(
        "/amistades",
        json={"usuario": usuario},
        headers=escenario.cabecera("ana"),
    )
    assert amistad.status_code == status.HTTP_201_CREATED
    aceptada = escenario.cliente.post(
        f"/amistades/{amistad.json()['id']}/respuesta",
        json={"aceptar": True},
        headers=escenario.cabecera(usuario),
    )
    assert aceptada.status_code == status.HTTP_200_OK
    invitacion = escenario.cliente.post(
        f"/grupos/{grupo_id}/miembros",
        json={"usuario": usuario},
        headers=escenario.cabecera("ana"),
    )
    assert invitacion.status_code == status.HTTP_201_CREATED
    if aceptar:
        respuesta = escenario.cliente.post(
            f"/grupos/{grupo_id}/miembros/yo/respuesta",
            json={"aceptar": True},
            headers=escenario.cabecera(usuario),
        )
        assert respuesta.status_code == status.HTTP_200_OK


def _grupo_con(
    escenario: EscenarioSocial,
    aceptados: tuple[str, ...] = (),
    pendientes: tuple[str, ...] = (),
) -> str:
    grupo_id = _crear_grupo(escenario)
    for usuario in aceptados:
        _invitar(escenario, grupo_id, usuario, aceptar=True)
    for usuario in pendientes:
        _invitar(escenario, grupo_id, usuario, aceptar=False)
    return grupo_id


def _gasto(participantes: list[str]) -> dict[str, Any]:
    return {
        "concepto": "Hotel",
        "monto": "100.00",
        "fecha": "2026-11-16",
        "pagador": "ana",
        "regla": "iguales",
        "participantes": participantes,
        "valores": {},
    }


def _registrar(
    escenario: EscenarioSocial,
    grupo_id: str,
    actor: str,
    participantes: list[str],
    clave: str | None = None,
) -> Any:
    return escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json=_gasto(participantes),
        headers={
            **escenario.cabecera(actor),
            "Idempotency-Key": clave or str(uuid4()),
        },
    )


def _cambios(escenario: EscenarioSocial, usuario: str, desde: int = 0) -> Any:
    respuesta = escenario.cliente.get(
        "/cambios",
        params={"desde": desde},
        headers=escenario.cabecera(usuario),
    )
    assert respuesta.status_code == status.HTTP_200_OK
    return respuesta.json()


def test_quien_nunca_fue_miembro_no_recibe_eventos(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _grupo_con(escenario, aceptados=("beto",))
    assert _registrar(escenario, grupo_id, "ana", ["ana", "beto"]).status_code == 201

    assert len(_cambios(escenario, "beto")["eventos"]) == 1
    assert _cambios(escenario, "carla")["eventos"] == []


def test_invitado_que_no_acepto_no_recibe_eventos(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _grupo_con(escenario, pendientes=("beto",))
    assert _registrar(escenario, grupo_id, "ana", ["ana"]).status_code == 201

    assert _cambios(escenario, "beto")["eventos"] == []


def test_cada_usuario_solo_ve_sus_propios_eventos(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _grupo_con(escenario, aceptados=("beto",))
    assert _registrar(escenario, grupo_id, "ana", ["ana"]).status_code == 201

    de_ana = _cambios(escenario, "ana")
    assert len(de_ana["eventos"]) == 1
    assert _cambios(escenario, "beto")["eventos"] == []


def test_ajeno_recibe_el_mismo_404_que_ante_un_grupo_inexistente(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _grupo_con(escenario, aceptados=("beto",))

    ajeno = _registrar(escenario, grupo_id, "carla", ["carla"])
    inexistente = _registrar(escenario, str(uuid4()), "carla", ["carla"])

    assert ajeno.status_code == status.HTTP_404_NOT_FOUND
    assert inexistente.status_code == status.HTTP_404_NOT_FOUND
    assert ajeno.json() == inexistente.json()
    assert _cambios(escenario, "ana")["eventos"] == []


def test_repetir_la_clave_no_genera_eventos_de_mas(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _grupo_con(escenario, aceptados=("beto",))
    clave = str(uuid4())

    primera = _registrar(escenario, grupo_id, "ana", ["ana", "beto"], clave)
    segunda = _registrar(escenario, grupo_id, "ana", ["ana", "beto"], clave)

    assert primera.status_code == status.HTTP_201_CREATED
    assert segunda.status_code == status.HTTP_200_OK
    for usuario in ("ana", "beto"):
        assert len(_cambios(escenario, usuario)["eventos"]) == 1


def test_leer_cambios_desde_cero_muchas_veces_devuelve_lo_mismo(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _grupo_con(escenario, aceptados=("beto",))
    assert _registrar(escenario, grupo_id, "ana", ["ana", "beto"]).status_code == 201

    primera = _cambios(escenario, "beto")
    for _ in range(3):
        assert _cambios(escenario, "beto") == primera


def test_clave_usada_por_otro_usuario_da_conflicto_y_no_crea_gasto(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _grupo_con(escenario, aceptados=("beto",))
    clave = str(uuid4())

    assert _registrar(escenario, grupo_id, "ana", ["ana"], clave).status_code == 201
    ajena = _registrar(escenario, grupo_id, "beto", ["ana"], clave)

    assert ajena.status_code == status.HTTP_409_CONFLICT
    assert len(_cambios(escenario, "ana")["eventos"]) == 1


def test_clave_reutilizada_en_otro_grupo_da_conflicto(
    escenario: EscenarioSocial,
) -> None:
    primero = _crear_grupo(escenario)
    segundo = _crear_grupo(escenario)
    clave = str(uuid4())

    assert _registrar(escenario, primero, "ana", ["ana"], clave).status_code == 201
    otra = _registrar(escenario, segundo, "ana", ["ana"], clave)

    assert otra.status_code == status.HTTP_409_CONFLICT
    assert len(_cambios(escenario, "ana")["eventos"]) == 1


def test_el_servidor_no_tiene_tablas_del_libro_personal(
    escenario: EscenarioSocial,
) -> None:
    with escenario.fabrica_db() as db:
        tablas = inspect(db.get_bind()).get_table_names()
    assert tablas, "No se encontraron tablas para revisar."
    for tabla in tablas:
        assert not any(palabra in tabla for palabra in PROHIBIDOS), tabla


def _nombres_de_campos(nodo: Any) -> set[str]:
    nombres: set[str] = set()
    if isinstance(nodo, dict):
        for clave, valor in nodo.items():
            if clave == "properties" and isinstance(valor, dict):
                nombres.update(valor)
            nombres |= _nombres_de_campos(valor)
    elif isinstance(nodo, list):
        for hijo in nodo:
            nombres |= _nombres_de_campos(hijo)
    return nombres


def test_ningun_contrato_de_la_api_tiene_campos_del_libro_personal(
    escenario: EscenarioSocial,
) -> None:
    esquemas = escenario.cliente.app.openapi().get("components", {}).get("schemas", {})
    campos = _nombres_de_campos(esquemas)
    assert campos, "No se encontraron contratos para revisar."
    assert PROHIBIDOS.isdisjoint(campos), PROHIBIDOS & campos
