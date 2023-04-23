# AIrtifex

Self-hosted, generative AI server and a web app. The API provides the necessary endpoints for interacting with the generative models, while the web app serves as a client-side rendered WASM application for user interaction. The entire project is written in Rust.



![Preview GIF](https://raw.githubusercontent.com/vv9k/airtifex/master/assets/preview.gif)


## Table of Contents

- [Prerequisites](#prerequisites)
- [Setup](#setup)
- [API Configuration](#api-configuration)
- [Building and Running the Project](#building-and-running-the-project)
  - [API With SQLite](#api-with-sqlite)
  - [API With PostgreSQL](#api-with-postgresql)
  - [Web App](#web-app)

## Prerequisites

To work with this project, you will need the following tools installed:

- [Rust](https://www.rust-lang.org/tools/install): nightly compiler
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html): latest version
- [Trunk](https://trunkrs.dev/#install)
- [Make](https://www.gnu.org/software/make/)

## Setup

* Clone the repository:

```sh
git clone https://github.com/vv9k/airtifex.git
cd airtifex
```

## API Configuration

Below is an example configuration for the server that loads a single 7B Alpaca model for text generation as well as Stable Diffusion v2.1 and v1.5:

```yaml
---
listen_addr: 127.0.0.1
listen_port: 6901
db_url: sqlite://data.db
#db_url: postgres://airtifex:airtifex@localhost/airtifex
jwt_secret: change-me!

llms:
  - model_path: ./llm_models/ggml-alpaca-7b-q4.bin
    model_description: Alpaca 7B, quantized
    float16: false

stable_diffusion:
  - version: v2.1
    name: sd-v2.1
    model_description: Stable Diffusion v2.1
    clip_weights_path: ./sd_models/clip_v2.1.ot
    vae_weights_path: ./sd_models/vae_v2.1.ot
    unet_weights_path: ./sd_models/unet_v2.1.ot
    vocab_file: ./sd_models/bpe_simple_vocab_16e6.txt
  - version: v1.5
    name: sd-v1.5
    model_description: Stable Diffusion v1.5
    clip_weights_path: ./sd_models/clip_v1.5.ot
    unet_weights_path: ./sd_models/unet_v1.5.ot
    vocab_file: ./sd_models/bpe_simple_vocab_16e6.txt
```

## Building and Running the Project

### API with Sqlite

To build and run the project using SQLite as the database, follow these steps:

* To run directly use:

```sh
# start the server
cd airtifex-api
make serve_release
```
* To build use:
```sh
cd airtifex-api
make build_release
```
The binary will be in the `target/release` directory after the build succeeds.


### API with PostgreSQL

To build and run the project using PostgreSQL as the database, follow these steps:

* Set up a PostgreSQL database and update the db_url field in the API configuration file (e.g., `airtifex-api/config.yaml`).

* Run directly:
```sh
cd airtifex-api
make serve_release_pg
```

* Build the API server with PostgreSQL support:
```sh
cd airtifex-api
make build_release_pg
```

### Web App

In another terminal start the web app:
```sh
cd airtifex-web
make serve_release
```

The web app will be accessible at http://localhost:8080 by default and is configured to connect to the API server at localhost:6901. To configure it change the values in the `Trunk.toml` file.


## License
[GPLv3](https://github.com/vv9k/airtifex/blob/master/LICENSE)
