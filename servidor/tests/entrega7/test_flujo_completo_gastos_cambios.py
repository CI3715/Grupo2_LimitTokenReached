from fastapi import status
from qa.entrega6.escenario import EscenarioSocial


def test_flujo_completo_gasto_idempotente_y_cambios(
    escenario: EscenarioSocial,
) -> None:
    for miembro in ("beto", "carla"):
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

    grupo = escenario.cliente.post(
        "/grupos",
        json={
            "nombre": "Viaje completo",
            "descripcion": "Prueba integral",
            "moneda": "USD",
            "actividad": "Viaje",
        },
        headers=escenario.cabecera("ana"),
    )
    assert grupo.status_code == status.HTTP_201_CREATED
    grupo_id = grupo.json()["id"]

    for miembro in ("beto", "carla"):
        invitacion = escenario.cliente.post(
            f"/grupos/{grupo_id}/miembros",
            json={"usuario": miembro},
            headers=escenario.cabecera("ana"),
        )
        assert invitacion.status_code == status.HTTP_201_CREATED

        respuesta = escenario.cliente.post(
            f"/grupos/{grupo_id}/miembros/yo/respuesta",
            json={"aceptar": True},
            headers=escenario.cabecera(miembro),
        )
        assert respuesta.status_code == status.HTTP_200_OK

    datos_gasto = {
        "concepto": "Hotel",
        "monto": "100.01",
        "fecha": "2026-11-16",
        "pagador": "ana",
        "regla": "iguales",
        "participantes": ["ana", "beto"],
        "valores": {},
    }
    clave = "55555555-5555-4555-8555-555555555555"
    headers = {
        **escenario.cabecera("ana"),
        "Idempotency-Key": clave,
    }

    primero = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json=datos_gasto,
        headers=headers,
    )
    assert primero.status_code == status.HTTP_201_CREATED

    segundo = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json=datos_gasto,
        headers=headers,
    )
    assert segundo.status_code == status.HTTP_200_OK
    assert segundo.json()["id"] == primero.json()["id"]

    datos_distintos = {**datos_gasto, "concepto": "Cena"}
    conflicto = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        json=datos_distintos,
        headers=headers,
    )
    assert conflicto.status_code == status.HTTP_409_CONFLICT

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
    assert len(cambios_ana.json()["eventos"]) == 1
    assert len(cambios_beto.json()["eventos"]) == 1
    assert cambios_carla.json()["eventos"] == []

    evento_ana = cambios_ana.json()["eventos"][0]
    assert evento_ana["grupo_id"] == grupo_id
    assert evento_ana["gasto_id"] == primero.json()["id"]
    assert evento_ana["tipo"] == "GastoDeGrupoRegistrado"
    assert evento_ana["monto_parte"] == "50.01"
