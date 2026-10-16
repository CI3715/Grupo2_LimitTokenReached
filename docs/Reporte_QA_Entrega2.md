# Reporte de Aseguramiento de Calidad (QA) - Entrega 2

**Proyecto:** Cuentas Claras (Grupo 2 LimitTokenReached — Sección 1)  
**Curso:** CI3715  
**Fecha:** 15 de octubre de 2026  
**Rama de referencia:** `main` 

---

## 1. Resumen Ejecutivo
Se ha validado el cumplimiento de los criterios de aceptación de la Entrega 2, con enfoque en la arquitectura **Local-First** y la integridad del dominio financiero. La aplicación permite la creación y persistencia de cuentas sin conexión a internet, respetando los invariantes de dominio (INV-01, INV-03, INV-18) y la arquitectura hexagonal. Todas las pruebas de negocio se ejecutan de forma aislada utilizando un repositorio en memoria, garantizando velocidad y ausencia de acoplamiento con infraestructura externa.

---

## 2. Evidencia de Pruebas Automatizadas (Invariantes y Validaciones)
De acuerdo con el **ADR-009**, las pruebas de dominio se ejecutan utilizando un **repositorio en memoria (Fake)**, sin depender de SQLite ni de la red. Esto cumple con el criterio de la rúbrica: *"pruebas automatizadas para INV-01, INV-03 e INV-18 y para cada validación de CrearCuenta, ejecutadas con el repositorio en memoria"*.

### 2.1. Invariantes Validados
- **INV-01 (Monto como par indivisible):** se verifica que todo request de creación de cuenta exija explícitamente `moneda_codigo` y `saldo_inicial`.
- **INV-03 (Prohibición de coma flotante):** se valida en tiempo de compilación y ejecución que el saldo se maneje como `String` en los DTOs, preparándose para su conversión a `rust_decimal` en el dominio, evitando el uso de `f32`/`f64`.
- **INV-18 (Inmutabilidad de la moneda):** se verifica que la moneda asociada a una cuenta sea fija tras su creación y no pueda ser modificada por operaciones posteriores.

### 2.2. Validaciones de `CrearCuenta`
Se implementaron pruebas que verifican la detectabilidad de los siguientes escenarios inválidos en el request:
- Nombre vacío o compuesto solo por espacios.
- Moneda fuera del catálogo base permitido (ej. "BTC").
- Saldo inicial con valor negativo.
- Saldo inicial con cantidad de decimales inválida para la moneda seleccionada (ej. 3 decimales para USD).

### 2.3. Resultado de la Ejecución
Las pruebas se ejecutan exitosamente en el entorno local y en el pipeline de CI/CD de GitHub Actions.
Nota: Las pruebas del crate infraestructura que involucran integración de red (como conexion_servidor) se ejecutan de forma manual en entornos locales con el backend activo, ya que el entorno de CI/CD no levanta el servidor Python. Las pruebas de invariantes y dominio (el núcleo de la rúbrica) se validan automáticamente en el pipeline.

---

## 3. Prueba de Autonomía (Local-First)
Se ejecutó el flujo de autonomía para validar el requisito no funcional de funcionamiento offline (ADR-005). La prueba consiste en operar la aplicación sin conectividad a internet y con el servidor backend apagado.

**Pasos ejecutados:**
1. Abrir la aplicación y verificar la carga de cuentas existentes.
2. Desactivar la interfaz de red (WiFi/Ethernet) y detener el servidor FastAPI.
3. Reabrir la aplicación y confirmar que la lista de cuentas persiste y se renderiza correctamente.
4. Crear una nueva cuenta en modo offline y confirmar que se guarda localmente y se refleja en la UI.

--- 

## 4. Ubicación del Almacenamiento
Cumpliendo con el principio de privacidad y el **ADR-005**, los datos financieros no salen del dispositivo. 
- **Tecnología:** SQLite embebido mediante la crate `rusqlite`.
- **Ubicación:** El archivo de base de datos se almacena en el directorio de datos nativo de la aplicación, gestionado por la API `app_data_dir` de Tauri (ej. `C:\Users\[Usuario]\AppData\Roaming\com.cuentasclaras.app` en Windows, o `~/.local/share/com.cuentasclaras.app` en Linux).

---

## 5. Instrucciones de Ejecución 
Para reproducir las validaciones de QA, siga estos pasos desde la raíz del repositorio:

1. Navegar al directorio de la aplicación:
   ```bash
   cd app
    ```
2. Ejecutar las pruebas unitarias de invariantes y repositorio en memoria:
   ```bash
   cargo test -p contrato -p aplicacion 
   ```
3. (Opcional) Verificar que el código compila sin advertencias:
   ```bash
   cargo check -p contrato
   ```

---
   
## 6. Conclusión de QA
El sistema cumple con los criterios de aceptación de la Entrega 2. La arquitectura hexagonal se respeta, los invariantes financieros están protegidos por pruebas automatizadas en memoria, y la funcionalidad Local-First ha sido verificada. Arrojando un estado adecuado para ser entregado. 
