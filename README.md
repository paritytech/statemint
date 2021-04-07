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

* Pointed the repo towards `rococo-v1` branch.

__Commits:__
```
"git+https://github.com/paritytech/substrate.git?branch=rococo-v1#3ec97a31b285181fb1beab426ee2a8e2cb1188ab"
"git+https://github.com/paritytech/polkadot?branch=rococo-v1#a67612b3fc4bca05c324032faa9108a414e9be61"
"git+https://github.com/paritytech/cumulus.git?branch=rococo-v1#651abcedba1d88f5c00c47d2715820c3bfac9c15"
```

* **Polkadot launch** can be run by dropping the proper polkadot binary in bin 
  * Run Globally 
    * polkadot-launch config.json
  * Run locally, navigate into polkadot-launch, 
    * ``` yarn ```
    * ``` yarn start ```

