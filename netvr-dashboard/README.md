# netvr-dashboard

This folder contains simple dashboard application for managing devices connected to single netvr server. It also contains source code used to render pdf file of the thesis.

## Compiling

To compile the dashboard you'll need latest [nodejs 20](https://nodejs.org/en/).
If you have not done so, I recommend running `corepack enable yarn` as admin (do not run any other commands as admin). This will make sure that you have correct yarn version for this project.

Once you have the correct system-wide dependencies you can run `yarn` to download and install this projects dependencies. The run `yarn build` to build the project.

## Developing

Same as compiling except as the last step you run `yarn dev` instead of `yarn build`.
