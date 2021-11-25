/**
 * Creates single executable to run server which includes all the assets required
 *
 * Run using:
 * $ deno run --unstable --allow-read --allow-write --allow-run compile.ts
 * Or
 * $ yarn deno:compile
 */
//@ts-ignore
import { Leaf } from './vendor/leaf.ts'

await Leaf.compile({
  modulePath: './server.ts',
  contentFolders: ['../netvr-dashboard/dist'],
  flags: ['--allow-net'],
  emitOptions: {
    importMapPath: './import_map.json',
  },
})
