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
          id: "crystal-api/get-all-blocks",
          label: "Get all blocks",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "crystal-api/get-blocks-by-hash-or-number",
          label: "Get block(s) by hash or number.",
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
          id: "crystal-api/schemas/paginationdata",
          label: "PaginationData",
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
          id: "crystal-api/schemas/blockstatus",
          label: "BlockStatus",
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
          id: "crystal-api/schemas/paginatedblocklist",
          label: "PaginatedBlockList",
          className: "schema",
        },
      ],
    },
  ],
};

export default sidebar.apisidebar;
