import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { createRequire } from 'node:module';
import ts from 'typescript';
import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';

const resolveDependency = createRequire(import.meta.url);

// Usa el compilador ya instalado; no requiere otro runner ni modificar el código productivo.
function load(relative, mocks = {}) {
  const filename = path.resolve(import.meta.dirname, '..', relative);
  const source = ts.transpileModule(fs.readFileSync(filename, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020, jsx: ts.JsxEmit.ReactJSX },
  }).outputText;
  const compiledModule = { exports: {} };
  vm.runInThisContext(`(function(require, module, exports) { ${source}\n})`, { filename })(
    (name) => {
      if (Object.hasOwn(mocks, name)) return mocks[name];
      if (name.endsWith('.css')) return {};
      if (name.startsWith('@/')) return load(`src/${name.slice(2)}.ts`, mocks);
      return resolveDependency(name);
    }, compiledModule, compiledModule.exports,
  );
  return compiledModule.exports;
}

const request = {
  cuenta_id: 'cuenta-real', tipo: 'Gasto', monto: '50.00', fecha: '2026-10-19',
  categoria_id: 'categoria-real', nota: null,
};
const response = {
  ...request, id: 'movimiento-real', es_correccion: false, movimiento_original_id: null,
};

test('envía el comando y el contrato exactos, preservando el monto como string y nota null', async () => {
  const service = load('src/services/registerMovement.ts', {
    '@tauri-apps/api/core': { invoke: async (command, args) => {
      assert.equal(command, 'registrar_movimiento');
      assert.deepEqual(args, { datos: request });
      assert.equal(typeof args.datos.monto, 'string');
      return response;
    } },
  });
  assert.deepEqual(await service.registerMovement(request), response);
});

test('acepta un monto equivalente normalizado por el backend', async () => {
  const service = load('src/services/registerMovement.ts', {
    '@tauri-apps/api/core': { invoke: async () => ({ ...response, monto: '50' }) },
  });
  assert.equal((await service.registerMovement(request)).monto, '50');
});

for (const [name, patch] of Object.entries({
  'otra cuenta': { cuenta_id: 'otra' },
  'otro monto': { monto: '50.01' },
  'monto numérico': { monto: 50 },
  'otra fecha': { fecha: '2026-10-18' },
  'otra categoría': { categoria_id: 'otra' },
  'ajuste': { tipo: 'Ajuste', es_correccion: true },
  'id ausente': { id: undefined },
  'referencia de corrección': { movimiento_original_id: 'original' },
})) {
  test(`respuesta inesperada (${name}): exige verificar antes de reenviar`, async () => {
    const service = load('src/services/registerMovement.ts', {
      '@tauri-apps/api/core': { invoke: async () => ({ ...response, ...patch }) },
    });
    await assert.rejects(service.registerMovement(request), (error) => error.verificarRegistro === true);
  });
}

test('preserva el campo y mensaje del error de dominio en objeto o JSON', async () => {
  const error = { campo: 'monto', mensaje: 'El monto debe ser mayor que cero' };
  const service = load('src/services/registerMovement.ts', {
    '@tauri-apps/api/core': { invoke: async () => { throw error; } },
  });
  await assert.rejects(service.registerMovement(request), (caught) => caught === error);
  for (const value of [error, JSON.stringify(error)]) {
    assert.equal(service.movementError(value).campo, error.campo);
    assert.equal(service.movementError(value).mensaje, error.mensaje);
  }
  assert.ok(service.movementError(null).mensaje);
  assert.equal(service.movementError('No disponible').mensaje, 'No disponible');
});

test('delega la fecha futura al backend y conserva su error de campo', async () => {
  const futureRequest = { ...request, fecha: '2030-01-01' };
  const error = { campo: 'fecha', mensaje: 'La fecha no puede ser futura' };
  const service = load('src/services/registerMovement.ts', {
    '@tauri-apps/api/core': { invoke: async (command, args) => {
      assert.equal(command, 'registrar_movimiento');
      assert.deepEqual(args, { datos: futureRequest });
      throw error;
    } },
  });
  await assert.rejects(service.registerMovement(futureRequest), (caught) => caught === error);
  for (const value of [error, JSON.stringify(error)]) {
    assert.equal(service.movementError(value).campo, 'fecha');
    assert.equal(service.movementError(value).mensaje, error.mensaje);
  }
});

test('advertencia exacta: igualdad, fracciones, saldo negativo y cantidades superiores a Number.MAX_SAFE_INTEGER', () => {
  const { expenseExceedsBalance: exceeds } = load('src/types/movements.ts');
  assert.equal(exceeds('0.30', '0.3'), false);
  assert.equal(exceeds('0.31', '0.30'), true);
  assert.equal(exceeds('50', '100.00'), false);
  assert.equal(exceeds('0.00000002', '0.00000001'), true);
  assert.equal(exceeds('9007199254740993.01', '9007199254740993.00'), true);
  assert.equal(exceeds('9007199254740993.00', '9007199254740993.01'), false);
  assert.equal(exceeds('1', '-10'), true);
  assert.equal(exceeds('no es un monto', '100'), false);
});

const formProps = {
  account: { id: 'cuenta-real', nombre: 'Mi cuenta', moneda_codigo: 'USD', saldo: '100.00' },
  currency: { codigo: 'USD', nombre: 'Dólar', simbolo: '$', decimales: 2 },
  categories: [
    { id: 'gasto-real', nombre: 'Alimentación', tipo: 'Gasto' },
    { id: 'ingreso-real', nombre: 'Salario', tipo: 'Ingreso' },
  ],
  onRegister: async () => response, onRegistered() {}, onCancel() {},
};

test('formulario inicial: cuenta recibida, solo ingreso/gasto, IDs reales y categorías del tipo seleccionado', () => {
  const { MovementForm } = load('src/components/movements/movement-form.tsx');
  const html = renderToStaticMarkup(React.createElement(MovementForm, formProps));
  assert.match(html, /Mi cuenta/);
  assert.match(html, /Monto \(USD\)/);
  assert.match(html, /value="Gasto"/);
  assert.match(html, /value="Ingreso"/);
  assert.doesNotMatch(html, /value="Ajuste"/);
  assert.match(html, /value="gasto-real"/);
  assert.doesNotMatch(html, /value="ingreso-real"/);
  assert.doesNotMatch(html, /name="cuenta_id"/);
  const amountInput = html.match(/<input\b[^>]*name="monto"[^>]*>/)?.[0] ?? '';
  assert.match(amountInput, /type="text"/);
  assert.match(amountInput, /inputMode="decimal"/);
  const now = new Date();
  const today = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
  const dateInput = html.match(/<input\b[^>]*name="fecha"[^>]*>/)?.[0] ?? '';
  // El dominio rechaza fechas futuras; el formulario presenta el error devuelto.
  assert.match(dateInput, /type="date"/);
  assert.match(dateInput, /aria-describedby="[^"]*-fecha-error"/);
  assert.match(dateInput, new RegExp(`value="${today}"`));
});

test('sin catálogo real muestra estado vacío y deshabilita el registro', () => {
  const { MovementForm } = load('src/components/movements/movement-form.tsx');
  const html = renderToStaticMarkup(React.createElement(MovementForm, { ...formProps, categories: [] }));
  assert.match(html, /No hay categorías de gasto disponibles/);
  assert.match(html, /type="submit"[^>]*disabled/);
});
