export const UIEvent = {
    Layout: {
        Resize: 'ui:layout:resize',
        Collapse: 'ui:layout:collapse',
        Expand: 'ui:layout:expand',
    },
    Theme: {
        Change: 'ui:theme:change',
        Toggle: 'ui:theme:toggle',
    },
    Modal: {
        Open: 'ui:modal:open',
        Close: 'ui:modal:close',
    },
} as const;

export type UIEventMap = {
    [UIEvent.Layout.Resize]: { width: number; height: number };
    [UIEvent.Layout.Collapse]: { panel: string };
    [UIEvent.Theme.Change]: 'dark' | 'light';
    [UIEvent.Theme.Toggle]: void;
    [UIEvent.Modal.Open]: { id: string; type: string };
    [UIEvent.Modal.Close]: { id: string };
};
