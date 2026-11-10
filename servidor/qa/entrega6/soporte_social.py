"""Adaptación de pruebas a los DTOs que integra el equipo de contratos.

CUENTAS_CLARAS_QA_SOCIAL_CONTRACT=modulo:formato python -m pytest qa/entrega6

`formato` debe implementar los dos métodos del Protocolo FormatoSocial. Es un
objeto de pruebas: traduce campos JSON, nunca calcula resultados ni inventa IDs,
estados, gastos o balances. Cada llamada HTTP sigue pasando por la app real.

datos recibe operaciones amistad, respuesta_amistad, grupo, miembro,
respuesta_grupo y gasto. Argumentos semánticos: usuario, aceptar, nombre,
descripcion, moneda, actividad; para gasto: concepto, monto (string), fecha,
pagador, regla (iguales/porcentajes/fijos), participantes (lista de usernames),
valores (strings por username para porcentajes/fijos).

leer conserva los datos recibidos y normaliza únicamente nombres/envolturas:
amistad/grupo -> {id}; amistades -> {amigos: [username], enviadas: [username],
recibidas: [username]}; grupos -> {miembros: [id], invitaciones: [id]};
detalle -> {gastos: lista JSON, balances: {username: string decimal},
partes: [{usuario, monto: string decimal, residuo: bool}] del último gasto}.
La búsqueda tiene el esquema exigido directamente por la especificación.
"""

from typing import Any, Protocol

from qa.entrega6.escenario import EscenarioSocial

PROHIBIDOS = {"cuentas", "movimientos", "saldos", "tasas"}


class FormatoSocial(Protocol):
    def datos(self, operacion: str, **valores: Any) -> dict[str, Any]: ...

    def leer(self, operacion: str, respuesta: Any) -> dict[str, Any]: ...


def sin_libro_personal(valor: Any) -> None:
    if isinstance(valor, dict):
        assert PROHIBIDOS.isdisjoint(valor), "La petición contiene libro personal."
        for hijo in valor.values():
            sin_libro_personal(hijo)
    elif isinstance(valor, list):
        for hijo in valor:
            sin_libro_personal(hijo)


class Social:
    def __init__(self, escenario: EscenarioSocial, formato: FormatoSocial):
        self.escenario = escenario
        self.formato = formato

    def enviar(self, actor, metodo, ruta, codigo, operacion=None, **valores):
        datos = None if operacion is None else self.formato.datos(operacion, **valores)
        sin_libro_personal(datos)
        respuesta = self.escenario.cliente.request(
            metodo, ruta, headers=self.escenario.cabecera(actor), json=datos
        )
        assert (
            respuesta.status_code == codigo
        ), f"{metodo} {ruta}: esperado {codigo}, recibido {respuesta.status_code}"
        return respuesta

    def invitar_amigo(self, emisor="ana", destinatario="beto"):
        respuesta = self.enviar(
            emisor, "POST", "/amistades", 201, "amistad", usuario=destinatario
        )
        return str(self.formato.leer("amistad", respuesta.json())["id"])

    def responder_amigo(self, invitacion, usuario="beto", aceptar=True, codigo=200):
        return self.enviar(
            usuario,
            "POST",
            f"/amistades/{invitacion}/respuesta",
            codigo,
            "respuesta_amistad",
            aceptar=aceptar,
        )

    def amistad(self, usuario="beto"):
        invitacion = self.invitar_amigo(destinatario=usuario)
        self.responder_amigo(invitacion, usuario)
        return invitacion

    def grupo(self):
        respuesta = self.enviar(
            "ana",
            "POST",
            "/grupos",
            201,
            "grupo",
            nombre="Viaje a Mérida",
            descripcion="Prueba E6",
            moneda="USD",
            actividad="Noviembre",
        )
        return str(self.formato.leer("grupo", respuesta.json())["id"])

    def invitar_grupo(self, grupo, usuario, emisor="ana", codigo=201):
        return self.enviar(
            emisor,
            "POST",
            f"/grupos/{grupo}/miembros",
            codigo,
            "miembro",
            usuario=usuario,
        )

    def responder_grupo(self, grupo, usuario, aceptar=True, codigo=200):
        return self.enviar(
            usuario,
            "POST",
            f"/grupos/{grupo}/miembros/yo/respuesta",
            codigo,
            "respuesta_grupo",
            aceptar=aceptar,
        )

    def grupo_completo(self):
        self.amistad("beto")
        self.amistad("carla")
        grupo = self.grupo()
        for usuario in ("beto", "carla"):
            self.invitar_grupo(grupo, usuario)
            self.responder_grupo(grupo, usuario)
        return grupo

    def gasto(self, grupo, usuario="ana", codigo=201, **cambios):
        datos = {
            "concepto": "Comida",
            "monto": "100.00",
            "fecha": self.escenario.reloj.ahora().date().isoformat(),
            "pagador": "ana",
            "regla": "iguales",
            "participantes": ["ana", "beto", "carla"],
            "valores": {},
        }
        datos.update(cambios)
        return self.enviar(
            usuario, "POST", f"/grupos/{grupo}/gastos", codigo, "gasto", **datos
        )

    def detalle(self, grupo, usuario="ana"):
        respuesta = self.enviar(usuario, "GET", f"/grupos/{grupo}", 200)
        return self.formato.leer("detalle", respuesta.json())
