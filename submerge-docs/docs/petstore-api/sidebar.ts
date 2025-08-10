import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const sidebar: SidebarsConfig = {
    apisidebar: [
        {
            type: 'doc',
            id: 'petstore-api/swagger-petstore-yaml',
        },
        {
            type: 'category',
            label: 'Pets',
            link: {
                type: 'doc',
                id: 'petstore-api/pet',
            },
            items: [
                {
                    type: 'doc',
                    id: 'petstore-api/schemas/cat',
                    label: 'Cat',
                    className: 'schema',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/add-pet',
                    label: 'Add a new pet to the store',
                    className: 'api-method post',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/update-pet',
                    label: 'Update an existing pet',
                    className: 'api-method put',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/get-pet-by-id',
                    label: 'Find pet by ID',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/update-pet-with-form',
                    label: 'Updates a pet in the store with form data',
                    className: 'api-method post',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/delete-pet',
                    label: 'Deletes a pet',
                    className: 'api-method delete',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/upload-file',
                    label: 'uploads an image',
                    className: 'api-method post',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/find-pets-by-status',
                    label: 'Finds Pets by status',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/find-pets-by-tags',
                    label: 'Finds Pets by tags',
                    className: 'menu__list-item--deprecated api-method get',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/new-pet',
                    label: 'New pet',
                    className: 'api-method event',
                },
            ],
        },
        {
            type: 'category',
            label: 'Petstore Orders',
            link: {
                type: 'doc',
                id: 'petstore-api/store',
            },
            items: [
                {
                    type: 'doc',
                    id: 'petstore-api/get-inventory',
                    label: 'Returns pet inventories by status',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/place-order',
                    label: 'Place an order for a pet',
                    className: 'api-method post',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/get-order-by-id',
                    label: 'Find purchase order by ID',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/delete-order',
                    label: 'Delete purchase order by ID',
                    className: 'api-method delete',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/subscribe-to-the-store-events',
                    label: 'Subscribe to the Store events',
                    className: 'api-method post',
                },
            ],
        },
        {
            type: 'category',
            label: 'Users',
            link: {
                type: 'doc',
                id: 'petstore-api/user',
            },
            items: [
                {
                    type: 'doc',
                    id: 'petstore-api/create-user',
                    label: 'Create user',
                    className: 'api-method post',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/get-user-by-name',
                    label: 'Get user by user name',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/update-user',
                    label: 'Updated user',
                    className: 'api-method put',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/delete-user',
                    label: 'Delete user',
                    className: 'api-method delete',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/create-users-with-array-input',
                    label: 'Creates list of users with given input array',
                    className: 'api-method post',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/create-users-with-list-input',
                    label: 'Creates list of users with given input list',
                    className: 'api-method post',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/login-user',
                    label: 'Logs user into the system',
                    className: 'api-method get',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/logout-user',
                    label: 'Logs out current logged in user session',
                    className: 'api-method get',
                },
            ],
        },
        {
            type: 'category',
            label: 'Schemas',
            items: [
                {
                    type: 'doc',
                    id: 'petstore-api/schemas/apiresponse',
                    label: 'ApiResponse',
                    className: 'schema',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/schemas/category',
                    label: 'Category',
                    className: 'schema',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/schemas/dog',
                    label: 'Dog',
                    className: 'schema',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/schemas/honeybee',
                    label: 'HoneyBee',
                    className: 'schema',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/schemas/id',
                    label: 'Id',
                    className: 'schema',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/schemas/order',
                    label: 'Order',
                    className: 'schema',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/schemas/pet',
                    label: 'Pet',
                    className: 'schema',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/schemas/tag',
                    label: 'Tag',
                    className: 'schema',
                },
                {
                    type: 'doc',
                    id: 'petstore-api/schemas/user',
                    label: 'User',
                    className: 'schema',
                },
            ],
        },
    ],
};

export default sidebar.apisidebar;
