import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const crystalAPISidebar = require('./docs/crystal-api/sidebar.ts');

const sidebars: SidebarsConfig = {
    introductionSidebar: [
        {
            type: 'doc',
            id: 'introduction',
            label: 'Introduction',
        },
        {
            type: 'doc',
            id: 'mycelium-spec',
            label: 'Mycelium Specification',
        },
    ],
    crystalAPISidebar,
};

export default sidebars;
