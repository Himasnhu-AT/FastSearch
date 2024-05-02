# FastSearch Dev Docs

Fast Search is implementation of end to end google's Search Engines whose core is written in Rust. The benchmark of the project will be released soon. This repo is built to ensure privatization of search engine technology, such that organizations / individuals can build their own search engine which only parses certain repository's to ensure that the result recieved are from trusted websites (/sources) which are accepted by them.

## Get Started

### Installation

#### Using Github

1. Clone the repository and setup the environment

```bash
git clone https://github.com/himasnhu-at/fastsearch.git
cd fastsearch
git submodule init
git submodule update
```

> [!Note]
> If submodules doesn't work, go to `/packages/engine/` and run `git clone https://github.com/BSVino/docs.gl`

2. Build the project

```bash
cd packages/engine
cargo build
```

3. Run the project

```bash
cargo run --release
```

> [!Note]
> Along with
>
> ```bash
> cargo run --release
> ```
>
> you need to pass arguments to the engine on what it should do at the moments. For example, to index the documents, you can run
>
> ```bash
> cargo run --release index <path_to_folder>
> ```
>
> where `<path_to_folder>` is the path to the folder containing the documents to be indexed. You can find all arguments that can be passed to the engine by running
>
> ```bash
> cargo run --release
> ```

4. to build dev docs

```bash
cd <root_of_project>/dev-docs
pnpm install
pnpm run dev
```

#### Using Docker

> TBD

### How to use

> TBD

## Guidelines

> TBD

## Contribution

To contribute to this repo, follow [contribution guidelines](/contributing.md)

## License

This is licensed under SYNTHAI-LABS OpenSource License
