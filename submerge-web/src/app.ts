import typescriptLogo from './typescript.svg';
import viteLogo from '/vite.svg';
import { setupCounter } from './counter';

import { AppEvent, eventBus, type EventMap } from './event/event';
import { logger } from './logger';
import { UIEvent } from './event/ui';
import { $counter } from './data/data-store';

class App {
    private unsubs: Array<() => void> = [];

    constructor() {}

    private async onPing(): Promise<void> {
        logger.info('Pong.');
    }

    private async onClose(event: EventMap[typeof AppEvent.Close]): Promise<void> {
        logger.info('Close', event.id);
    }

    private async onLayoutChange(event: EventMap[typeof UIEvent.Layout.Resize]): Promise<void> {
        logger.info('New height:', event.height);
    }

    async start() {
        document.querySelector<HTMLDivElement>('#app')!.innerHTML = `
            <div>
                <a href="https://vite.dev" target="_blank">
                <img src="${viteLogo}" class="logo" alt="Vite logo" />
                </a>
                <a href="https://www.typescriptlang.org/" target="_blank">
                <img src="${typescriptLogo}" class="logo vanilla" alt="TypeScript logo" />
                </a>
                <h1>Vite + TypeScript</h1>
                <div class="card">
                <button id="counter" type="button"></button>
                </div>
                <p class="read-the-docs">
                Click on the Vite and TypeScript logos to learn more
                </p>
            </div>
        `;
        setupCounter(document.querySelector<HTMLButtonElement>('#counter')!);

        try {
            eventBus.on(AppEvent.Ping, this.onPing);
            eventBus.on(AppEvent.Close, this.onClose);
            eventBus.on(UIEvent.Layout.Resize, this.onLayoutChange);

            const unsub = $counter.subscribe((value, oldValue) => {
                logger.info(`counter value changed from ${oldValue} to ${value}`);
            });
            this.unsubs.push(unsub);

            setTimeout(() => {
                eventBus.emit(AppEvent.Ping);
                eventBus.emit(AppEvent.Close, { id: 'close-id-200' });
                $counter.set($counter.get() + 2);
            }, 2500);

            setTimeout(() => {
                eventBus.emit(AppEvent.UI.Layout.Resize, { width: 1920, height: 1080 });
            }, 5000);
        } catch (error) {
            logger.error('Failed to start app:', error);
        }
    }

    async stop() {
        logger.info('Stop app.');
        eventBus.off(AppEvent.Ping, this.onPing);
        eventBus.off(AppEvent.Close, this.onClose);
        eventBus.off(UIEvent.Layout.Resize, this.onLayoutChange);
        for (const unsub of this.unsubs) {
            unsub();
        }
        this.unsubs = [];
        $counter.set(0);
    }
}

export { App };
