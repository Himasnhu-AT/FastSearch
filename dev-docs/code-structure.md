# Code Structure:

## `/packages/engine`:

Contains rust code for the engine. It have following files:

#### `src/main.rs`:

- Entry point of the engine. It parse command line arguments and call appropriate function.

#### `docs.gl`:

github submodules repo which have `xhtml` files (around 1600+) for testing and benchmarking purpose.

#### `small`:

contains subset of `docs.gl` for intial testing purpose.

#### `webclient`:

`HTML` files that are sent to frontend from engine.

## External Dependencies:

- `serde_json`: for parsing json files.
- `tiny_http`: for setting up server and handling requests.
- `xml-rs`: for parsing xml files, mainly `xhtml`
