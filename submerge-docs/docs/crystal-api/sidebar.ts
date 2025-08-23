import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebar: SidebarsConfig = {
  apisidebar: [
    {
      type: "doc",
      id: "crystal-api/submerge-crystal-api-v-1",
    },
    {
      type: "category",
      label: "block",
      link: {
        type: "doc",
        id: "crystal-api/block",
      },
      items: [
        {
          type: "doc",
          id: "crystal-api/get-blocks",
          label: "Get blocks",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-block",
          label: "Get block by reference",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "trace",
      link: {
        type: "doc",
        id: "crystal-api/trace",
      },
      items: [
        {
          type: "doc",
          id: "crystal-api/get-block-traces",
          label: "Get traces in a block",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-traces",
          label: "Get traces",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-trace-by-id",
          label: "Get trace by id",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "call",
      link: {
        type: "doc",
        id: "crystal-api/call",
      },
      items: [
        {
          type: "doc",
          id: "crystal-api/get-block-calls",
          label: "Get calls in a block",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-block-extrinsic-calls",
          label: "Get block extrinsic calls",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-calls",
          label: "Get calls",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-call-by-id",
          label: "Get call by id",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-extrinsic-calls",
          label: "Get extrinsic calls",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "extrinsic",
      link: {
        type: "doc",
        id: "crystal-api/extrinsic",
      },
      items: [
        {
          type: "doc",
          id: "crystal-api/get-block-extrinsics",
          label: "Get extrinsics in a block",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-block-extrinsic",
          label: "Get block extrinsic by index",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-extrinsics",
          label: "Get extrinsics",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-extrinsic-by-ref",
          label: "Get extrinsic by reference",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "event",
      link: {
        type: "doc",
        id: "crystal-api/event",
      },
      items: [
        {
          type: "doc",
          id: "crystal-api/get-block-events",
          label: "Get events in a block",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-block-event",
          label: "Get block event by index",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-block-extrinsic-events",
          label: "Get block extrinsic events",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-events",
          label: "Get events",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-event-by-id",
          label: "Get event by id",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-extrinsic-events",
          label: "Get extrinsic events",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "genesis",
      link: {
        type: "doc",
        id: "crystal-api/genesis",
      },
      items: [
        {
          type: "doc",
          id: "crystal-api/get-genesis",
          label: "Get genesis records",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "metadata",
      link: {
        type: "doc",
        id: "crystal-api/metadata",
      },
      items: [
        {
          type: "doc",
          id: "crystal-api/get-metadata-list",
          label: "Get metadata list",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-metadata-json",
          label: "Get metadata JSON",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "Schemas",
      items: [
        {
          type: "doc",
          id: "crystal-api/schemas/error",
          label: "Error",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/paginationdata",
          label: "PaginationData",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/hexstring",
          label: "HexString",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/accountidhex",
          label: "AccountIdHex",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/hash-256-hex",
          label: "Hash256Hex",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/signaturehex",
          label: "SignatureHex",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/blockstatus",
          label: "BlockStatus",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/tracemethod",
          label: "TraceMethod",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/block",
          label: "Block",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/trace",
          label: "Trace",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/call",
          label: "Call",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/event",
          label: "Event",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/extrinsic",
          label: "Extrinsic",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/genesisrecord",
          label: "GenesisRecord",
          className: "schema",
        },
        {
          type: "doc",
          id: "crystal-api/schemas/metadatasummary",
          label: "MetadataSummary",
          className: "schema",
        },
      ],
    },
  ],
};

export default sidebar.apisidebar;
