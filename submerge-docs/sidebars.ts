import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

const crystalAPISidebar = require('./docs/crystal-api/sidebar.ts');

const sidebars: SidebarsConfig = {
    introductionSidebar: [
        {
            type: 'doc',
            id: 'introduction',
            label: 'Introduction',
        },
    ],
    crystalAPISidebar: crystalAPISidebar,
};

export default sidebars;
