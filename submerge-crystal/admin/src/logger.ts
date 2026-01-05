/* eslint-disable no-console */
export const logger = {
    info: (message: string, ...args: unknown[]) => console.log(message, ...args),
    warn: (message: string, ...args: unknown[]) => console.warn(message, ...args),
    error: (message: string, ...args: unknown[]) => console.error(message, ...args),
    debug: (message: string, ...args: unknown[]) => {
        if (import.meta.env.DEV) {
            console.log(`[DEBUG] ${message}`, ...args);
        }
    },
};
/* eslint-enable no-console */
