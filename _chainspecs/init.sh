#!/usr/bin/env bash
# coretime westend
cd "$(dirname "$0")"

# polkadot
mkdir -p polkadot
curl -o ./polkadot/polkadot.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/polkadot/node/service/chain-specs/polkadot.json
# polkadot system chains
mkdir -p polkadot/sys
curl -o ./polkadot/sys/asset-hub-polkadot.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/cumulus/parachains/chain-specs/asset-hub-polkadot.json
curl -o ./polkadot/sys/bridge-hub-polkadot.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/cumulus/parachains/chain-specs/bridge-hub-polkadot.json
curl -o ./polkadot/sys/collectives-polkadot.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/cumulus/parachains/chain-specs/collectives-polkadot.json
curl -o ./polkadot/sys/coretime-polkadot.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/cumulus/parachains/chain-specs/coretime-polkadot.json
curl -o ./polkadot/sys/people-polkadot.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/cumulus/parachains/chain-specs/people-polkadot.json
# polkadot parachains
mkdir -p polkadot/para
curl -o ./polkadot/para/astar.json https://raw.githubusercontent.com/AstarNetwork/Astar/refs/heads/master/bin/collator/res/astar.raw.json
curl -o ./polkadot/para/bifrost-polkadot.json https://raw.githubusercontent.com/bifrost-io/bifrost/refs/heads/develop/node/service/res/bifrost-polkadot.json
curl -o ./polkadot/para/centrifuge.json https://raw.githubusercontent.com/centrifuge/centrifuge-chain/refs/heads/main/node/res/genesis/centrifuge-genesis-spec-raw.json
curl -o ./polkadot/para/hydration.json https://raw.githubusercontent.com/galacticcouncil/hydration-node/refs/heads/master/node/res/hydradx.json
curl -o ./polkadot/para/moonbeam.json https://raw.githubusercontent.com/moonbeam-foundation/moonbeam/refs/heads/master/specs/moonbeam/parachain-embedded-specs.json
curl -o ./polkadot/para/mythos.json https://raw.githubusercontent.com/paritytech/project-mythical/refs/heads/main/chainspecs/mythos-raw.json
curl -o ./polkadot/para/hyperbridge-nexus.json https://raw.githubusercontent.com/polytope-labs/hyperbridge/refs/heads/main/parachain/chainspec/nexus.json

# kusama
mkdir -p kusama
curl -o ./kusama/kusama.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/polkadot/node/service/chain-specs/kusama.json
# kusama system chains
mkdir -p kusama/sys
curl -o ./kusama/sys/asset-hub-kusama.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/cumulus/parachains/chain-specs/asset-hub-kusama.json
curl -o ./kusama/sys/bridge-hub-kusama.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/cumulus/parachains/chain-specs/bridge-hub-kusama.json
curl -o ./kusama/sys/coretime-kusama.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/cumulus/parachains/chain-specs/coretime-kusama.json
curl -o ./kusama/sys/people-kusama.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/cumulus/parachains/chain-specs/people-kusama.json
# kusama parachains
mkdir -p kusama/para
curl -o ./kusama/para/bifrost-kusama.json https://raw.githubusercontent.com/bifrost-io/bifrost/refs/heads/develop/node/service/res/bifrost-kusama.json
curl -o ./kusama/para/moonriver.json https://raw.githubusercontent.com/moonbeam-foundation/moonbeam/refs/heads/master/specs/moonriver/parachain-embedded-specs.json

# westend
mkdir -p westend
curl -o ./westend/westend.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/polkadot/node/service/chain-specs/westend.json
# westend system chains
mkdir -p westend/sys
curl -o ./westend/sys/coretime-westend.json https://raw.githubusercontent.com/paritytech/polkadot-sdk/refs/heads/master/cumulus/parachains/chain-specs/coretime-westend.json