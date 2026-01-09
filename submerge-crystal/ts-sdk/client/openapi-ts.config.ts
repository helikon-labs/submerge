import { defineConfig } from "@hey-api/openapi-ts";

export default defineConfig({
  input:
    "https://raw.githubusercontent.com/helikon-labs/submerge/refs/heads/development/submerge-crystal/api-spec/submerge-crystal-api.json",
  output: {
    format: "prettier",
    lint: "eslint",
    path: "./src/client",
    preferExportAll: true,
  },
  plugins: [
    "@hey-api/schemas",
    {
      dates: true,
      name: "@hey-api/transformers",
    },
    {
      enums: "typescript",
      name: "@hey-api/typescript",
    },
    {
      name: "@hey-api/sdk",
      transformer: true,
      asClass: true,
      classNameBuilder(name) {
        return `${name}Service`;
      },
      validator: true,
    },
    {
      name: "zod",
      requests: true,
      // FIX: better to enable it but OpenAPI spec file causes validation related issue
      responses: false,
    },
  ],
});
