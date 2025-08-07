import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const sidebar: SidebarsConfig = {
    apisidebar: [
        {
            type: 'doc',
            id: 'api/jsonplaceholder-api',
        },
        {
            type: 'category',
            label: 'Posts',
            items: [
                {
                    type: 'doc',
                    id: 'api/get-all-posts',
                    label: 'Get all posts',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'api/create-post',
                    label: 'Create a new post',
                    className: 'api-method post',
                },
                {
                    type: 'doc',
                    id: 'api/get-post-by-id',
                    label: 'Get a specific post',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'api/update-post',
                    label: 'Update a post',
                    className: 'api-method put',
                },
                {
                    type: 'doc',
                    id: 'api/patch-post',
                    label: 'Partially update a post',
                    className: 'api-method patch',
                },
                {
                    type: 'doc',
                    id: 'api/delete-post',
                    label: 'Delete a post',
                    className: 'api-method delete',
                },
            ],
        },
        {
            type: 'category',
            label: 'Comments',
            items: [
                {
                    type: 'doc',
                    id: 'api/get-comments-by-post-id',
                    label: 'Get comments for a post',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'api/get-all-comments',
                    label: 'Get all comments',
                    className: 'api-method get',
                },
            ],
        },
        {
            type: 'category',
            label: 'Users',
            items: [
                {
                    type: 'doc',
                    id: 'api/get-all-users',
                    label: 'Get all users',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'api/get-user-by-id',
                    label: 'Get a specific user',
                    className: 'api-method get',
                },
            ],
        },
        {
            type: 'category',
            label: 'Todos',
            items: [
                {
                    type: 'doc',
                    id: 'api/get-all-todos',
                    label: 'Get all todos',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'api/get-todo-by-id',
                    label: 'Get a specific todo',
                    className: 'api-method get',
                },
            ],
        },
    ],
};

export default sidebar.apisidebar;
