"""Aceptación HTTP E6; requiere rutas reales y traducción de los DTOs definitivos."""

import json
from decimal import Decimal

import pytest

from qa.entrega6.rutas import ID_INEXISTENTE


def test_busqueda_exacta_no_enumera_ni_filtra_correo(social):
    respuesta = social.enviar("ana", "GET", "/usuarios?usuario=beto", 200)
    assert respuesta.json() == {"nombre_usuario": "beto", "nombre_visible": "Beto"}
    for busqueda in ("bet", "b", "no-registrado-qa"):
        social.enviar("ana", "GET", f"/usuarios?usuario={busqueda}", 404)
    social.enviar("ana", "GET", "/usuarios", 400)
    social.enviar("ana", "GET", "/usuarios?usuario=", 400)


@pytest.mark.parametrize("emisor,destinatario", [("ana", "beto"), ("beto", "ana")])
def test_invitacion_pendiente_no_se_duplica_en_ningun_sentido(
    social, emisor, destinatario
):
    social.invitar_amigo()
    social.enviar(emisor, "POST", "/amistades", 409, "amistad", usuario=destinatario)


def test_invitarse_a_uno_mismo_es_dato_invalido(social):
    social.enviar("ana", "POST", "/amistades", 400, "amistad", usuario="ana")


def test_amistad_solo_nace_tras_aceptacion_y_solo_responde_destinatario(social):
    invitacion = social.invitar_amigo()
    for usuario, lista in (("ana", "enviadas"), ("beto", "recibidas")):
        respuesta = social.enviar(usuario, "GET", "/amistades", 200)
        estado = social.formato.leer("amistades", respuesta.json())
        otro = "beto" if usuario == "ana" else "ana"
        assert otro not in estado["amigos"]
        assert otro in estado[lista]
    for usuario in ("ana", "diana"):
        social.responder_amigo(invitacion, usuario, codigo=403)
    social.responder_amigo(invitacion)
    social.responder_amigo(invitacion, codigo=409)
    for usuario, otro in (("ana", "beto"), ("beto", "ana")):
        respuesta = social.enviar(usuario, "GET", "/amistades", 200)
        assert otro in social.formato.leer("amistades", respuesta.json())["amigos"]
    social.enviar("ana", "POST", "/amistades", 409, "amistad", usuario="beto")


def test_rechazo_cierra_invitacion_sin_crear_amistad(social):
    invitacion = social.invitar_amigo()
    social.responder_amigo(invitacion, aceptar=False)
    social.responder_amigo(invitacion, aceptar=False, codigo=409)
    for usuario in ("ana", "beto"):
        respuesta = social.enviar(usuario, "GET", "/amistades", 200)
        estado = social.formato.leer("amistades", respuesta.json())
        assert estado == {"amigos": [], "enviadas": [], "recibidas": []}


def test_creador_es_miembro_y_administrador_y_aparece_en_sus_grupos(social):
    social.amistad()
    grupo = social.grupo()
    social.detalle(grupo)
    respuesta = social.enviar("ana", "GET", "/grupos", 200)
    assert grupo in map(
        str, social.formato.leer("grupos", respuesta.json())["miembros"]
    )
    social.invitar_grupo(grupo, "beto")
    respuesta = social.enviar("beto", "GET", "/grupos", 200)
    estado = social.formato.leer("grupos", respuesta.json())
    assert grupo not in map(str, estado["miembros"])
    assert grupo in map(str, estado["invitaciones"])


def test_admin_solo_invita_amigos_y_miembro_no_puede_invitar(social):
    social.amistad()
    grupo = social.grupo()
    social.invitar_grupo(grupo, "carla", codigo=403)
    social.invitar_grupo(grupo, "beto")
    social.invitar_grupo(grupo, "beto", codigo=409)
    social.responder_grupo(grupo, "beto")
    social.invitar_grupo(grupo, "beto", codigo=409)
    social.invitar_grupo(grupo, "carla", emisor="beto", codigo=403)


def test_invitado_no_ve_ni_registra_gastos_hasta_aceptar(social):
    social.amistad()
    grupo = social.grupo()
    social.invitar_grupo(grupo, "beto")
    antes = social.enviar("beto", "GET", f"/grupos/{grupo}", 404)
    inexistente = social.enviar("beto", "GET", f"/grupos/{ID_INEXISTENTE}", 404)
    assert antes.json() == inexistente.json()
    social.gasto(grupo, usuario="beto", participantes=["ana"], codigo=404)
    social.responder_grupo(grupo, "beto")
    social.detalle(grupo, "beto")
    social.responder_grupo(grupo, "beto", codigo=409)


def test_rechazar_invitacion_de_grupo_no_concede_acceso(social):
    social.amistad()
    grupo = social.grupo()
    social.invitar_grupo(grupo, "beto")
    social.responder_grupo(grupo, "beto", aceptar=False)
    social.enviar("beto", "GET", f"/grupos/{grupo}", 404)
    social.responder_grupo(grupo, "beto", codigo=409)


