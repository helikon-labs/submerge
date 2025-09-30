import { themes as prismThemes } from 'prism-react-renderer';
import type { Config } from '@docusaurus/types';
import type * as Preset from '@docusaurus/preset-classic';
import type * as Plugin from '@docusaurus/types/src/plugin';
import type * as OpenApiPlugin from 'docusaurus-plugin-openapi-docs';
import remarkMath from 'remark-math';
import rehypeKatex from 'rehype-katex';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
    title: 'Submerge Developer Documentation',
    tagline: 'Blockchain data simplified',
    favicon: 'img/submerge_icon_circular.png',

    headTags: [
        {
            tagName: 'link',
            attributes: {
                rel: 'preconnect',
                href: 'https://fonts.googleapis.com',
            },
        },
        {
            tagName: 'link',
            attributes: {
                rel: 'preconnect',
                href: 'https://fonts.gstatic.com',
                crossorigin: 'anonymous',
            },
        },
        {
            tagName: 'link',
            attributes: {
                rel: 'stylesheet',
                href: 'https://fonts.googleapis.com/css2?family=Inter:ital,opsz,wght@0,14..32,100..900;1,14..32,100..900&display=swap',
            },
        },
    ],

    // Future flags, see https://docusaurus.io/docs/api/docusaurus-config#future
    future: {
        v4: true, // Improve compatibility with the upcoming Docusaurus v4
    },

    // Set the production url of your site here
    url: 'https://developer.submerge.io',
    // Set the /<baseUrl>/ pathname under which your site is served
    // For GitHub pages deployment, it is often '/<projectName>/'
    baseUrl: '/',

    onBrokenLinks: 'throw',

    // Even if you don't use internationalization, you can use this field to set
    // useful metadata like html lang. For example, if your site is Chinese, you
    // may want to replace "en" with "zh-Hans".
    i18n: {
        defaultLocale: 'en',
        locales: ['en'],
    },

    presets: [
        [
            'classic',
            {
                docs: {
                    sidebarPath: './sidebars.ts',
                    // Please change this to your repo.
                    // Remove this to remove the "edit this page" links.
                    editUrl: 'https://github.com/helikon-labs/submerge/tree/main/submerge-docs/',
                    docItemComponent: '@theme/ApiItem',
                    routeBasePath: '/',
                    remarkPlugins: [remarkMath],
                    rehypePlugins: [rehypeKatex],
                },
                theme: {
                    customCss: './src/css/custom.css',
                },
            } satisfies Preset.Options,
        ],
    ],

    themeConfig: {
        // Replace with your project's social card
        image: 'img/docusaurus-social-card.jpg',
        colorMode: {
            defaultMode: 'light',
            disableSwitch: false,
            respectPrefersColorScheme: false,
        },
        navbar: {
            title: null,
            logo: {
                alt: 'Submerge Developer Documentation',
                src: 'img/submerge_logo_black.png',
            },
            items: [
                /*
                {
                    type: 'docsVersionDropdown',
                    position: 'left',
                },
                */
                {
                    href: 'https://github.com/helikon-labs/submerge',
                    label: 'GitHub',
                    position: 'left',
                },
                {
                    type: 'doc',
                    label: 'Introduction',
                    docId: 'introduction',
                    position: 'right',
                },
                {
                    label: 'Crystal API Reference',
                    position: 'right',
                    to: '/crystal-api',
                },
            ],
        },
        footer: {
            style: 'dark',
            links: [
                {
                    title: 'Docs',
                    items: [
                        {
                            label: 'Introduction',
                            to: '/',
                        },
                        {
                            label: 'Crystal API',
                            to: '/crystal-api',
                        },
                    ],
                },
                {
                    title: 'Links',
                    items: [
                        {
                            label: 'Submerge',
                            href: 'https://discordapp.com/invite/docusaurus',
                        },
                        {
                            label: 'Helikon',
                            href: 'https://helikon.io',
                        },
                        {
                            label: 'Helikon X',
                            href: 'https://x.com/helikonlabs',
                        },
                    ],
                },
                {
                    title: 'More',
                    items: [
                        {
                            label: 'Submerge GitHub',
                            href: 'https://github.com/helikon-labs/submerge',
                        },
                        {
                            label: 'Helikon GitHub',
                            href: 'https://github.com/helikon-labs',
                        },
                    ],
                },
            ],
            copyright: `Copyright © ${new Date().getFullYear()} Helikon`,
        },
        prism: {
            theme: prismThemes.github,
            darkTheme: prismThemes.dracula,
        },
    } satisfies Preset.ThemeConfig,

    plugins: [
        [
            'docusaurus-plugin-openapi-docs',
            {
                id: 'openapi',
                docsPluginId: 'classic',
                config: {
                    crystal: {
                        specPath: '../submerge-crystal/api-spec/submerge-crystal-api.yaml',
                        outputDir: './docs/crystal-api',
                        infoTemplate: './templates/crystal-api-info.mustache',
                        sidebarOptions: {
                            groupPathsBy: 'tag',
                            categoryLinkSource: undefined,
                            sidebarCollapsible: false,
                            sidebarCollapsed: false,
                        },
                        showSchemas: true,
                    } satisfies OpenApiPlugin.Options,
                } satisfies Plugin.PluginOptions,
            },
        ],
    ],
    themes: ['docusaurus-theme-openapi-docs'],
    stylesheets: [
        {
            href: 'https://cdn.jsdelivr.net/npm/katex@0.13.24/dist/katex.min.css',
            type: 'text/css',
            integrity: 'sha384-odtC+0UGzzFL/6PNoE8rX/SPcQDXBJ+uRepguP4QkPCm2LBxH3FA3y+fKSiJ+AmM',
            crossorigin: 'anonymous',
        },
    ],
    markdown: {
        hooks: {
            onBrokenMarkdownLinks: 'warn',
        },
    },
};

export default config;
