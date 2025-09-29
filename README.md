<p align="center">
	<img width="600" src="https://raw.githubusercontent.com/helikon-labs/submerge/main/_readme-files/submerge-logo.png">
</p>

![](https://github.com/helikon-labs/submerge/actions/workflows/rust_checks.yml/badge.svg)

[Submerge](https://submerge.io) is a multi-function blockchain data indexing, processing, analysis, and compliance platform. Below is a list of the components of Submerge:

1. **Crystal:** The main indexer component. Indexes genesis records, blocks, extrinsics, calls within extrinsics, events, traces, logs, and metadata (versions, pallets, calls, events, constants, errors). Deployed per chain.
2. **Fractal:** Processes the output of Crystal to index selected storage items into a queryable format. Also indexes data related to state transitions, such as block headers, extrinsics, events, etc. Supports backfilling. Deployed per chain.
3. **Bloom:** Transforms the output of Fractal into queryable relational models, as defined by DSL specifications. Supports backfilling. Deployed per chain.
4. **Mycelium:** Processes data from all deployed chains to index and track cross-chain data (XCM). Single deployment.
5. **Cortex:** The component that hosts and executes data analysis algorithms.
6. **Reflex:** The component responsible for real-time analysis of multi-chain state.
7. **Sentinel:** The security and compliance protocol interfacing with providers such as Merkle Science, Chainalysis, Scorechain, and OFAC API.
8. **Auth3:** Provides authentication and authorisation via wallet signatures and predefined access rules.
9. **API:** Provides unified access to all Submerge component APIs. At this release date, it will provide access to the Crystal API, through which users can access all data made available by this component.
10. **Web:** The grid-based front-end application for the complete suite.