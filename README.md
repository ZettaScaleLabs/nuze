<div align="center">
    <h1>Nuze</h1>
    <p><strong>A Nu shell for Zenoh: debug systems, write (end-to-end) tests and build powerful CLI tools</strong></p>
    <p>
        <a href="https://crates.io/crates/nuze"><img alt="Crates.io" src="https://img.shields.io/crates/v/nuze.svg"></a>
        <a href="https://www.nushell.sh/"><img alt="Nushell 0.114.0" src="https://img.shields.io/badge/nushell-0.114.0-blue.svg"></a>
        <a href="https://zenoh.io/"><img alt="Zenoh 1.10.0" src="https://img.shields.io/badge/zenoh-1.10.0-blue.svg"></a>
    </p>
    <sub>Built by the <a href="https://zenoh.io">Zenoh</a> team at <a href="https://www.zettascale.tech">ZettaScale</a></sub>
</div>

## Demo

[![asciicast](https://asciinema.org/a/Uy6yvpT86vWzYW5DmWBfLcc8V.svg)](https://asciinema.org/a/Uy6yvpT86vWzYW5DmWBfLcc8V)

## Usage

Nuze is available on crates.io:

```bash
cargo install nuze --locked
```

A REPL instance supports multiple Zenoh sessions each identified with a name (a Nu string).
On startup, a session named `default` is created. All commands use this session unless
the argument `--session (-s)` is supplied:

```console
$ nuze
41aa8953> zenoh session list
╭───┬─────────┬──────────────────────────────────╮
│ # │  name   │               zid                │
├───┼─────────┼──────────────────────────────────┤
│ 0 │ default │ 41aa8953ad1abda60a9149e25c54067d │
╰───┴─────────┴──────────────────────────────────╯
41aa8953> zenoh zid -s default --short
41aa8953
```

If you would like to start Nuze without the `default` session, use the `--no-default-session (-0)` argument.

The Nuze CLI can be consulted with:

```console
nuze --help
```

To get the list of available commands:

```console
41aa8953> help zenoh
```

All `zenoh ...` commands are also available under the shorter `z ...` prefix:

```console
41aa8953> z session list
41aa8953> z zid --short
```

To get help on a specific command:

```console
41aa8953> help zenoh liveliness declare-token
```

## Version matrix

| Nuze version | Nushell version | Zenoh version |
| ------------ | --------------- | ------------- |
| v0.1.0       | 0.106.1         | 1.6.2         |
| v0.2.0       | 0.106.1         | 1.7.1         |
| v0.3.0       | 0.112.1         | 1.9.0         |
| v0.4.0       | 0.114.0         | 1.9.0         |
