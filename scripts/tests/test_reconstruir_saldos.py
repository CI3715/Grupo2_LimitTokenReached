"""Pruebas del script independiente de reconstruccion (RNF-04).

Se ejecutan con: python -m unittest discover -s scripts/tests -v
(o con pytest). No importan nada de la app.

Los eventos de prueba usan los nombres y campos REALES que guarda la app en la
tabla `eventos` (CuentaCreada, MovimientoRegistrado, MovimientoCorregido,
GastoDeGrupoAplicado), con timestamps RFC 3339 como los escribe chrono.
"""

from __future__ import annotations

import contextlib
import io
import json
import random
import sys
import tempfile
import unittest
from decimal import Decimal
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import reconstruir_saldos as rs  # noqa: E402

CUENTA = "acc_efectivo"


def ev(tipo: str, id_evento: str, ts: str, dev: str, datos: dict) -> str:
    return f"{tipo}|{id_evento}|{ts}|{dev}|{json.dumps(datos, separators=(',', ':'))}"


def cuenta(id_evento: str, ts: str, dev: str, inicial: str = "100.00") -> str:
    return ev(
        "CuentaCreada",
        id_evento,
        ts,
        dev,
        {"id": CUENTA, "nombre": "Efectivo", "tipo": "Efectivo", "moneda_codigo": "USD",
         "saldo_inicial": inicial, "creada_en": "2026-11-01"},
    )


def mov(id_evento: str, ts: str, dev: str, tipo: str, monto: str, mov_id: str | None = None) -> str:
    return ev(
        "MovimientoRegistrado",
        id_evento,
        ts,
        dev,
        {"id": mov_id or f"m_{id_evento}", "cuenta_id": CUENTA, "tipo": tipo, "monto": monto,
         "fecha": "2026-11-10", "categoria_id": "cat", "nota": None},
    )


def exportacion(lineas: list[str], total: int | None = None, version: str = "1.0") -> str:
    n = len(lineas) if total is None else total
    cabecera = [f"VERSION={version}", "FECHA_EXPORTACION=2026-11-25T10:00:00Z", f"TOTAL_EVENTOS={n}"]
    return "\n".join(cabecera + lineas) + "\n"


def base_40() -> list[str]:
    """Escenario 2.6: 40 eventos que dejan Efectivo USD = 151,67."""
    eventos = [cuenta("evt_000", "2026-11-01T08:00:00.000000000+00:00", "dev_laptop")]
    for i in range(1, 39):  # 38 ingresos de 1,00
        eventos.append(mov(f"evt_{i:03d}", f"2026-11-01T09:{i:02d}:00.000000000+00:00", "dev_laptop", "Ingreso", "1.00"))
    eventos.append(mov("evt_039", "2026-11-01T10:00:00.000000000+00:00", "dev_laptop", "Ingreso", "13.67"))
    return eventos  # 1 + 38 + 1 = 40


LAPTOP = [
    mov("evt_lap_1", "2026-11-10T09:00:00.100000000+00:00", "dev_laptop", "Gasto", "20.00"),
    mov("evt_lap_2", "2026-11-10T09:05:00.100000000+00:00", "dev_laptop", "Gasto", "5.00"),
]
TELEFONO = [
    mov("evt_tel_1", "2026-11-10T09:02:00.200000000+00:00", "dev_telefono", "Ingreso", "50.00"),
    mov("evt_tel_2", "2026-11-10T09:07:00.200000000+00:00", "dev_telefono", "Gasto", "10.00"),
]


def resultado(lineas: list[str]) -> rs.Resultado:
    return rs.reconstruir_desde_texto(exportacion(lineas))


def saldo(lineas: list[str]) -> Decimal:
    return resultado(lineas).saldos[CUENTA].monto


class Escenario26(unittest.TestCase):
    def test_base_tiene_40_eventos_y_151_67(self):
        r = resultado(base_40())
        self.assertEqual(r.eventos_unicos, 40)
        self.assertEqual(r.saldos[CUENTA].monto, Decimal("151.67"))
        self.assertEqual((r.saldos[CUENTA].nombre, r.saldos[CUENTA].moneda), ("Efectivo", "USD"))

    def test_saldos_parciales_de_cada_dispositivo(self):
        self.assertEqual(saldo(base_40() + LAPTOP), Decimal("126.67"))
        self.assertEqual(saldo(base_40() + TELEFONO), Decimal("191.67"))

    def test_converge_a_44_eventos_y_166_67(self):
        r = resultado(base_40() + LAPTOP + TELEFONO)
        self.assertEqual(r.eventos_unicos, 44)
        self.assertEqual(r.saldos[CUENTA].monto, Decimal("166.67"))

    def test_el_orden_de_llegada_no_importa(self):
        todos = base_40() + LAPTOP + TELEFONO
        esperado = saldo(todos)
        generador = random.Random(2026)
        for _ in range(50):
            mezclados = todos[:]
            generador.shuffle(mezclados)
            self.assertEqual(saldo(mezclados), esperado)

    def test_laptop_luego_telefono_igual_a_telefono_luego_laptop(self):
        self.assertEqual(
            saldo(base_40() + LAPTOP + TELEFONO), saldo(base_40() + TELEFONO + LAPTOP)
        )


