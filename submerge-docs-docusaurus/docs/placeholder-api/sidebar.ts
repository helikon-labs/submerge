import type { SidebarsConfig } from "@docusaurus/plugin-content-docs";

const sidebar: SidebarsConfig = {
  apisidebar: [
    {
      type: "doc",
      id: "placeholder-api/jsonplaceholder-api",
    },
    {
      type: "category",
      label: "Posts",
      link: {
        type: "doc",
        id: "placeholder-api/posts",
      },
      items: [
        {
          type: "doc",
          id: "placeholder-api/get-all-posts",
          label: "Get all posts",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "placeholder-api/create-post",
          label: "Create a new post",
          className: "api-method post",
        },
        {
          type: "doc",
          id: "placeholder-api/get-post-by-id",
          label: "Get a specific post",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "placeholder-api/update-post",
          label: "Update a post",
          className: "api-method put",
        },
        {
          type: "doc",
          id: "placeholder-api/patch-post",
          label: "Partially update a post",
          className: "api-method patch",
        },
        {
          type: "doc",
          id: "placeholder-api/delete-post",
          label: "Delete a post",
          className: "api-method delete",
        },
      ],
    },
    {
      type: "category",
      label: "Comments",
      link: {
        type: "doc",
        id: "placeholder-api/comments",
      },
      items: [
        {
          type: "doc",
          id: "placeholder-api/get-comments-by-post-id",
          label: "Get comments for a post",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "placeholder-api/get-all-comments",
          label: "Get all comments",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "Users",
      link: {
        type: "doc",
        id: "placeholder-api/users",
      },
      items: [
        {
          type: "doc",
          id: "placeholder-api/get-all-users",
          label: "Get all users",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "placeholder-api/get-user-by-id",
          label: "Get a specific user",
          className: "api-method get",
        },
      ],
    },
    {
      type: "category",
      label: "Todos",
      link: {
        type: "doc",
        id: "placeholder-api/todos",
      },
      items: [
        {
          type: "doc",
          id: "placeholder-api/get-all-todos",
          label: "Get all todos",
          className: "api-method get",
        },
        {
          type: "doc",
          id: "placeholder-api/get-todo-by-id",
          label: "Get a specific todo",
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
          id: "placeholder-api/schemas/post",
          label: "Post",
          className: "schema",
        },
        {
          type: "doc",
          id: "placeholder-api/schemas/postinput",
          label: "PostInput",
          className: "schema",
        },
        {
          type: "doc",
          id: "placeholder-api/schemas/postupdate",
          label: "PostUpdate",
          className: "schema",
        },
        {
          type: "doc",
          id: "placeholder-api/schemas/postpatch",
          label: "PostPatch",
          className: "schema",
        },
        {
          type: "doc",
          id: "placeholder-api/schemas/comment",
          label: "Comment",
          className: "schema",
        },
        {
          type: "doc",
          id: "placeholder-api/schemas/user",
          label: "User",
          className: "schema",
        },
        {
          type: "doc",
          id: "placeholder-api/schemas/address",
          label: "Address",
          className: "schema",
        },
        {
          type: "doc",
          id: "placeholder-api/schemas/geo",
          label: "Geo",
          className: "schema",
        },
        {
          type: "doc",
          id: "placeholder-api/schemas/company",
          label: "Company",
          className: "schema",
        },
        {
          type: "doc",
          id: "placeholder-api/schemas/todo",
          label: "Todo",
          className: "schema",
        },
      ],
    },
  ],
};

export default sidebar.apisidebar;