@pytest.mark.parametrize(
    "metodo,sufijo,operacion,valores",
    [
        ("GET", "", None, {}),
        ("POST", "/miembros", "miembro", {"usuario": "beto"}),
        ("POST", "/miembros/yo/respuesta", "respuesta_grupo", {"aceptar": True}),
        (
            "POST",
            "/gastos",
            "gasto",
            {
                "concepto": "Privado",
                "monto": "1.00",
                "fecha": "2026-11-10",
                "pagador": "ana",
                "regla": "iguales",
                "participantes": ["ana"],
                "valores": {},
            },
        ),
    ],
)
def test_ajeno_recibe_mismo_404_en_todas_las_rutas_del_grupo(
    social, metodo, sufijo, operacion, valores
):
    grupo = social.grupo_completo()
    real = social.enviar(
        "diana", metodo, f"/grupos/{grupo}{sufijo}", 404, operacion, **valores
    )
    inexistente = social.enviar(
        "diana", metodo, f"/grupos/{ID_INEXISTENTE}{sufijo}", 404, operacion, **valores
    )
    assert real.content == inexistente.content


@pytest.mark.parametrize("quien_elimina", ["ana", "beto"])
def test_eliminar_amistad_no_borra_grupos_ni_gastos(social, quien_elimina):
    amistad = social.amistad()
    respuesta_id = social.enviar(
        quien_elimina, "GET", "/amistades/beto/id", 200
    )
    assert respuesta_id.json()["id"] == amistad
    grupo = social.grupo()
    social.invitar_grupo(grupo, "beto")
    social.responder_grupo(grupo, "beto")
    social.gasto(grupo, participantes=["ana", "beto"])
    antes = social.detalle(grupo)
    social.enviar("diana", "DELETE", f"/amistades/{amistad}", 403)
    respuesta = social.enviar(quien_elimina, "DELETE", f"/amistades/{amistad}", 204)
    assert respuesta.content == b""
    assert social.detalle(grupo, "beto") == antes
    nuevo = social.grupo()
    social.invitar_grupo(nuevo, "beto", codigo=403)


@pytest.mark.parametrize(
    "cambios",
    [{"nombre": ""}, {"nombre": "   "}, {"moneda": "US"}, {"moneda": "USDT"}],
)
def test_grupo_rechaza_datos_invalidos(social, cambios):
    datos = dict(nombre="Viaje", descripcion="", moneda="USD", actividad="Hoy")
    datos.update(cambios)
    social.enviar("ana", "POST", "/grupos", 400, "grupo", **datos)


def _balances(detalle):
    assert all(isinstance(valor, str) for valor in detalle["balances"].values())
    return {usuario: Decimal(valor) for usuario, valor in detalle["balances"].items()}


def test_flujo_exactamente_como_ejemplo_28_y_visibilidad_para_todos(social):
    grupo = social.grupo_completo()
    social.gasto(grupo)
    detalle = social.detalle(grupo)
    assert sum(_balances(detalle).values()) == Decimal("0.00")
    assert sorted(
        (p["usuario"], p["monto"], p["residuo"]) for p in detalle["partes"]
    ) == [("ana", "33.34", True), ("beto", "33.33", False), ("carla", "33.33", False)]
    antes = detalle
    rechazo = social.gasto(
        grupo,
        usuario="beto",
        codigo=400,
        pagador="beto",
        monto="60.00",
        regla="porcentajes",
        valores={"ana": "45", "beto": "25", "carla": "25"},
    )
    assert "95" in rechazo.text and "5" in rechazo.text
    assert "falt" in rechazo.text.casefold()
    assert social.detalle(grupo) == antes  # No debe persistir un gasto inválido.
    social.gasto(
        grupo,
        usuario="beto",
        pagador="beto",
        monto="60.00",
        regla="porcentajes",
        valores={"ana": "50", "beto": "25", "carla": "25"},
    )
    esperado = {
        "ana": Decimal("36.66"),
        "beto": Decimal("11.67"),
        "carla": Decimal("-48.33"),
    }
    for usuario in ("ana", "beto", "carla"):
        actual = social.detalle(grupo, usuario)
        assert _balances(actual) == esperado
        assert sum(_balances(actual).values()) == 0
        assert actual["gastos"] == social.detalle(grupo)["gastos"]


