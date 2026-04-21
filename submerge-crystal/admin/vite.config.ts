import { defineConfig } from 'vite';
import browserslist from 'browserslist';
import { browserslistToTargets } from 'lightningcss';

const cssTargets = browserslistToTargets(
    browserslist(['last 2 versions', '> 1%', 'not dead', 'not ie 11']),
);

const jsTarget = 'es2022';

export default defineConfig({
    cacheDir: '.vite',
    server: {
        // auto-open the default browser on server start
        open: true,
        // if 5173 is in use, fail instead of picking another port
        strictPort: true,
        // listen on this port
        port: 5173,
    },
    preview: { port: 4173 },
    css: {
        transformer: 'lightningcss',
        lightningcss: {
            targets: cssTargets,
        },
    },
    build: {
        target: jsTarget,
        sourcemap: true,
        cssMinify: 'lightningcss',
        minify: 'oxc',
    },
    resolve: {
        alias: {
            '@': '/src',
        },
    },
});
