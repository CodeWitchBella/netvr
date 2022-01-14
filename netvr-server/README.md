# netvr-server

This folder contains source code for the server part of netvr. If you want the server to include the dashboard you first have to build that. See instructions in `netvr-dashboard`.

To run the server you have three options

- download prebuilt server binary from CI
- build deno-based local server
- deploy server to cloudflare workers

## Downloading prebuilt server binaries

Go to [actions/netvr-server on github](https://github.com/CodeWitchBella/netvr/actions/workflows/netvr-server.yaml), open latest successfully completed run, download the build you want. This might not work if artifacts expired, I needed to set aggressive expiration to cut down on costs.

## Building deno-based local server

Install [deno](https://deno.land).

If you have `yarn` you can then run `yarn deno:compile`, otherwise look in `package.json` and run the command specified in `scripts/deno:compile`. This will produce executable file which you can use to run local server.

## Deploying server to cloudflare

You'll need [wrangler](https://developers.cloudflare.com/workers/cli-wrangler), and [yarn](https://classic.yarnpkg.com/lang/en/). Install both and run `yarn` followed by `wrangler publish`. You might need to update `wrangler.toml` with your account_id and/or login to cloudflare. See wrangler's documentation.

## Developing locally

If you have deno and yarn (see above), you can just run `yarn deno:run`.
