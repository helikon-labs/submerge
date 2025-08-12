import mitt from 'mitt';
import { BaseEvent, type BaseEventMap } from './base';
import { UIEvent, type UIEventMap } from './ui';

export const AppEvent = {
    ...BaseEvent,
    UI: UIEvent,
} as const;

export type EventMap = BaseEventMap & UIEventMap;
export const eventBus = mitt<EventMap>();
