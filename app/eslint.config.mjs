import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTs from "eslint-config-next/typescript";
import boundaries from "eslint-plugin-boundaries";
import { recommended } from "eslint-plugin-boundaries/config";


const eslintConfig = defineConfig([
  ...nextVitals,
  ...nextTs,

  {
    files: ["src/**/*.{ts,tsx}"],
    plugins: { boundaries },
    settings: {
      ...recommended.settings,
      "import/resolver": { typescript: { alwaysTryTypes: true } },
      "boundaries/elements": [
        { type: "app", pattern: "src/app"},
        { type: "features", pattern: "src/features"},
        { type: "components", pattern: "src/components"},
        { type: "services", pattern: "src/services"},
        { type: "types", pattern: "src/types"},
      ],
    },
    rules: {
      ...recommended.rules,
      "boundaries/dependencies": [
        2,
        {
          default: "disallow",
          policies: [
            {
              from: { element: { type: "app" } },
              allow: { to: { element: { type: ["app", "features", "components", "services", "types"] } } },
            },
            {
              from: { element: { type: "features" } },
              allow: { to: { element: { type: ["features", "components", "services", "types"] } } },
            },
            {
              from: { element: { type: "components" } },
              allow: { to: { element: { type: ["components", "types"] } } },
            },
            {
              from: { element: { type: "services" } },
              allow: { to: { element: { type: ["services", "types"] } } },
            },
            {
              from: { element: { type: "types" } },
              allow: { to: { element: { type: ["types"] } } },
            },
          ],
        },
      ],
    },
  },
  
  
  // Override default ignores of eslint-config-next.
  globalIgnores([
  ".next/**",
  "out/**",
  "build/**",
  "next-env.d.ts",
  "src-tauri/**",
  "**/target/**",
  ]),
]);

export default eslintConfig;
