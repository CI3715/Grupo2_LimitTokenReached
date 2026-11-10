// Prueba el contrato UI -> Tauri. La autorización HTTP se prueba en servidor/.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { test } from 'node:test';
import vm from 'node:vm';
import ts from 'typescript';

function servicio(invoke) {
  const filename = path.resolve(import.meta.dirname, '../src/services/autenticacion.ts');
  const source = ts.transpileModule(fs.readFileSync(filename, 'utf8'), {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2020 },
  }).outputText;
  const compiledModule = { exports: {} };
  const require = (name) => {
    assert.equal(name, '@tauri-apps/api/core');
    return { invoke };
  };
  vm.runInThisContext(`(function(require, module, exports) { ${source}\n})`, { filename })(
    require, compiledModule, compiledModule.exports,
  );
  return compiledModule.exports;
}

const registro = {
  nombre_usuario: 'rosa.qa', nombre_visible: 'Rosa QA',
  correo: 'rosa.qa@example.com', contrasena: 'Clave exclusiva de prueba 123',
};

for (const [funcion, comando, datos, resultado] of [
  ['registrarUsuario', 'registrar_usuario', registro, { nombre_usuario: 'rosa.qa' }],
  ['iniciarSesion', 'iniciar_sesion', { correo: registro.correo, contrasena: registro.contrasena }, { nombre_usuario: 'rosa.qa' }],
  ['obtenerPerfil', 'obtener_perfil', undefined, { nombre_usuario: 'rosa.qa' }],
  ['cerrarSesion', 'cerrar_sesion', undefined, undefined],
]) {
  test(`${funcion}: utiliza el comando y los argumentos esperados por Tauri`, async () => {
    let llamadas = 0;
    const auth = servicio(async (nombre, args) => {
      llamadas++;
      assert.equal(nombre, comando);
      assert.deepEqual(args, datos);
      if (args) {
        for (const prohibido of ['cuentas', 'movimientos', 'saldos', 'tasas', 'token']) {
          assert.equal(Object.hasOwn(args, prohibido), false);
        }
      }
      return resultado;
    });
    assert.deepEqual(await auth[funcion](datos), resultado);
    assert.equal(llamadas, 1);
  });

  test(`${funcion}: conserva el rechazo del backend y no reintenta la operación`, async () => {
    for (const error of [
      { campo: 'sesion', mensaje: 'Autenticación requerida.' },
      'Tu sesión venció, inicia sesión de nuevo.',
      'No se pudo conectar con el servidor. Verifica tu conexión.',
    ]) {
      let llamadas = 0;
      const auth = servicio(async () => { llamadas++; throw error; });
      await assert.rejects(auth[funcion](datos), (actual) => actual === error);
      assert.equal(llamadas, 1);
    }
  });
}
