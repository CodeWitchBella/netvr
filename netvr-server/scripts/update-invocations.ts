/**
 * Updates package.json scripts to match invocations in headers of script files.
 * Like the following one 😉
 *
 * Run using:
 * $ deno run --allow-read --allow-write scripts/update-invocations.ts
 * or
 * $ yarn update-invocations
 */

const base = new URL('..', (import.meta as any).url).toString()
console.log(base)
const scripts = [
  'scripts/update-invocations.ts',
  'scripts/vendor.ts',
  'scripts/compile.ts',
  'worker.ts',
  'server.ts',
].map((f) => new URL(f, base))
let pkgChanged = false
const pkgUrl = new URL('package.json', base)
const pkg = JSON.parse(await Deno.readTextFile(pkgUrl))

const template = ` * ---:(unmatch)
 * $ ...
 * or
 * $ yarn ...
`

for (const script of scripts) {
  console.log()
  console.log('>', script.toString())
  let original = await Deno.readTextFile(script)
  const matches = original.matchAll(pattern())
  for (const match of matches) {
    const [, command, yarnScript] = match
    const finalCommand = command.startsWith('yarn ')
      ? command.slice(5)
      : command
    console.log(yarnScript, '->', finalCommand)
    if (pkg.scripts[yarnScript] !== finalCommand) {
      pkg.scripts[yarnScript] = finalCommand
      pkgChanged = true
      console.log('Updated script', yarnScript)
    }
  }
}

if (pkgChanged)
  await Deno.writeTextFile(pkgUrl, JSON.stringify(pkg, null, 2) + '\n')

function pattern() {
  return new RegExp(
    template
      .replace('(unmatch)', '')
      .replace(/---/, '[^\n]+')
      .replace(/([*$])/g, '\\$1')
      .replace(/\.\.\./g, '([^\n]+)'),
    'g',
  )
}

export {}
