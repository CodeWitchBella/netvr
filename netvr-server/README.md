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

Run `deno task compile`. This will produce executable file which you can use to run local server.

Also to have dashboard available you need to have dashboard built first. See README.md in netvr-dashboard folder for instructions on how to do that.

## Deploying server to cloudflare

You'll need [nodejs 16][nodejs] (later might also work, but I did not test that), and [yarn classic][yarn]. To install nodejs use the LTS download button on their [home page][nodejs]. To install yarn I recommend running `corepack enable yarn` as admin (corepack is installed by default with recent nodejs versions). Then you can run `yarn` to install dependencies followed by `yarn publish` to do the deployment in this folder (netvr-server). You might need to update `wrangler.toml` with your account_id and/or login to cloudflare. See wrangler's documentation.

Also to have dashboard available you need to have dashboard built first. See README.md in netvr-dashboard folder for instructions on how to do that.

[nodejs]: https://nodejs.org/en/
[yarn]: https://classic.yarnpkg.com/lang/en/

## Developing locally

If you have deno (see above), you can just run `deno task run`.

If you want to simulate the cloudflare version run `yarn` to install dependencies followed by `yarn dev` to run the dev server.

If you have dashboard build the local dev server will serve that, but if you want to make changes you can also access the dashboard from its development server. It will connect to same server.
