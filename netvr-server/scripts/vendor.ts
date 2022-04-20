/**
 * Simple wrapper around deno vendor which also updates the resulting import map
 *
 * Run using:
 * $ deno task vendor
 */

await Deno.run({
  cmd: [
    'deno',
    'vendor',
    '--import-map=./import_map.json',
    '--force',
    'server.ts',
  ],
}).status()

const importMap = JSON.parse(
  await Deno.readTextFile(new URL('../import_map.json', import.meta.url)),
)
const vendorMapUrl = new URL('../vendor/import_map.json', import.meta.url)
const vendorImportMap = JSON.parse(await Deno.readTextFile(vendorMapUrl))

Deno.writeTextFile(
  vendorMapUrl,
  JSON.stringify(
    {
      imports: {
        ...vendorImportMap.imports,
        ...Object.fromEntries(
          Object.entries(importMap.imports as { [key: string]: string }).map(
            ([k, v]) => [
              k.startsWith('./') ? '.' + k : k,
              v.startsWith('./') ? '.' + v : v,
            ],
          ),
        ),
      },
      scopes: vendorImportMap.scopes,
    },
    null,
    2,
  ) + '\n',
)

export {}
