#!/usr/bin/env bash
set -e

# polkadot
subxt metadata --url "wss://rpc.helikon.io/polkadot" -o ./polkadot.scale
subxt codegen --file ./polkadot.scale --no-docs --derive ::subxt::ext::subxt_core::ext::codec::Encode --derive ::subxt::ext::subxt_core::ext::codec::Decode | rustfmt --edition=2021 --emit=stdout > ../dv-report-metadata/src/runtime/polkadot.rs
rm ./polkadot.scale

# polkadot current
subxt metadata --url "wss://rpc.helikon.io/polkadot" --pallets Referenda,ConvictionVoting,Utility,Multisig,Proxy -o ./polkadot.scale
subxt codegen --file ./polkadot.scale --no-docs --derive ::subxt::ext::subxt_core::ext::codec::Encode --derive ::subxt::ext::subxt_core::ext::codec::Decode | rustfmt --edition=2021 --emit=stdout > ../dv-report-metadata/src/runtime/polkadot_current.rs
rm ./polkadot.scale

# kusama
subxt metadata --url "wss://rpc.helikon.io/kusama" -o ./kusama.scale
subxt codegen --file ./kusama.scale --no-docs --derive ::subxt::ext::subxt_core::ext::codec::Encode --derive ::subxt::ext::subxt_core::ext::codec::Decode | rustfmt --edition=2021 --emit=stdout > ../dv-report-metadata/src/runtime/kusama.rs
rm ./kusama.scale

# kusama current
subxt metadata --url "wss://rpc.helikon.io/kusama" --pallets Referenda,ConvictionVoting,Utility,Multisig,Proxy -o ./kusama.scale
subxt codegen --file ./kusama.scale --no-docs --derive ::subxt::ext::subxt_core::ext::codec::Encode --derive ::subxt::ext::subxt_core::ext::codec::Decode  | rustfmt --edition=2021 --emit=stdout > ../dv-report-metadata/src/runtime/kusama_current.rs
rm ./kusama.scale