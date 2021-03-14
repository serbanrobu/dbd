# Database Dumper

## dbd

Client for fetching database dumps

**Build:**

```sh
cargo build --release -p dbd
```

**Setup (optional):**

Copy _dbd/Config.toml.example_ file to _$HOME/.dbd.toml_ and configure it.

**Help:**

```txt
USAGE:
    dbd [OPTIONS] <connection-id>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --agent-id <agent-id>
        Agent ID from configuration. Defaults to database connection ID if the
        ids match [env: AGENT_ID=]

    -k, --api-key <api-key>    Key for accessing agent's API [env: API_KEY]

    -c, --config <config>
        Config file name. Defaults to $HOME/.dbd.toml [env: CONFIG=]

    -d, --dbname <dbname>
        Database Name. Required if the database connection doesn't have a
        default dbname [env: DBNAME=]

    -e, --exclude-table-data <exclude-table-data>
        Do not dump the specified table data. To specify more than one table to
        ignore, use comma separator, e.g. --exclude-table-data=table_1,table_2
        [env: EXCLUDE_TABLE_DATA=]

    -u, --url <url>    Agent URL [env: URL=]

ARGS:
    <connection-id>
        Database connection ID configured from the agent [env: CONNECTION_ID=]
```

## dbd-agent

Agent for serving database dumps

**Setup:**

Copy _dbd-agent/Config.toml.example_ file to _$HOME/.dbd-agent.toml_ and
configure it.

**Build:**

```sh
cargo build --release -p dbd-agent
```

**Help:**

```txt
USAGE:
    dbd-agent [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>
        Config file name. Defaults to $HOME/.dbd-agent.toml [env: CONFIG=]
```

## Docker

### dbd container

**Build:**

```sh
cd dbd
docker build -t dbd .
```

**Help:**

```sh
docker run --rm -i \
  --env-file .env \
  --network host \
  -v $HOME/.dbd.toml:/root/.dbd.toml \
  dbd -h
```

### dbd-agent container

**Build:**

```sh
cd dbd-agent
docker build -t dbd-agent .
```

**Help:**

```sh
docker run --rm -it \
  --env-file .env \
  --network host \
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
rust-musl-builder cargo build --release -p dbd-agent
```

**Client:**

```sh
rust-musl-builder cargo build --release -p dbd
```
