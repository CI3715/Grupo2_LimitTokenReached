import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import vm from "node:vm";
import ts from "typescript";

const filename = path.resolve(
  import.meta.dirname,
  "../src/features/backup/useBackup.ts",
);
const source = ts.transpileModule(fs.readFileSync(filename, "utf8"), {
  compilerOptions: {
    module: ts.ModuleKind.CommonJS,
    target: ts.ScriptTarget.ES2020,
  },
}).outputText;
const compiled = { exports: {} };
vm.runInThisContext(`(function(require,module,exports){${source}\n})`, {
  filename,
})(
  (name) => {
    if (name === "react" || name === "@/services/backup") return {};
    throw new Error(`Dependencia inesperada: ${name}`);
  },
  compiled,
  compiled.exports,
);
const { backupErrorMessage } = compiled.exports;

const casos = [
  [
    "Frase de paso incorrecta o archivo dañado.",
    "La frase de paso no es correcta o el archivo está dañado.",
  ],
  [
    "Versión de formato no soportada: 2.0.",
    "La versión del archivo no es compatible con esta aplicación.",
  ],
  [
    "No se pudo llegar a la carpeta; se reintentará.",
    "No se pudo acceder al destino. Comprueba la ruta y vuelve a intentarlo.",
  ],
  [
    "Las credenciales fueron rechazadas",
    "Git rechazó las credenciales guardadas. Actualízalas en el gestor de credenciales de Git de este dispositivo y vuelve a sincronizar.",
  ],
  [
    "Evento con identificador repetido y contenido distinto",
    "Hay un evento con identificador repetido y contenido diferente.",
  ],
];

test("presenta mensajes seguros para los cinco errores del contrato", () => {
  for (const [entrada, esperado] of casos) {
    assert.equal(
      backupErrorMessage(`${entrada}\nstack trace src/secret.rs unwrap`),
      esperado,
    );
  }
});

test("oculta errores técnicos desconocidos", () => {
  assert.equal(
    backupErrorMessage(new Error("panic src/secret.rs")),
    "No se pudo completar la operación. Inténtalo nuevamente.",
  );
});

test("interpreta los códigos serializados por los comandos", () => {
  const codigos = [
    "FraseIncorrectaOArchivoDanado",
    "VersionFormatoNoSoportada",
    "DestinoInaccesibleOSinConexion",
    "CredencialesGitInvalidas",
    "EventoConflictoContenido",
  ];
  for (let i = 0; i < codigos.length; i++)
    assert.equal(backupErrorMessage({ codigo: codigos[i] }), casos[i][1]);
  assert.equal(
    backupErrorMessage({ codigo: "ErrorOperacion", detalle: "src/privado" }),
    "No se pudo completar la operación. Inténtalo nuevamente.",
  );
});
