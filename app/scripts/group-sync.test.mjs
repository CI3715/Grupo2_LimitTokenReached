import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';
import vm from 'node:vm';
import { createRequire } from 'node:module';
import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import ts from 'typescript';

const filename = path.resolve(import.meta.dirname, '../src/components/group-sync/GroupSyncIndicators.tsx');
const source = ts.transpileModule(fs.readFileSync(filename, 'utf8'), { compilerOptions: { module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX } }).outputText;
const compiled = { exports: {} };
const resolve = createRequire(import.meta.url);
vm.runInThisContext(`(function(require,module,exports){${source}\n})`, { filename })(
  (name) => name.endsWith('.css') ? { __esModule: true, default: new Proxy({}, { get: (_, key) => key }) } : resolve(name), compiled, compiled.exports,
);
const { PendingGroupExpenses, QueuedGroupExpenses, GroupMovementLabel } = compiled.exports;
const render = (component, props) => renderToStaticMarkup(React.createElement(component, props));

test('los listados vacíos no muestran avisos', () => {
  assert.equal(render(PendingGroupExpenses, { items: [] }), '');
  assert.equal(render(QueuedGroupExpenses, { items: [] }), '');
});
test('muestra el grupo, el gasto y la parte pendiente', () => {
  const html = render(PendingGroupExpenses, { items: [{ id: '1', grupo: 'Viaje', concepto: 'Hotel', fecha: '2026-11-16', monto: '33.33', moneda: 'USD' }] });
  for (const value of ['Viaje', 'Hotel', '33.33', 'USD', '2026-11-16']) assert.ok(html.includes(value));
});
test('distingue gastos pendientes y rechazados con su motivo', () => {
  const html = render(QueuedGroupExpenses, { items: [{ id: '1', concepto: 'Cena', monto: '60.00', moneda: 'USD', estado: 'pendiente' }, { id: '2', concepto: 'Hotel', monto: '10.00', moneda: 'USD', estado: 'rechazado', motivo: 'Sin permiso' }] });
  for (const value of ['Pendiente de envío', 'Rechazado al sincronizar', 'Sin permiso', 'role="alert"']) assert.ok(html.includes(value));
});
test('identifica el movimiento de grupo', () => assert.match(render(GroupMovementLabel, {}), /Gasto de grupo/));
test('selector muestra solo cuentas activas de la moneda del grupo', () => {
  const html = render(compiled.exports.GroupAccountSelector, { moneda: 'USD', seleccion: '', guardando: false, onSelect() {}, cuentas: [
    { id: '1', nombre: 'Disponible', moneda: 'USD', activa: true },
    { id: '2', nombre: 'Inactiva', moneda: 'USD', activa: false },
    { id: '3', nombre: 'Otra moneda', moneda: 'VES', activa: true },
  ] });
  assert.match(html, /Disponible/);
  assert.doesNotMatch(html, /Inactiva|Otra moneda/);
});
test('sesión vencida desactiva sincronización y muestra mensaje', () => {
  const html = render(compiled.exports.SyncButton, { ultimaSincronizacion: null, sincronizando: false, sesionVencida: true, onSync() {} });
  assert.match(html, /Tu sesión venció/);
  assert.match(html, /disabled/);
});

test('servicios usan los comandos y contratos Tauri de sincronización', async () => {
  const file = path.resolve(import.meta.dirname, '../src/services/groupSync.ts');
  const js = ts.transpileModule(fs.readFileSync(file, 'utf8'), { compilerOptions: { module: ts.ModuleKind.CommonJS } }).outputText;
  const mod = { exports: {} };
  const calls = [];
  vm.runInThisContext(`(function(require,module,exports){${js}\n})`, { filename: file })(
    () => ({ invoke: async (name, args) => { calls.push([name, args]); return null; } }), mod, mod.exports,
  );
  await mod.exports.sincronizarGrupos();
  await mod.exports.estadoSincronizacion();
  await mod.exports.listarPendientesGrupo();
  await mod.exports.listarColaGrupo();
  await mod.exports.obtenerCuentaGrupo('grupo-1');
  await mod.exports.elegirCuentaGrupo({ grupo_id: 'grupo-1', cuenta_id: 'cuenta-1', moneda: 'USD' }, 'USD');
  assert.deepEqual(calls, [
    ['sincronizar_grupos', undefined], ['estado_sincronizacion', undefined],
    ['listar_partes_grupo_pendientes', undefined], ['listar_cola_gastos_grupo', undefined],
    ['obtener_cuenta_grupo', { grupo_id: 'grupo-1' }],
    ['elegir_cuenta_grupo', { seleccion: { grupo_id: 'grupo-1', cuenta_id: 'cuenta-1', moneda: 'USD' }, moneda_grupo: 'USD' }],
  ]);
});
