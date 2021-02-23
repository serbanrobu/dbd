# Database Dumper

## dbd

Client for fetching database dumps

**Build:**

```sh
cargo build --release --bin dbd
```

**Help:**

```txt
USAGE:
    dbd [OPTIONS] <database-id> --api-key <api-key> --url <url>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -k, --api-key <api-key>                          Key for accessing agent's
    API [env: API_KEY]
        --exclude-table-data <exclude-table-data>
            Do not dump the specified table data. To specify more than one
            table to ignore, use comma separator, e.g.
            --exclude-table-data=table_1,table_2 [env: EXCLUDE_TABLE_DATA=]
    -u, --url <url>                                  Agent URL [env: URL=]

ARGS:
    <database-id>    Database ID configured from the agent [env: DATABASE_ID=]
```

## dbd-agent

Agent for serving database dumps

**Setup:**

Copy _.dbd-agent.toml.example_ file to _$HOME/.dbd-agent.toml_ and configure
it.

**Build:**

```sh
cargo build --release --bin dbd-agent
```

**Help:**

```txt
USAGE:
    dbd-agent [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>    Config file name. Defaults to $HOME/.dbd-agent.*
    [env: CONFIG=]
```

## Docker

### dbd container

**Build:**

```sh
docker build -t dbd .
```

**Help:**

```sh
docker run --rm -i --network host --env-file .env dbd -h
```

### dbd-agent container

**Build:**

```sh
docker build -t dbd-agent -f Agent.Dockerfile .
```

**Help:**

```sh
docker run --rm -it --network host \
  -v $HOME/.dbd-agent.toml:/root/.dbd-agent.toml \
  dbd-agent -h
```

## MUSL for fully static binaries

Check out [rust-musl-builder](https://github.com/emk/rust-musl-builder), a
Docker container for easily building static Rust binaries.

You can set a bash alias like this:

```bash
alias rust-musl-builder='docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder'
```

And build the binaries like so:

**Agent:**

```sh
rust-musl-builder cargo build --release --bin dbd-agent
```

**Client:**

```sh
rust-musl-builder cargo build --release --bin dbd
```