@pytest.mark.parametrize(
    "cambios",
    [
        {"monto": "0.00"},
        {"monto": "-1.00"},
        {"monto": "1.001"},
        {"concepto": ""},
        {"fecha": "no-es-fecha"},
        {"participantes": []},
        {"pagador": "diana"},
        {"participantes": ["ana", "diana"]},
        {"participantes": ["ana", "ana"]},
        {
            "regla": "porcentajes",
            "valores": {"ana": "50", "beto": "50", "carla": "0.01"},
        },
        {"regla": "porcentajes", "valores": {"ana": "-1", "beto": "51", "carla": "50"}},
        {
            "regla": "porcentajes",
            "valores": {"ana": "33.333", "beto": "33.333", "carla": "33.334"},
        },
        {"regla": "fijos", "valores": {"ana": "50", "beto": "40", "carla": "9.99"}},
    ],
)
def test_gasto_invalido_no_modifica_gastos_ni_balances(social, cambios):
    grupo = social.grupo_completo()
    antes = social.detalle(grupo)
    social.gasto(grupo, codigo=400, **cambios)
    assert social.detalle(grupo) == antes


def test_montos_fijos_incorrectos_indican_diferencia_exacta(social):
    grupo = social.grupo_completo()
    respuesta = social.gasto(
        grupo,
        codigo=400,
        monto="120.00",
        regla="fijos",
        valores={"ana": "50.00", "beto": "40.00", "carla": "28.00"},
    )
    texto = respuesta.text.replace(",", ".")
    assert "118" in texto and "2.00" in texto and "falt" in texto.casefold()
    assert social.detalle(grupo)["gastos"] == []


@pytest.mark.parametrize(
    "monto,regla,participantes,valores,esperado",
    [
        (
            "10.00",
            "porcentajes",
            ["carla", "beto", "ana"],
            {"ana": "33.33", "beto": "33.33", "carla": "33.34"},
            [("ana", "3.34", True), ("beto", "3.33", False), ("carla", "3.33", False)],
        ),
        (
            "120.00",
            "fijos",
            ["ana", "beto", "carla"],
            {"ana": "50.00", "beto": "40.00", "carla": "30.00"},
            [
                ("ana", "50.00", False),
                ("beto", "40.00", False),
                ("carla", "30.00", False),
            ],
        ),
        (
            "0.01",
            "iguales",
            ["carla", "ana", "beto"],
            {},
            [("ana", "0.01", True), ("beto", "0.00", False), ("carla", "0.00", False)],
        ),
        (
            "2.00",
            "iguales",
            ["carla", "ana", "beto"],
            {},
            [("ana", "0.67", True), ("beto", "0.67", True), ("carla", "0.66", False)],
        ),
        (
            "90071992547409.93",
            "iguales",
            ["ana"],
            {},
            [("ana", "90071992547409.93", False)],
        ),
    ],
)
def test_servidor_calcula_partes_exactas_residuo_y_balance_cero(
    social, monto, regla, participantes, valores, esperado
):
    grupo = social.grupo_completo()
    social.gasto(
        grupo, monto=monto, regla=regla, participantes=participantes, valores=valores
    )
    detalle = social.detalle(grupo)
    partes = detalle["partes"]
    assert all(isinstance(parte["monto"], str) for parte in partes)
    assert sorted((p["usuario"], p["monto"], p["residuo"]) for p in partes) == esperado
    assert sum(Decimal(p["monto"]) for p in partes) == Decimal(monto)
    balances = _balances(detalle)
    assert sum(balances.values()) == 0
    for parte in partes:
        pagado = Decimal(monto) if parte["usuario"] == "ana" else Decimal(0)
        assert balances[parte["usuario"]] == pagado - Decimal(parte["monto"])


@pytest.mark.parametrize(
    "cambio", [{"cuentas": []}, {"movimientos": []}, {"saldos": []}, {"tasas": []}]
)
def test_api_rechaza_libro_personal_en_gastos(social, cambio):
    grupo = social.grupo_completo()
    datos = social.formato.datos(
        "gasto",
        concepto="Comida",
        monto="1.00",
        fecha="2026-11-10",
        pagador="ana",
        regla="iguales",
        participantes=["ana"],
        valores={},
    )
    # Petición maliciosa directa: la API también debe impedir datos personales.
    respuesta = social.escenario.cliente.post(
        f"/grupos/{grupo}/gastos",
        headers=social.escenario.cabecera("ana"),
        json={**datos, **cambio},
    )
    assert respuesta.status_code == 400
    assert social.detalle(grupo)["gastos"] == []


def test_token_vencido_no_escribe_y_otro_miembro_sigue_viendo_el_grupo(social):
    grupo = social.grupo_completo()
    antes = social.detalle(grupo)
    social.escenario.reloj.avanzar(3600)
    social.gasto(grupo, participantes=["ana"], codigo=401)
    login = social.escenario.cliente.post(
        "/sesiones",
        json={"correo": "beto@example.com", "contrasena": "Clave-exclusiva-QA-123"},
    )
    assert login.status_code == 200
    social.escenario.tokens["beto"] = login.json()["token"]
    assert social.detalle(grupo, "beto") == antes


def test_respuestas_de_grupo_no_filtran_secretos(social):
    grupo = social.grupo_completo()
    texto = json.dumps(social.detalle(grupo), ensure_ascii=False)
    assert "hash_contrasena" not in texto
    assert "Clave-exclusiva-QA-123" not in texto
