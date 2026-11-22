from datetime import datetime, timezone
from uuid import uuid4

from fastapi import status
from qa.entrega6.escenario import EscenarioSocial, USUARIO_IDS
from sqlalchemy import select

from cuentas_claras.infraestructura.modelos_sociales import (
    CursorCambiosDB,
    EventoCambioDB,
)


def _crear_grupo_con_miembros(
    escenario: EscenarioSocial,
    miembros: tuple[str, ...] = ("beto", "carla"),
) -> str:
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
    grupo_id = respuesta.json()["id"]

    for miembro in miembros:
        amistad = escenario.cliente.post(
            "/amistades",
            json={"usuario": miembro},
            headers=escenario.cabecera("ana"),
        )
        assert amistad.status_code == status.HTTP_201_CREATED

        respuesta_amistad = escenario.cliente.post(
            f"/amistades/{amistad.json()['id']}/respuesta",
            json={"aceptar": True},
            headers=escenario.cabecera(miembro),
        )
        assert respuesta_amistad.status_code == status.HTTP_200_OK

        invitacion = escenario.cliente.post(
            f"/grupos/{grupo_id}/miembros",
            json={"usuario": miembro},
            headers=escenario.cabecera("ana"),
        )
        assert invitacion.status_code == status.HTTP_201_CREATED

        respuesta_invitacion = escenario.cliente.post(
            f"/grupos/{grupo_id}/miembros/yo/respuesta",
            json={"aceptar": True},
            headers=escenario.cabecera(miembro),
        )
        assert respuesta_invitacion.status_code == status.HTTP_200_OK

    return grupo_id


def test_gasto_crea_eventos_solo_para_participantes(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _crear_grupo_con_miembros(escenario)
    respuesta = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json={
            "concepto": "Hotel",
            "monto": "100.01",
            "fecha": "2026-11-16",
            "pagador": "ana",
            "regla": "iguales",
            "participantes": ["ana", "beto"],
            "valores": {},
        },
        headers={
            **escenario.cabecera("ana"),
            "Idempotency-Key": "44444444-4444-4444-8444-444444444444",
        },
    )

    assert respuesta.status_code == status.HTTP_201_CREATED

    cambios_ana = escenario.cliente.get(
        "/cambios",
        params={"desde": 0},
        headers=escenario.cabecera("ana"),
    )
    cambios_beto = escenario.cliente.get(
        "/cambios",
        params={"desde": 0},
        headers=escenario.cabecera("beto"),
    )
    cambios_carla = escenario.cliente.get(
        "/cambios",
        params={"desde": 0},
        headers=escenario.cabecera("carla"),
    )

    assert cambios_ana.status_code == status.HTTP_200_OK
    assert cambios_beto.status_code == status.HTTP_200_OK
    assert cambios_carla.status_code == status.HTTP_200_OK

    evento_ana = cambios_ana.json()["eventos"][0]
    evento_beto = cambios_beto.json()["eventos"][0]

    assert len(cambios_ana.json()["eventos"]) == 1
    assert len(cambios_beto.json()["eventos"]) == 1
    assert cambios_carla.json()["eventos"] == []

    for evento in (evento_ana, evento_beto):
        assert evento["grupo_id"] == grupo_id
        assert evento["gasto_id"] == respuesta.json()["id"]
        assert evento["tipo"] == "GastoDeGrupoRegistrado"
        assert evento["concepto"] == "Hotel"
        assert evento["fecha"] == "2026-11-16T00:00:00Z"
        assert evento["moneda"] == "USD"
        assert set(evento) == {
            "id_evento",
            "tipo",
            "grupo_id",
            "gasto_id",
            "concepto",
            "fecha",
            "moneda",
            "monto_parte",
            "incluye_redondeo",
            "posicion_cursor",
        }


def test_eventos_se_paginan_en_lotes_de_100(
    escenario: EscenarioSocial,
) -> None:
    with escenario.fabrica_db() as db:
        db.add(CursorCambiosDB(usuario_id=USUARIO_IDS["ana"], posicion=101))
        db.flush()
        db.add_all(
            EventoCambioDB(
                id=str(uuid4()),
                usuario_id=USUARIO_IDS["ana"],
                posicion=posicion,
                grupo_id="grupo-prueba",
                gasto_id=f"gasto-{posicion}",
                tipo="GastoDeGrupoRegistrado",
                concepto="Hotel",
                fecha=datetime(2026, 11, 16).date(),
                moneda="USD",
                monto_centimos=1000,
                incluye_redondeo=False,
                creado_en=datetime.now(timezone.utc),
            )
            for posicion in range(1, 102)
        )
        db.commit()

    primer = escenario.cliente.get(
        "/cambios",
        params={"desde": 0},
        headers=escenario.cabecera("ana"),
    )
    segundo = escenario.cliente.get(
        "/cambios",
        params={"desde": primer.json()["nuevo_cursor"]},
        headers=escenario.cabecera("ana"),
    )

    assert primer.status_code == status.HTTP_200_OK
    assert segundo.status_code == status.HTTP_200_OK
    assert len(primer.json()["eventos"]) == 100
    assert primer.json()["nuevo_cursor"] == 100
    assert primer.json()["hay_mas"] is True
    assert len(segundo.json()["eventos"]) == 1
    assert segundo.json()["nuevo_cursor"] == 101
    assert segundo.json()["hay_mas"] is False

    posiciones = [
        evento["posicion_cursor"]
        for evento in primer.json()["eventos"] + segundo.json()["eventos"]
    ]
    assert posiciones == sorted(posiciones)


def test_cursor_negativo_es_rechazado(
    escenario: EscenarioSocial,
) -> None:
    respuesta = escenario.cliente.get(
        "/cambios",
        params={"desde": -1},
        headers=escenario.cabecera("ana"),
    )

    assert respuesta.status_code == status.HTTP_400_BAD_REQUEST
