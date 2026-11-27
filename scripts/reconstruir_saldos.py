#!/usr/bin/env python3
"""Reconstruccion independiente de saldos desde una exportacion SIN cifrar (RNF-04).

Este script NO importa nada de la app: solo biblioteca estandar y Decimal
(nunca float). Sigue docs/formato-exportacion.md:

  * Encabezado de 3 lineas: VERSION, FECHA_EXPORTACION, TOTAL_EVENTOS.
  * Un evento por linea: TIPO|ID|TIMESTAMP|DEVICE_ID|DATOS_JSON.
  * Orden determinista: TIMESTAMP, luego DEVICE_ID, luego ID_EVENTO.
  * Tipos de evento desconocidos se ignoran (compatibilidad hacia adelante).

Uso:
    python scripts/reconstruir_saldos.py exportacion.txt [--json]

EVENTOS QUE ENTIENDE (nombres y campos reales que guarda la app en la tabla
`eventos`, ver app/crates/infraestructura/src/repositorio_*_sqlite.rs):

  CuentaCreada          datos: id, nombre, moneda_codigo, saldo_inicial
  MovimientoRegistrado  datos: id, cuenta_id, tipo (Ingreso|Gasto), monto (sin signo)
  MovimientoCorregido   datos: movimiento_original_id, cuenta_id, tipo_original,
                        variacion_monto  (efecto: Ingreso +variacion, Gasto -variacion)
  GastoDeGrupoAplicado  datos: id_evento, movimiento_id, grupo_id, gasto_id
                        (marca de deduplicacion; el dinero va en su MovimientoRegistrado)

El documento del formato usa otros nombres de ejemplo; se aceptan como alias:
  CREAR_CUENTA -> CuentaCreada (campos cuenta_id/moneda tambien valen)
  REGISTRAR_MOVIMIENTO -> MovimientoRegistrado (sin campo `tipo`, el monto trae signo)
  GASTO_GRUPO_APLICADO -> GastoDeGrupoAplicado
Todo lo demas (CategoriaCreada, MonedaCreada/Eliminada, TasaRegistrada/Eliminada,
CREAR_TASA...) no cambia el saldo de una cuenta y se ignora.

Deduplicacion de gastos de grupo (E7/E8): si dos dispositivos bajaron la misma parte
(mismo `id_evento`), solo cuenta la primera segun el orden determinista; el movimiento
de las demas se descarta. Hoy cada dispositivo genera un `movimiento_id` distinto
(Uuid::new_v4), asi que SIN esta regla el gasto se sumaria dos veces.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from decimal import Decimal, InvalidOperation
from pathlib import Path

VERSION_SOPORTADA = "1.0"
CAMPOS_ENCABEZADO = ("VERSION", "FECHA_EXPORTACION", "TOTAL_EVENTOS")

ALIAS_TIPOS = {
    "CREAR_CUENTA": "CuentaCreada",
    "REGISTRAR_MOVIMIENTO": "MovimientoRegistrado",
    "GASTO_GRUPO_APLICADO": "GastoDeGrupoAplicado",
}
CUENTA_CREADA = "CuentaCreada"
MOVIMIENTO_REGISTRADO = "MovimientoRegistrado"
MOVIMIENTO_CORREGIDO = "MovimientoCorregido"
GASTO_GRUPO = "GastoDeGrupoAplicado"
MONEDA_DESCONOCIDA = "?"

_PATRON_TS = re.compile(
    r"^(\d{4}-\d{2}-\d{2})T(\d{2}:\d{2}:\d{2})(?:\.(\d{1,9}))?(Z|[+-]\d{2}:\d{2})$"
)


class ErrorFormato(ValueError):
    """La exportacion no cumple el formato; no se debe usar para reconstruir."""


@dataclass(frozen=True)
class Evento:
    tipo: str
    id: str
    timestamp: str
    dispositivo: str
    datos: dict


@dataclass(frozen=True)
class Rechazo:
    id: str
    motivo: str


@dataclass
class Saldo:
    nombre: str
    moneda: str
    monto: Decimal


@dataclass
class Resultado:
    saldos: dict[str, Saldo]
    eventos_unicos: int
    ya_existian: int
    rechazos: list[Rechazo] = field(default_factory=list)
    grupo_duplicados: int = 0


def _tipo(evento: Evento) -> str:
    return ALIAS_TIPOS.get(evento.tipo, evento.tipo)


def _decimal(valor: object, campo: str, evento_id: str) -> Decimal:
    if not isinstance(valor, str):
        # Los montos viajan como texto (INV-03); un numero JSON podria venir de float.
        raise ErrorFormato(f"Evento {evento_id}: '{campo}' debe ser texto, no numero.")
    try:
        numero = Decimal(valor.strip())
    except InvalidOperation as error:
        raise ErrorFormato(f"Evento {evento_id}: '{campo}' no es un decimal valido.") from error
    if not numero.is_finite():
        raise ErrorFormato(f"Evento {evento_id}: '{campo}' no es finito.")
    return numero


def _texto(datos: dict, claves: tuple[str, ...], evento_id: str) -> str:
    for clave in claves:
        if clave in datos and datos[clave] not in (None, ""):
            return str(datos[clave])
    raise ErrorFormato(f"Evento {evento_id}: falta el campo '{claves[0]}'.")


def instante(texto: str) -> tuple[datetime, int]:
    """(segundo UTC, nanosegundos). La app escribe nanosegundos (chrono::to_rfc3339)."""
    coincidencia = _PATRON_TS.match(texto)
    if coincidencia is None:
        raise ErrorFormato(f"TIMESTAMP invalido '{texto}'.")
    fecha, hora, fraccion, zona = coincidencia.groups()
    try:
        base = datetime.fromisoformat(f"{fecha}T{hora}")
        if zona != "Z":
            signo = 1 if zona[0] == "+" else -1
            horas, minutos = int(zona[1:3]), int(zona[4:6])
            base -= signo * timedelta(hours=horas, minutes=minutos)
    except ValueError as error:
        raise ErrorFormato(f"TIMESTAMP invalido '{texto}'.") from error
    nanos = int((fraccion or "").ljust(9, "0")) if fraccion else 0
    return base.replace(tzinfo=timezone.utc), nanos


def clave_orden(evento: Evento) -> tuple[datetime, int, str, str]:
    segundo, nanos = instante(evento.timestamp)
    return (segundo, nanos, evento.dispositivo, evento.id)


def parsear_encabezado(lineas: list[str]) -> int:
    """Valida las 3 primeras lineas y devuelve TOTAL_EVENTOS."""
    if len(lineas) < 3:
        raise ErrorFormato("Encabezado incompleto: se esperan 3 lineas.")
    valores: dict[str, str] = {}
    for numero, campo in enumerate(CAMPOS_ENCABEZADO, start=1):
        clave, separador, valor = lineas[numero - 1].partition("=")
        if separador != "=" or clave != campo:
            raise ErrorFormato(f"Linea {numero}: se esperaba '{campo}=...'.")
        valores[campo] = valor.strip()
    if valores["VERSION"] != VERSION_SOPORTADA:
        raise ErrorFormato(f"Version de formato no soportada: {valores['VERSION']}.")
    try:
        total = int(valores["TOTAL_EVENTOS"])
    except ValueError as error:
        raise ErrorFormato("TOTAL_EVENTOS debe ser un entero.") from error
    if total < 0:
        raise ErrorFormato("TOTAL_EVENTOS no puede ser negativo.")
    return total


def parsear_evento(linea: str, numero: int) -> Evento:
    # maxsplit=4: el JSON del ultimo campo puede contener '|'.
    partes = linea.split("|", 4)
    if len(partes) != 5:
        raise ErrorFormato(f"Linea {numero}: se esperan 5 campos separados por '|'.")
    tipo, id_evento, timestamp, dispositivo, datos_json = partes
    if not (tipo and id_evento and timestamp and dispositivo):
        raise ErrorFormato(f"Linea {numero}: hay campos vacios.")
    try:
        instante(timestamp)
    except ErrorFormato as error:
        raise ErrorFormato(f"Linea {numero}: {error}") from error
    try:
        datos = json.loads(datos_json)
    except json.JSONDecodeError as error:
        raise ErrorFormato(f"Linea {numero}: DATOS_JSON no es JSON valido.") from error
    if not isinstance(datos, dict):
        raise ErrorFormato(f"Linea {numero}: DATOS_JSON debe ser un objeto.")
    return Evento(tipo, id_evento, timestamp, dispositivo, datos)


def leer_exportacion(texto: str) -> tuple[list[Evento], list[Rechazo], int]:
    """Devuelve (eventos unicos, rechazados, cuantos ya existian)."""
    lineas = [linea for linea in texto.splitlines() if linea.strip()]
    total = parsear_encabezado(lineas)
    cuerpo = lineas[3:]
    if len(cuerpo) != total:
        raise ErrorFormato(
            f"TOTAL_EVENTOS={total} pero el archivo trae {len(cuerpo)} lineas de eventos."
        )
    por_id: dict[str, Evento] = {}
    rechazos: list[Rechazo] = []
    repetidos = 0
    for indice, linea in enumerate(cuerpo):
        evento = parsear_evento(linea, indice + 4)
        previo = por_id.get(evento.id)
        if previo is None:
            por_id[evento.id] = evento
        elif previo == evento:
            repetidos += 1  # mismo ID y mismo contenido: cuenta una sola vez
        else:
            rechazos.append(Rechazo(evento.id, "ID repetido con contenido distinto"))
    return list(por_id.values()), rechazos, repetidos


def _movimientos_de_grupo_descartados(ordenados: list[Evento]) -> tuple[set[str], int]:
    """Ids de movimiento a descartar: partes de grupo ya aplicadas por otro evento."""
    vistos: set[str] = set()
    descartados: set[str] = set()
    for evento in ordenados:
        if _tipo(evento) != GASTO_GRUPO:
            continue
        id_parte = _texto(evento.datos, ("id_evento",), evento.id)
        if id_parte in vistos:
            movimiento = evento.datos.get("movimiento_id")
            if movimiento:
                descartados.add(str(movimiento))
        else:
            vistos.add(id_parte)
    return descartados, len(descartados)


def _efecto_movimiento(datos: dict, evento_id: str) -> Decimal:
    monto = _decimal(datos.get("monto"), "monto", evento_id)
    tipo = datos.get("tipo")
    if tipo is None:
        return monto  # estilo del documento: el monto ya trae signo
    if tipo == "Ingreso":
        return monto
    if tipo == "Gasto":
        return -monto
    raise ErrorFormato(f"Evento {evento_id}: tipo de movimiento invalido '{tipo}'.")


def _efecto_correccion(datos: dict, evento_id: str) -> Decimal:
    variacion = _decimal(datos.get("variacion_monto"), "variacion_monto", evento_id)
    tipo = datos.get("tipo_original")
    if tipo == "Ingreso":
        return variacion
    if tipo == "Gasto":
        return -variacion
    raise ErrorFormato(f"Evento {evento_id}: tipo_original invalido '{tipo}'.")


def reconstruir_saldos(eventos: list[Evento]) -> tuple[dict[str, Saldo], int]:
    ordenados = sorted(eventos, key=clave_orden)
    descartados, grupo_duplicados = _movimientos_de_grupo_descartados(ordenados)
    saldos: dict[str, Saldo] = {}

    # Las cuentas se crean primero: un reloj desfasado no debe perder el saldo inicial.
    for evento in ordenados:
        if _tipo(evento) != CUENTA_CREADA:
            continue
        datos = evento.datos
        cuenta = _texto(datos, ("id", "cuenta_id"), evento.id)
        moneda = _texto(datos, ("moneda_codigo", "moneda"), evento.id)
        inicial = _decimal(datos.get("saldo_inicial", "0"), "saldo_inicial", evento.id)
        nombre = str(datos.get("nombre") or cuenta)
        saldos[cuenta] = Saldo(nombre, moneda, inicial)

    for evento in ordenados:
        tipo = _tipo(evento)
        datos = evento.datos
        if tipo == MOVIMIENTO_REGISTRADO:
            if str(datos.get("id")) in descartados:
                continue
            efecto = _efecto_movimiento(datos, evento.id)
        elif tipo == MOVIMIENTO_CORREGIDO:
            efecto = _efecto_correccion(datos, evento.id)
        elif tipo == GASTO_GRUPO and "cuenta_id" in datos and "monto" in datos:
            efecto = _efecto_movimiento(datos, evento.id)  # variante con dinero propio
        else:
            continue  # CuentaCreada ya aplicada; el resto no mueve saldos
        cuenta = _texto(datos, ("cuenta_id",), evento.id)
        moneda = str(datos.get("moneda") or datos.get("moneda_codigo") or "")
        saldo = saldos.setdefault(
            cuenta, Saldo(cuenta, moneda or MONEDA_DESCONOCIDA, Decimal(0))
        )
        if moneda and saldo.moneda != MONEDA_DESCONOCIDA and saldo.moneda != moneda:
            raise ErrorFormato(f"Evento {evento.id}: moneda {moneda} distinta de la cuenta.")
        saldo.monto += efecto
    return saldos, grupo_duplicados


def reconstruir_desde_texto(texto: str) -> Resultado:
    eventos, rechazos, repetidos = leer_exportacion(texto)
    saldos, grupo_duplicados = reconstruir_saldos(eventos)
    return Resultado(saldos, len(eventos), repetidos, rechazos, grupo_duplicados)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Reconstruye saldos desde una exportacion sin cifrar.")
    parser.add_argument("archivo", type=Path)
    parser.add_argument("--json", action="store_true", help="salida en JSON")
    args = parser.parse_args(argv)
    try:
        resultado = reconstruir_desde_texto(args.archivo.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, ErrorFormato) as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1

    if args.json:
        salida = {
            "eventos": resultado.eventos_unicos,
            "ya_existian": resultado.ya_existian,
            "grupo_duplicados": resultado.grupo_duplicados,
            "rechazados": [{"id": r.id, "motivo": r.motivo} for r in resultado.rechazos],
            "saldos": {
                cuenta: {"nombre": s.nombre, "moneda": s.moneda, "saldo": f"{s.monto:f}"}
                for cuenta, s in sorted(resultado.saldos.items())
            },
        }
        print(json.dumps(salida, ensure_ascii=False, indent=2))
    else:
        print(
            f"Eventos unicos: {resultado.eventos_unicos} | repetidos: {resultado.ya_existian}"
            f" | rechazados: {len(resultado.rechazos)} | gastos de grupo duplicados: {resultado.grupo_duplicados}"
        )
        for cuenta, s in sorted(resultado.saldos.items()):
            print(f"{cuenta}\t{s.nombre}\t{s.moneda}\t{s.monto:f}")
        for r in resultado.rechazos:
            print(f"RECHAZADO {r.id}: {r.motivo}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
