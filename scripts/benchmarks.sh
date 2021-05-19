#!/bin/bash

steps=50
repeat=20
statemineOutput=./runtime/statemine/src/weights/
statemintOutput=./runtime/statemint/src/weights/
statemineChain=statemine-dev
statemintChain=statemint-dev
pallets=(
	pallet_assets
	pallet_balances
	pallet_collator_selection
	pallet_multisig
	pallet_proxy
	pallet_timestamp
	pallet_utility
)

for p in ${pallets[@]}
do
	./target/release/statemint benchmark \
		--chain=$statemineChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p  \
		--extrinsic='*' \
		--steps=$steps  \
		--repeat=$repeat \
		--raw  \
		--output=$statemineOutput

	./target/release/statemint benchmark \
		--chain=$statemintChain \
		--execution=wasm \
		--wasm-execution=compiled \
		--pallet=$p  \
		--extrinsic='*' \
		--steps=$steps  \
		--repeat=$repeat \
		--raw  \
		--output=$statemintOutput

done