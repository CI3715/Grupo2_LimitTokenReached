from fastapi import status
from qa.entrega6.escenario import EscenarioSocial
from sqlalchemy import select

from cuentas_claras.infraestructura.modelos_sociales import (
    EventoCambioDB,
    GastoDB,
    ParteGastoDB,
)


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
    return respuesta.json()["id"]


def _gasto(participantes: list[str] | None = None) -> dict[str, object]:
    return {
        "concepto": "Hotel",
        "monto": "100.00",
        "fecha": "2026-11-16",
        "pagador": "ana",
        "regla": "iguales",
        "participantes": participantes or ["ana"],
        "valores": {},
    }


def test_clave_idempotencia_es_obligatoria(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _crear_grupo(escenario)

    respuesta = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json=_gasto(),
        headers=escenario.cabecera("ana"),
    )

    assert respuesta.status_code == status.HTTP_400_BAD_REQUEST


def test_repite_clave_con_mismos_datos_sin_crear_otro_gasto(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _crear_grupo(escenario)
    clave = "22222222-2222-4222-8222-222222222222"
    headers = {
        **escenario.cabecera("ana"),
        "Idempotency-Key": clave,
    }

    primera = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json=_gasto(),
        headers=headers,
    )
    segunda = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json=_gasto(),
        headers=headers,
    )

    assert primera.status_code == status.HTTP_201_CREATED
    assert segunda.status_code == status.HTTP_200_OK
    assert segunda.json()["id"] == primera.json()["id"]

    with escenario.fabrica_db() as db:
        gastos = db.scalars(select(GastoDB).where(GastoDB.id == primera.json()["id"]))
        assert len(gastos.all()) == 1
        assert len(db.scalars(
            select(ParteGastoDB).where(
                ParteGastoDB.gasto_id == primera.json()["id"]
            )
        ).all()) == 1


def test_reutilizar_clave_con_datos_distintos_produce_conflicto(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _crear_grupo(escenario)
    clave = "33333333-3333-4333-8333-333333333333"
    headers = {
        **escenario.cabecera("ana"),
        "Idempotency-Key": clave,
    }

    primera = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json=_gasto(),
        headers=headers,
    )
    datos_distintos = _gasto()
    datos_distintos["concepto"] = "Cena"
    segunda = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json=datos_distintos,
        headers=headers,
    )

    assert primera.status_code == status.HTTP_201_CREATED
    assert segunda.status_code == status.HTTP_409_CONFLICT

    with escenario.fabrica_db() as db:
        assert len(db.scalars(select(GastoDB)).all()) == 1
        assert len(db.scalars(select(EventoCambioDB)).all()) == 1


def test_clave_idempotencia_debe_ser_uuid(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _crear_grupo(escenario)
    headers = {
        **escenario.cabecera("ana"),
        "Idempotency-Key": "clave-invalida",
    }

    respuesta = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json=_gasto(),
        headers=headers,
    )

    assert respuesta.status_code == status.HTTP_400_BAD_REQUEST
