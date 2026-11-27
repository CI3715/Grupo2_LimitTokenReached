import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';
import vm from 'node:vm';
import { createRequire } from 'node:module';
import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import ts from 'typescript';

const resolveDependency = createRequire(import.meta.url);

const filename = path.resolve(
  import.meta.dirname,
  '../src/components/social/GroupDetail.tsx',
);
const source = ts.transpileModule(fs.readFileSync(filename, 'utf8'), {
  compilerOptions: {
    module: ts.ModuleKind.CommonJS,
    target: ts.ScriptTarget.ES2020,
    jsx: ts.JsxEmit.ReactJSX,
  },
}).outputText;

const compiled = { exports: {} };
vm.runInThisContext(`(function(require,module,exports){${source}\n})`, {
  filename,
})((name) => {
  if (name === 'react/jsx-runtime') return resolveDependency(name);
  if (name === '@/components/social/GroupForms') {
    return {
      GroupExpenseForm: ({ feedback }) =>
        feedback ? React.createElement(
          'p',
          { className: 'group-success', role: 'status' },
          feedback,
        ) : null,
      InviteMemberForm: () => null,
    };
  }
  return resolveDependency(name);
}, compiled, compiled.exports);

const { GroupDetail } = compiled.exports;

function syncProps(overrides = {}) {
  return {
    cuentas: [
      {
        id: 'available',
        nombre: 'Disponible',
        moneda_codigo: 'USD',
        activa: true,
      },
      {
        id: 'inactive',
        nombre: 'Inactiva',
        moneda_codigo: 'USD',
        activa: false,
      },
      {
        id: 'other-currency',
        nombre: 'Otra moneda',
        moneda_codigo: 'VES',
        activa: true,
      },
    ],
    seleccion: '',
    estado: {
      cursor: 0,
      ultima_sincronizacion: null,
      sesion_vencida: false,
    },
    guardando: false,
    sincronizando: false,
    error: '',
    vincular: async () => true,
    sincronizar: async () => true,
    ...overrides,
  };
}

function detalleProps(sync) {
  return {
    detalle: {
      id: 'group-1',
      nombre: 'Viaje',
      descripcion: null,
      moneda: 'USD',
      actividad: null,
      miembros: [],
      gastos: [],
      balances: {},
      partes: [],
    },
    usuarioActual: 'carlos',
    amigos: [],
    pending: false,
    gastoFeedback: '',
    sync,
    onInvite: async () => true,
    onRegisterExpense: async () => true,
    onPreview: async () => [],
  };
}

test('selector muestra solo cuentas activas de la moneda del grupo', () => {
  const html = renderToStaticMarkup(
    React.createElement(GroupDetail, detalleProps(syncProps())),
  );

  assert.match(html, /Disponible/);
  assert.doesNotMatch(html, /Inactiva|Otra moneda/);
});

test('sesión vencida desactiva sincronización y muestra mensaje', () => {
  const sync = syncProps({
    seleccion: 'available',
    estado: {
      cursor: 0,
      ultima_sincronizacion: null,
      sesion_vencida: true,
    },
  });
  const html = renderToStaticMarkup(
    React.createElement(GroupDetail, detalleProps(sync)),
  );

  assert.match(html, /Tu sesión venció/);
  assert.match(html, /disabled/);
});

test('muestra la última sincronización cuando existe', () => {
  const sync = syncProps({
    seleccion: 'available',
    estado: {
      cursor: 1,
      ultima_sincronizacion: '2026-11-23T17:30:00Z',
      sesion_vencida: false,
    },
  });
  const html = renderToStaticMarkup(
    React.createElement(GroupDetail, detalleProps(sync)),
  );

  assert.match(html, /Última sincronización:/);
  assert.doesNotMatch(html, /Tu sesión venció/);
});

test('aviso de gasto en cola aparece junto al formulario como éxito', () => {
  const props = detalleProps(syncProps());
  props.gastoFeedback = 'Gasto guardado pendiente de envío.';
  const html = renderToStaticMarkup(
    React.createElement(GroupDetail, props),
  );

  assert.match(html, /Gasto guardado pendiente de envío\./);
  assert.match(html, /class="group-success"/);
  assert.doesNotMatch(html, /class="group-page-error"/);
});