class OrdenYDuplicados(unittest.TestCase):
    def test_orden_timestamp_dispositivo_id(self):
        t = "2026-11-10T09:00:00.000000000+00:00"
        e1 = rs.parsear_evento(mov("b", t, "dev_a", "Ingreso", "1"), 1)
        e2 = rs.parsear_evento(mov("a", t, "dev_a", "Ingreso", "1"), 2)
        e3 = rs.parsear_evento(mov("z", t, "dev_0", "Ingreso", "1"), 3)
        e4 = rs.parsear_evento(mov("y", "2026-11-09T09:00:00.000000000+00:00", "dev_z", "Ingreso", "1"), 4)
        self.assertEqual([e.id for e in sorted([e1, e2, e3, e4], key=rs.clave_orden)], ["y", "z", "a", "b"])

    def test_los_nanosegundos_desempatan_antes_que_el_dispositivo(self):
        antes = rs.parsear_evento(mov("x", "2026-11-10T09:00:00.000000001+00:00", "dev_z", "Ingreso", "1"), 1)
        despues = rs.parsear_evento(mov("y", "2026-11-10T09:00:00.000000002+00:00", "dev_a", "Ingreso", "1"), 2)
        self.assertEqual([e.id for e in sorted([despues, antes], key=rs.clave_orden)], ["x", "y"])

    def test_zona_horaria_z_y_offset_cero_son_el_mismo_instante(self):
        self.assertEqual(rs.instante("2026-11-10T09:00:00Z"), rs.instante("2026-11-10T09:00:00.000000000+00:00"))
        self.assertEqual(rs.instante("2026-11-10T09:00:00-04:00"), rs.instante("2026-11-10T13:00:00Z"))

    def test_id_repetido_igual_cuenta_una_vez(self):
        r = resultado(base_40() + [LAPTOP[0], LAPTOP[0]])
        self.assertEqual((r.eventos_unicos, r.ya_existian, r.rechazos), (41, 1, []))
        self.assertEqual(r.saldos[CUENTA].monto, Decimal("131.67"))

    def test_id_repetido_con_contenido_distinto_se_rechaza_y_el_resto_se_aplica(self):
        alterado = mov("evt_lap_1", "2026-11-10T09:00:00.100000000+00:00", "dev_laptop", "Gasto", "999.00")
        r = resultado(base_40() + [LAPTOP[0], alterado, LAPTOP[1]])
        self.assertEqual([x.id for x in r.rechazos], ["evt_lap_1"])
        self.assertEqual(r.eventos_unicos, 42)
        self.assertEqual(r.saldos[CUENTA].monto, Decimal("126.67"))


