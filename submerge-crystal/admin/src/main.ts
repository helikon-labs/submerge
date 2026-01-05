import './style.css';
import { App } from '@/app';

let app: App | undefined;

function bootstrap() {
    if (!app) {
        app = new App();
        app.start();
    }
}

bootstrap();

if (import.meta.hot) {
    import.meta.hot.dispose(() => {
        try {
            app?.stop?.();
        } finally {
            app = undefined;
        }
    });

    // re-evaluate this module on update - top-level bootstrap() will run again
    import.meta.hot.accept();
}
