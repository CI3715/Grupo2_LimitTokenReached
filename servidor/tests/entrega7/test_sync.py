from uuid import uuid4

from qa.entrega6.escenario import EscenarioSocial


def _crear_grupo_completo(escenario: EscenarioSocial) -> str:
    for amigo in ("beto", "carla"):
        invitacion = escenario.cliente.post(
            "/amistades",
            headers=escenario.cabecera("ana"),
            json={"usuario": amigo},
        )
        assert invitacion.status_code == 201
        respuesta = escenario.cliente.post(
            f"/amistades/{invitacion.json()['id']}/respuesta",
            headers=escenario.cabecera(amigo),
            json={"aceptar": True},
        )
        assert respuesta.status_code == 200

    grupo = escenario.cliente.post(
        "/grupos",
        headers=escenario.cabecera("ana"),
        json={
            "nombre": "Viaje a Mérida",
            "descripcion": "Prueba de sincronización",
            "moneda": "USD",
            "actividad": "Noviembre",
        },
    )
    assert grupo.status_code == 201
    grupo_id = grupo.json()["id"]

    for amigo in ("beto", "carla"):
        invitacion = escenario.cliente.post(
            f"/grupos/{grupo_id}/miembros",
            headers=escenario.cabecera("ana"),
            json={"usuario": amigo},
        )
        assert invitacion.status_code == 201
        respuesta = escenario.cliente.post(
            f"/grupos/{grupo_id}/miembros/yo/respuesta",
            headers=escenario.cabecera(amigo),
            json={"aceptar": True},
        )
        assert respuesta.status_code == 200

    return grupo_id


def _gasto(grupo_id: str, escenario: EscenarioSocial, clave: str | None = None):
    headers = escenario.cabecera("ana")
    if clave is not None:
        headers["Idempotency-Key"] = clave
    return escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        headers=headers,
        json={
            "concepto": "Hotel",
            "monto": "100.00",
            "fecha": "2026-11-20",
            "pagador": "ana",
            "regla": "iguales",
            "participantes": ["ana", "beto", "carla"],
            "valores": {},
        },
    )


def test_cambios_solo_contienen_participantes_y_cursor(
    escenario: EscenarioSocial,
) -> None:
    assert escenario.cliente.get("/cambios").status_code == 401

    grupo_id = _crear_grupo_completo(escenario)
    gasto = _gasto(grupo_id, escenario)
    assert gasto.status_code == 201

    for usuario in ("ana", "beto", "carla"):
        respuesta = escenario.cliente.get(
            "/cambios",
            headers=escenario.cabecera(usuario),
        )
        assert respuesta.status_code == 200
        cuerpo = respuesta.json()
        assert len(cuerpo["eventos"]) == 1
        evento = cuerpo["eventos"][0]
        assert evento["tipo"] == "GastoDeGrupoRegistrado"
        assert evento["grupo_id"] == grupo_id
        assert evento["gasto_id"] == gasto.json()["id"]
        assert evento["moneda"] == "USD"
        assert evento["posicion_cursor"] == 1
        assert cuerpo["nuevo_cursor"] == 1
        assert cuerpo["hay_mas"] is False

    diana = escenario.cliente.get(
        "/cambios",
        headers=escenario.cabecera("diana"),
    )
    assert diana.status_code == 200
    assert diana.json()["eventos"] == []

    assert (
        escenario.cliente.get(
            "/cambios?desde=-1",
            headers=escenario.cabecera("ana"),
        ).status_code
        == 400
    )


def test_repetir_clave_devuelve_gasto_existente_y_datos_distintos_conflictan(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _crear_grupo_completo(escenario)
    clave = str(uuid4())

    primero = _gasto(grupo_id, escenario, clave)
    segundo = _gasto(grupo_id, escenario, clave)

    assert primero.status_code == 201
    assert segundo.status_code == 200
    assert segundo.json() == primero.json()

    detalle = escenario.cliente.get(
        f"/grupos/{grupo_id}",
        headers=escenario.cabecera("ana"),
    )
    assert detalle.status_code == 200
    assert len(detalle.json()["gastos"]) == 1

    headers = escenario.cabecera("ana")
    headers["Idempotency-Key"] = clave
    diferente = escenario.cliente.post(
        f"/grupos/{grupo_id}/gastos",
        headers=headers,
        json={
            "concepto": "Cena",
            "monto": "60.00",
            "fecha": "2026-11-20",
            "pagador": "ana",
            "regla": "iguales",
            "participantes": ["ana", "beto", "carla"],
            "valores": {},
        },
    )
    assert diferente.status_code == 409


def test_cambios_se_entregan_en_lotes_de_100(
    escenario: EscenarioSocial,
) -> None:
    grupo_id = _crear_grupo_completo(escenario)
    headers = escenario.cabecera("ana")

    for numero in range(102):
        respuesta = escenario.cliente.post(
            f"/grupos/{grupo_id}/gastos",
            headers={**headers, "Idempotency-Key": str(uuid4())},
            json={
                "concepto": f"Gasto {numero}",
                "monto": "0.01",
                "fecha": "2026-11-20",
                "pagador": "ana",
                "regla": "iguales",
                "participantes": ["ana", "beto", "carla"],
                "valores": {},
            },
        )
        assert respuesta.status_code == 201

    primer_lote = escenario.cliente.get(
        "/cambios?desde=0",
        headers=escenario.cabecera("ana"),
    )
    assert primer_lote.status_code == 200
    cuerpo = primer_lote.json()
    assert len(cuerpo["eventos"]) == 100
    assert cuerpo["nuevo_cursor"] == 100
    assert cuerpo["hay_mas"] is True

    segundo_lote = escenario.cliente.get(
        "/cambios?desde=100",
        headers=escenario.cabecera("ana"),
    )
    assert segundo_lote.status_code == 200
    cuerpo = segundo_lote.json()
    assert len(cuerpo["eventos"]) == 2
    assert cuerpo["nuevo_cursor"] == 102
    assert cuerpo["hay_mas"] is False