class TiposDeEvento(unittest.TestCase):
    def test_correccion_de_gasto(self):
        # Gasto de 20,00 corregido a 15,00: variacion -5,00; el saldo sube 5,00.
        correccion = ev(
            "MovimientoCorregido", "evt_c1", "2026-11-11T00:00:00.000000000+00:00", "dev_laptop",
            {"id": "aj1", "movimiento_original_id": "m_evt_lap_1", "cuenta_id": CUENTA,
             "tipo_original": "Gasto", "variacion_monto": "-5.00"},
        )
        self.assertEqual(saldo(base_40() + [LAPTOP[0], correccion]), Decimal("136.67"))

    def test_correccion_de_ingreso(self):
        correccion = ev(
            "MovimientoCorregido", "evt_c2", "2026-11-11T00:00:00.000000000+00:00", "dev_laptop",
            {"id": "aj2", "movimiento_original_id": "m_evt_tel_1", "cuenta_id": CUENTA,
             "tipo_original": "Ingreso", "variacion_monto": "-30.00"},
        )
        self.assertEqual(saldo(base_40() + [TELEFONO[0], correccion]), Decimal("171.67"))

    def test_anular_un_movimiento_con_correccion_a_cero(self):
        correccion = ev(
            "MovimientoCorregido", "evt_c3", "2026-11-11T00:00:00.000000000+00:00", "dev_laptop",
            {"id": "aj3", "movimiento_original_id": "m_evt_lap_1", "cuenta_id": CUENTA,
             "tipo_original": "Gasto", "variacion_monto": "-20.00"},
        )
        self.assertEqual(saldo(base_40() + [LAPTOP[0], correccion]), Decimal("151.67"))

    def test_eventos_que_no_mueven_saldos_se_ignoran(self):
        otros = [
            ev("CategoriaCreada", "e1", "2026-11-12T00:00:00.000000000+00:00", "d", {"id": "c", "nombre": "X"}),
            ev("MonedaCreada", "e2", "2026-11-12T00:00:01.000000000+00:00", "d", {"codigo": "COP"}),
            ev("MonedaEliminada", "e3", "2026-11-12T00:00:02.000000000+00:00", "d", {"codigo": "COP"}),
            ev("TasaRegistrada", "e4", "2026-11-12T00:00:03.000000000+00:00", "d",
               {"tasa_id": "1", "moneda_origen": "USD", "moneda_destino": "VES", "valor": "36.5"}),
            ev("TasaEliminada", "e5", "2026-11-12T00:00:04.000000000+00:00", "d", {"tasa_id": "1"}),
            ev("TIPO_DEL_FUTURO", "e6", "2026-11-12T00:00:05.000000000+00:00", "d", {"a": 1}),
        ]
        self.assertEqual(saldo(base_40() + otros), Decimal("151.67"))

    def test_alias_del_documento_de_formato(self):
        lineas = [
            ev("CREAR_CUENTA", "c", "2026-11-01T08:00:00Z", "dev_001",
               {"cuenta_id": CUENTA, "nombre": "Efectivo", "moneda": "USD", "saldo_inicial": "100.00"}),
            ev("REGISTRAR_MOVIMIENTO", "evt_123-abc", "2026-11-25T10:05:00Z", "dev_001",
               {"cuenta_id": CUENTA, "monto": "-20.00", "moneda": "USD", "categoria": "Comida"}),
        ]
        self.assertEqual(saldo(lineas), Decimal("80.00"))

    def test_json_con_pipe_no_rompe_el_parseo(self):
        evento = ev(
            "MovimientoRegistrado", "evt_p", "2026-11-12T00:00:00.000000000+00:00", "dev_laptop",
            {"id": "m_p", "cuenta_id": CUENTA, "tipo": "Gasto", "monto": "1.00", "nota": "a|b|c"},
        )
        self.assertEqual(saldo(base_40() + [evento]), Decimal("150.67"))

    def test_decimal_exacto_sin_float(self):
        lineas = [cuenta("c", "2026-01-01T00:00:00.000000000+00:00", "d", "0")] + [
            mov(f"m{i}", f"2026-01-01T00:0{i}:00.000000000+00:00", "d", "Ingreso", "0.1") for i in range(3)
        ]
        self.assertEqual(saldo(lineas), Decimal("0.3"))  # con float seria 0.30000000000000004

    def test_la_cuenta_se_crea_primero_aunque_su_reloj_este_atrasado(self):
        # El movimiento parece anterior a la cuenta (reloj desfasado): el saldo inicial no se pierde.
        lineas = [
            mov("m1", "2026-01-01T00:00:00.000000000+00:00", "dev_telefono", "Gasto", "10.00"),
            cuenta("c", "2026-01-02T00:00:00.000000000+00:00", "dev_laptop", "100.00"),
        ]
        self.assertEqual(saldo(lineas), Decimal("90.00"))

    def test_movimiento_de_cuenta_desconocida_queda_con_moneda_desconocida(self):
        r = resultado([mov("m1", "2026-01-01T00:00:00.000000000+00:00", "d", "Ingreso", "5.00")])
        self.assertEqual(r.saldos[CUENTA].moneda, rs.MONEDA_DESCONOCIDA)
        self.assertEqual(r.saldos[CUENTA].monto, Decimal("5.00"))


class GastosDeGrupo(unittest.TestCase):
    @staticmethod
    def parte(id_evento: str, ts: str, dev: str, movimiento_id: str) -> list[str]:
        return [
            ev("MovimientoRegistrado", f"mv_{movimiento_id}", ts, dev,
               {"id": movimiento_id, "cuenta_id": CUENTA, "tipo": "Gasto", "monto": "8.00", "nota": "Cena"}),
            ev("GastoDeGrupoAplicado", f"ga_{movimiento_id}", ts, dev,
               {"id_evento": id_evento, "movimiento_id": movimiento_id, "grupo_id": "g1", "gasto_id": "gasto1"}),
        ]

    def test_misma_parte_bajada_en_dos_dispositivos_cuenta_una_sola_vez(self):
        lineas = (
            base_40()
            + self.parte("srv_77", "2026-11-10T10:00:00.000000000+00:00", "dev_laptop", "mov_A")
            + self.parte("srv_77", "2026-11-10T10:00:05.000000000+00:00", "dev_telefono", "mov_B")
        )
        r = resultado(lineas)
        self.assertEqual(r.saldos[CUENTA].monto, Decimal("143.67"))  # 151,67 - 8,00 una vez
        self.assertEqual(r.grupo_duplicados, 1)

    def test_la_deduplicacion_no_depende_del_orden_de_llegada(self):
        lineas = (
            base_40()
            + self.parte("srv_77", "2026-11-10T10:00:00.000000000+00:00", "dev_laptop", "mov_A")
            + self.parte("srv_77", "2026-11-10T10:00:05.000000000+00:00", "dev_telefono", "mov_B")
        )
        generador = random.Random(7)
        for _ in range(30):
            mezclados = lineas[:]
            generador.shuffle(mezclados)
            self.assertEqual(saldo(mezclados), Decimal("143.67"))

    def test_partes_distintas_si_se_suman_las_dos(self):
        lineas = (
            base_40()
            + self.parte("srv_77", "2026-11-10T10:00:00.000000000+00:00", "dev_laptop", "mov_A")
            + self.parte("srv_78", "2026-11-10T10:00:05.000000000+00:00", "dev_laptop", "mov_B")
        )
        r = resultado(lineas)
        self.assertEqual(r.saldos[CUENTA].monto, Decimal("135.67"))
        self.assertEqual(r.grupo_duplicados, 0)


