/**
 * Creates single executable to run server which includes all the assets required
 *
 * Run using:
 * $ deno task compile
 */
//@ts-ignore
import { Leaf } from '../vendor-patched/leaf.ts'

await Leaf.compile({
  modulePath: './server.ts',
  contentFolders: ['../netvr-dashboard/dist'],
  flags: [
    '--allow-net',
    '--no-check',
    '--allow-env',
    '--allow-write=./netvr-room.json',
    ...Deno.args,
  ],
  emitOptions: {
    importMapPath: './import_map.json',
  },
})
