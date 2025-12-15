## Submerge Documentation

The complete documentation for Submerge, its CLIs, and APIs. It's a [Docusaurus](https://docusaurus.io/)-based site, with [OpenAPI Doc Generator](https://github.com/PaloAltoNetworks/docusaurus-openapi-docs) to generate the API documentation.



To start in development mode:

```bash
npm install
npm run gen-all-api-docs
npm run start
```

To clean and regenerate all API docs, use the `regen-all-api-docs` command, which is effectively the same as first running the `clean-all-api-docs` and then `gen-all-api-docs`.

To build the static HTML site:

```bash
npm install
npm run gen-all-api-docs
npm run build
```