class Validacion(unittest.TestCase):
    def test_version_no_soportada(self):
        with self.assertRaises(rs.ErrorFormato):
            rs.reconstruir_desde_texto(exportacion(base_40(), version="2.0"))

    def test_total_eventos_no_coincide(self):
        with self.assertRaises(rs.ErrorFormato):
            rs.reconstruir_desde_texto(exportacion(base_40(), total=41))

    def test_encabezado_incompleto(self):
        with self.assertRaises(rs.ErrorFormato):
            rs.reconstruir_desde_texto("VERSION=1.0\n")

    def test_linea_con_campos_de_menos(self):
        with self.assertRaises(rs.ErrorFormato):
            rs.reconstruir_desde_texto(exportacion(["MovimientoRegistrado|solo|tres"]))

    def test_json_invalido(self):
        malo = "MovimientoRegistrado|e|2026-11-12T00:00:00Z|d|{no es json"
        with self.assertRaises(rs.ErrorFormato):
            rs.reconstruir_desde_texto(exportacion([malo]))

    def test_timestamp_invalido(self):
        malo = ev("MovimientoRegistrado", "e", "ayer", "d", {})
        with self.assertRaises(rs.ErrorFormato):
            rs.reconstruir_desde_texto(exportacion([malo]))

    def test_monto_numerico_en_json_se_rechaza(self):
        malo = ev("MovimientoRegistrado", "e", "2026-11-12T00:00:00Z", "d",
                  {"id": "m", "cuenta_id": CUENTA, "tipo": "Gasto", "monto": 1.5})
        with self.assertRaises(rs.ErrorFormato):
            rs.reconstruir_desde_texto(exportacion([malo]))

    def test_tipo_de_movimiento_invalido(self):
        malo = ev("MovimientoRegistrado", "e", "2026-11-12T00:00:00Z", "d",
                  {"id": "m", "cuenta_id": CUENTA, "tipo": "Regalo", "monto": "1.00"})
        with self.assertRaises(rs.ErrorFormato):
            rs.reconstruir_desde_texto(exportacion([malo]))

    def test_moneda_distinta_de_la_cuenta(self):
        malo = ev("MovimientoRegistrado", "e", "2026-11-12T00:00:00Z", "d",
                  {"id": "m", "cuenta_id": CUENTA, "moneda": "EUR", "tipo": "Gasto", "monto": "1.00"})
        with self.assertRaises(rs.ErrorFormato):
            rs.reconstruir_desde_texto(exportacion(base_40() + [malo]))


class LineaDeComandos(unittest.TestCase):
    def test_salida_y_codigo_de_retorno(self):
        with tempfile.TemporaryDirectory() as carpeta:
            ruta = Path(carpeta) / "export.txt"
            ruta.write_text(exportacion(base_40() + LAPTOP + TELEFONO), encoding="utf-8")
            salida = io.StringIO()
            with contextlib.redirect_stdout(salida):
                codigo = rs.main([str(ruta), "--json"])
            datos = json.loads(salida.getvalue())
        self.assertEqual(codigo, 0)
        self.assertEqual(datos["eventos"], 44)
        self.assertEqual(datos["saldos"][CUENTA]["saldo"], "166.67")

    def test_archivo_invalido_devuelve_1(self):
        with tempfile.TemporaryDirectory() as carpeta:
            ruta = Path(carpeta) / "malo.txt"
            ruta.write_text("esto no es una exportacion", encoding="utf-8")
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(rs.main([str(ruta)]), 1)

    def test_archivo_inexistente_devuelve_1(self):
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(rs.main(["no_existe_12345.txt"]), 1)


if __name__ == "__main__":
    unittest.main()
