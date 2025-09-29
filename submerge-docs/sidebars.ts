import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

function headerifyTopCategories(items: any[]): any[] {
    return items.map((item) => {
        if (item.type === 'category') {
            // remove link & collapse behaviour
            const { link, ...rest } = item;
            return {
                ...rest,
                link,
                collapsible: false,
                collapsed: false, // has no effect if collapsible is false
                items: headerifyTopCategories(item.items),
            };
        }
        return item;
    });
}

const crystalAPISidebar = require('./docs/crystal-api/sidebar.ts');

const sidebars: SidebarsConfig = {
    introductionSidebar: [
        {
            type: 'doc',
            id: 'introduction',
            label: 'Introduction',
        },
    ],
    crystalAPISidebar: headerifyTopCategories(crystalAPISidebar),
};

export default sidebars;
