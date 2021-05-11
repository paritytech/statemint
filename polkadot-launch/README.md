# polkadot-launch

Simple CLI tool to launch a local [Polkadot](https://github.com/paritytech/polkadot/) test network.

## Notes

- These is adapted instructions for quickly starting statemint. For the original README consult https://github.com/paritytech/polkadot-launch#readme

## Building binaries

To use polkadot-launch, you need to have binary files for a `polkadot` relay chain and a
`statemint` collator.

You can generate these files by cloning the `rococo-v1` branch of these projects and building them
with the specific flags below:

```bash
git clone -b statemint https://github.com/paritytech/polkadot
cd polkadot
cargo build --release
```

and in the root directory of this repo

```bash
cargo build --release
```

## Use

### Setting up config.json

Modify the `config.json` in this repo's root to point to your polkadot binary built from the
`statemint` branch:

```json
{
  "relaychain": {
    "bin": "<path to polkadot binary>",
    ...
  }
  ...
}
```

### Start up polkadot-launch

```bash
cd polkadot-launch
yarn
yarn start
```
