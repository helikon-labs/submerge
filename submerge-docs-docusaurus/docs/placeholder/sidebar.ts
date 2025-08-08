import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const sidebar: SidebarsConfig = {
    apisidebar: [
        {
            type: 'doc',
            id: 'placeholder/jsonplaceholder-api',
        },
        {
            type: 'category',
            label: 'Posts',
            link: {
                type: 'doc',
                id: 'placeholder/posts',
            },
            items: [
                {
                    type: 'doc',
                    id: 'placeholder/get-all-posts',
                    label: 'Get all posts',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'placeholder/create-post',
                    label: 'Create a new post',
                    className: 'api-method post',
                },
                {
                    type: 'doc',
                    id: 'placeholder/get-post-by-id',
                    label: 'Get a specific post',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'placeholder/update-post',
                    label: 'Update a post',
                    className: 'api-method put',
                },
                {
                    type: 'doc',
                    id: 'placeholder/patch-post',
                    label: 'Partially update a post',
                    className: 'api-method patch',
                },
                {
                    type: 'doc',
                    id: 'placeholder/delete-post',
                    label: 'Delete a post',
                    className: 'api-method delete',
                },
            ],
        },
        {
            type: 'category',
            label: 'Comments',
            link: {
                type: 'doc',
                id: 'placeholder/comments',
            },
            items: [
                {
                    type: 'doc',
                    id: 'placeholder/get-comments-by-post-id',
                    label: 'Get comments for a post',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'placeholder/get-all-comments',
                    label: 'Get all comments',
                    className: 'api-method get',
                },
            ],
        },
        {
            type: 'category',
            label: 'Users',
            link: {
                type: 'doc',
                id: 'placeholder/users',
            },
            items: [
                {
                    type: 'doc',
                    id: 'placeholder/get-all-users',
                    label: 'Get all users',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'placeholder/get-user-by-id',
                    label: 'Get a specific user',
                    className: 'api-method get',
                },
            ],
        },
        {
            type: 'category',
            label: 'Todos',
            link: {
                type: 'doc',
                id: 'placeholder/todos',
            },
            items: [
                {
                    type: 'doc',
                    id: 'placeholder/get-all-todos',
                    label: 'Get all todos',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'placeholder/get-todo-by-id',
                    label: 'Get a specific todo',
                    className: 'api-method get',
                },
            ],
        },
    ],
};

export default sidebar.apisidebar;
