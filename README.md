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

### F.A.Q.

As Statemint will likely be one of the first common good parachains, we've received a lot of
questions about how it will function and its place in the ecosystem.

#### Will Statemint be on Kusama and Polkadot?

Yes, Statemint is the Polkadot parachain, while _Statemine_ is the Kusama parachain. All of the
other answers apply equally to Statemine (but will reference Statemint/Polkadot for brevity).

#### How will Statemint get a parachain slot?

We are building Statemint with the intent to make a governance proposal for a parachain slot. The
proposal will go to referendum where Polkadot's stakeholders will decide. For more info on common
good parachains, see [this blog
article](https://polkadot.network/common-good-parachains-an-introduction-to-governance-allocated-parachain-slots/).

#### How will I use Statemint?

Statemint will be a parachain that uses the DOT token as its native token, i.e. represented in its
instance of the Balance pallet. In order to make transactions on Statemint, you will need to first
send some DOT from your Relay Chain account to your Statemint account using a cross-chain message.
Addresses on Statemint will use the same SS58 prefix as its Relay Chain. Note that this might not
be "end user friendly" until some user interfaces handle the cross-chain message.

One way to do this would be to have a back end that connects to both the Relay Chain and Statemint.
When the user tries to make a transaction on Statemint, the app would realize that it needed a
balance there and handle sending the cross-chain message to transfer balances, waiting for its
success, and then broadcasting the Statemint transaction, all in one click for the user. If you
want to contribute, building infrastructure and UIs that can handle applications with multi-chain
back ends would be a great contribution, not only for Statemint.

#### What will the fees be like?

The deposits and fees in the Statemint runtime are set to 10% of the levels of the Relay Chain.
That is, generally speaking, transaction fees should be approximately\* 1/10 of what they would be
on the Relay Chain (and likewise for deposits such as proxy and multisig). The exception here is the
_existential deposit,_ which remains equivalent to the Relay Chain's for user sanity.

\* They will not be exactly 1/10. Parachains have lower weight limits per block than the Relay
Chain, and fees change depending on block fullness. So if Statemint blocks are more full than the
Relay Chain blocks for some period of time, the fees would be higher than 1/10 those of the Relay
Chain.

#### Can I run a collator?

Yes, Statemint will have very simple logic that will allow one to become a collator for a fixed
bond. Note that there are no inflationary rewards for collators; they only receive a portion of the
transaction fees. At the time of this writing, Aura is not yet working for parachains, so please be
patient as we scale up the number of collator slots with this new capability.

#### Will Statemint support smart contracts?

No, Statemint supports specialized logic for handling assets. It will not provide any smart
contract interface.

#### Will Statemint support NFTs?

Eventually, yes, but probably not in the first version to hit Kusama and Polkadot. See the [token
tracking issue](https://github.com/paritytech/substrate/issues/8453) for the roadmap on support for
fungible and non-fungible features.

#### Will Statemint compete with {Smart contract chain, NFT chain, etc.}?

Statemint is a very basic chain that only provides an interface for representing assets and some
primitive functions for handling them. Tokens tend to bloat smart contract chains with contract
storage and metering transactions that have known complexity. By encoding this logic directly into
the Statemint runtime, token storage and actions can happen faster and cheaper.

This allows other chains to specialize in what they are good at, and not be weighed down by token
operations. Smart contract chains specialize in allowing anyone to deploy untrusted code to a
global system. NFT platforms specialize in their ability to foster communities, marketplaces, and
galleries. Statemint can store the low-level interfaces for tokens, while other systems can write
the logic to interact with them.

For example, operations from contract execution could trigger a cross-chain message that lets
Statemint handle the token transactions that are a result of the contract execution. This will
reduce wasted gas and keep fees lower on the smart contract chain. Likewise, an NFT chain or
platform can focus on its business logic of representing the interactions of its community members
and what the conditions are for transferring an NFT, but the data structures and primitive
functionality of NFTs can be on Statemint for multiple communities to use.

#### How will Statemint be governed?

As a common good parachain, Statemint must stay fully aligned with the Relay Chain. Upgrades to
Statemint will require the Relay Chain's "root origin", i.e. a referendum. Some of the other logic
(like privileged asset functionality) will defer to the Relay Chain's Council, which can always be
superceded by root.

## Contributing

- [Contribution Guidelines](CONTRIBUTING.md)
- [Code of Conduct](CODE_OF_CONDUCT.md)

## License

Statemint is licensed under [Apache 2](LICENSE).

### Temp

* Pointed the repo towards `rococo-v1` branch.

__Commits:__
```
"git+https://github.com/paritytech/substrate.git?branch=rococo-v1#2be8fcc4236d32786c62f6f27a98e7fe7e550807"
"git+https://github.com/paritytech/polkadot?branch=rococo-v1#127eb17a25bbe2a9f2731ff11a65d7f8170f2373"
"git+https://github.com/paritytech/cumulus.git?branch=rococo-v1#da4c3bac6e9584e65740ef5db4dbd2c31c1a91db"
"git+https://github.com/paritytech/grandpa-bridge-gadget?branch=rococo-v1#b0e5f2da52cc9bc9804a23e111d003413b268faf"
```

* **Polkadot launch** can be run by dropping the proper polkadot binary in bin 
  * Run Globally
    * polkadot-launch config.json
  * Run locally, navigate into polkadot-launch,
    * ``` yarn ```
    * ``` yarn start ```

### Benchmarks 

* TODO
 - [] choose steps and repeats
 - [] run benchmarks on proper machine specs

 * running 
    * from root run ```./scripts/benchmarks```
