export const BaseEvent = {
    Open: 'wm:open',
    Close: 'wm:close',
    ThemeChange: 'theme:change',
    Ping: 'ping',
} as const;

export type BaseEventMap = {
    [BaseEvent.Open]: { id: string; type: string };
    [BaseEvent.Close]: { id: string };
    [BaseEvent.ThemeChange]: 'dark' | 'light';
    [BaseEvent.Ping]: void;
};
