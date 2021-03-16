# Statemint

Implementation of _Statemint,_ a blockchain to support generic assets in the Polkadot and Kusama
networks.

Statemint will allow users to:

- Deploy promise-backed assets with a DOT/KSM deposit.
- Set admin roles for an `AssetId` to mint, freeze, thaw, and burn tokens.
- Register assets as "self-sufficient" if the Relay Chain agrees, i.e. the ability for an account
  to exist without DOT/KSM so long as it maintains a minimum token balance.
- Pay fees using asset balances.
- Transfer (and approve transfer) assets.

Statemint must stay fully aligned with the Relay Chain it is connected to. As such, it will accept
the Relay Chain's governance origins as its own.

## Contributing

- [Contribution Guidelines](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## License

Statemint is licensed under [GPL 3.0](LICENSE)


### Temp 

* Pointed the repo towards master to get the latest version of assets and make it easy to track as the pallet changes 
* latest working commit https://github.com/paritytech/substrate/commit/24df4c50cb971fdfc9188d0009ef9a49eea2ce88