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
      ],
    },
    {
      type: "category",
      label: "Schemas",
      items: [
        {
          type: "doc",
          id: "crystal-api/schemas/chainslug",
          label: "ChainSlug",
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
