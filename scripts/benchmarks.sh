#!/bin/bash

steps=20
repeat=10
statemineOutput=./runtime/statemine/src/weights/
statemintOutput=./runtime/statemint/src/weights/
statemineChain=statemine-dev
statemintChain=statemint-dev

./target/release/statemint benchmark \
	--chain $statemineChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_assets  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemineOutput

./target/release/statemint benchmark \
	--chain $statemineChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_balances  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemineOutput

./target/release/statemint benchmark \
	--chain $statemineChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_multisig  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemineOutput

./target/release/statemint benchmark \
	--chain $statemineChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_proxy  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemineOutput

./target/release/statemint benchmark \
	--chain $statemineChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_utility  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemineOutput

./target/release/statemint benchmark \
	--chain $statemineChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_timestamp \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemineOutput

./target/release/statemint benchmark \
	--chain $statemineChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_collator_selection  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemineOutput

./target/release/statemint benchmark \
	--chain $statemintChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_assets  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemintOutput

./target/release/statemint benchmark \
	--chain $statemintChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_balances  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemintOutput

./target/release/statemint benchmark \
	--chain $statemintChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_multisig  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemintOutput

./target/release/statemint benchmark \
	--chain $statemintChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_proxy  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemintOutput

./target/release/statemint benchmark \
	--chain $statemintChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_utility  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemintOutput

./target/release/statemint benchmark \
	--chain $statemintChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_timestamp \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemintOutput

./target/release/statemint benchmark \
	--chain $statemintChain \
	--execution wasm \
	--wasm-execution compiled \
	--pallet pallet_collator_selection  \
	--extrinsic '*' \
	--steps $steps  \
	--repeat $repeat \
	--raw  \
	--output $statemintOutput