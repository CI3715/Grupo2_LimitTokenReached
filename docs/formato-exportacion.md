# Especificación del Formato de Exportación (Cuentas Claras)

Este documento describe el formato de texto estructurado utilizado para exportar e importar el libro personal. El formato está diseñado para ser legible por humanos y procesable por herramientas externas (RNF-04).

## 1. Estructura General
El archivo (antes de cifrar) consta de dos secciones: un **Encabezado** de 3 líneas y un **Cuerpo** de eventos, un evento por línea.

### 1.1. Encabezado
Las primeras 3 líneas del archivo contienen metadatos obligatorios:
```text
VERSION=1.0
FECHA_EXPORTACION=2026-11-25T10:00:00Z
TOTAL_EVENTOS=42
```
* **VERSION:** es la versión del formato (actualmente `1.0`).
* **FECHA_EXPORTACION:** timestamp ISO 8601 en UTC de cuando se generó el archivo.
* **TOTAL_EVENTOS:** se refiere al número entero que indica cuántas líneas de eventos siguen a continuación.

### 1.2. Cuerpo (Eventos)
Cada línea subsiguiente representa un evento inmutable. Los campos están separados por el carácter pipe (`|`).

**Sintaxis general:**
```text
TIPO_EVENTO|ID_EVENTO|TIMESTAMP|DEVICE_ID|DATOS_JSON
```

**Descripción de campos:**
* `TIPO_EVENTO`: identificador del tipo (ej. `CREAR_CUENTA`, `REGISTRAR_MOVIMIENTO`, `CREAR_TASA`, `GASTO_GRUPO_APLICADO`).
* `ID_EVENTO`: UUID v4 único e inmutable del evento.
* `TIMESTAMP`: fecha y hora de registro en formato ISO 8601 (ej. `2026-11-25T10:05:00Z`).
* `DEVICE_ID`: identificador único del dispositivo que originó el evento.
* `DATOS_JSON`: objeto JSON serializado con los datos específicos del evento.

**Ejemplo de línea:**
```text
REGISTRAR_MOVIMIENTO|evt_123-abc|2026-11-25T10:05:00Z|dev_001|{"cuenta_id":"acc_1","monto":"-20.00","moneda":"USD","categoria":"Comida"}
```

## 2. Regla Determinista de Cálculo de Saldos (RNF-04)
Para reconstruir los saldos desde cero, una herramienta externa debe aplicar los eventos siguiendo este orden estricto:

1. **Ordenamiento:** ordenar todos los eventos por:
   1. `TIMESTAMP` (ascendente)
   2. `DEVICE_ID` (ascendente, alfabético)
   3. `ID_EVENTO` (ascendente, alfabético)
2. **Aplicación:** Recorrer la lista ordenada y aplicar el efecto de cada evento (sumar ingresos, restar gastos, aplicar tasas).
3. **Filtrado:** Ignorar líneas con `TIPO_EVENTO` desconocido para garantizar compatibilidad hacia adelante.

Al seguir este orden, dos dispositivos con el mismo conjunto de eventos (sin importar el orden en que los recibieron) calcularán exactamente los mismos saldos.
