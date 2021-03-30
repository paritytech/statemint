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

Statemint is licensed under [Apache 2](LICENSE).

### Temp 

* Pointed the repo towards `rococo-v1` to get the latest version of assets and make it easy to track as the pallet changes 
* latest known working commit https://github.com/paritytech/substrate/commit/401c24e8a62cdf058882b0e92815faef966d9fa1
* polkadot needs to be built off of branch = 'rococo-v1'

* Polkadot launch can be run by dropping the proper polkadot binary in bin 
  * Run Globally 
    * polkadot-launch config.json
  * Run locally, navigate into polkadot-launch, 
    * ``` yarn ```
    * ``` yarn start ```

