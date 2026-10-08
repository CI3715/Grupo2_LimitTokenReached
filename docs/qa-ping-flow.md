# Checklist de QA: Full Stack Ping Flow (Entrega 1)

## 1. Interfaz de Usuario (Frontend)
- [ ] La pantalla muestra un botón con el texto exacto: "Verificar Conexión con el Servidor".
- [ ] La pantalla muestra un indicador de estado inicial: "Desconectado".
- [ ] Al tener éxito, el indicador cambia a "Conectado".
- [ ] Se muestra el mensaje del servidor: "Servidor Cuentas Claras activo".
- [ ] Se muestra la versión recibida: "0.1.0".
- [ ] Se muestra el tiempo de respuesta (latencia) en milisegundos (ms).

## 2. Flujo de Comunicación (Arquitectura)
- [ ] El frontend NO usa Server Components, Server Actions ni API Routes (Guía Técnica §1.1).
- [ ] El estado se maneja solo con estado local o máximo un contexto (Nada de Redux, Zustand o React Query).
- [ ] Next.js tiene configurado `output: 'export'`.
- [ ] Al presionar el botón, Next.js invoca `invoke('ping_servidor')` (comando Tauri en Rust).

## 3. Respuesta del Servidor (Backend)
- [ ] El endpoint `GET /ping` responde con código HTTP `200 OK`.
- [ ] El JSON de respuesta coincide exactamente con: `{"estado": "ok", "mensaje": "Servidor Cuentas Claras activo", "version": "0.1.0", "timestamp": "..."}`.
- [ ] El `timestamp` se genera a través de un puerto `Reloj` inyectado (no es una llamada directa a `datetime.now()`).

## 4. Fitness Functions y Calidad (CI)
- [ ] `import-linter` (Python) o `eslint-plugin-boundaries` (TS/Rust) pasa en verde.
- [ ] No hay warnings de `mypy --strict`, `ruff`, `black` o `clippy`.
