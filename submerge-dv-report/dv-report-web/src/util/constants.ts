export abstract class Constants {
    //static readonly API_URL = 'http://localhost:7900';
    static readonly API_URL = 'https://dv-report.api.submerge.io:13101';
    // connection
    static readonly CONNECTION_TIMEOUT_MS = 30000;
    static readonly CONNECTION_RETRY_MS = 5000;
    // UI
    static readonly HASH_TRIM_SIZE = 7;
    static readonly CONTENT_FADE_ANIM_DURATION_MS = 300;
    static readonly ARTIFICIAL_DELAY_MS = 1500;
    static readonly LOADING_STATE_TRANSITION_MIN_MS = 50;
    static readonly CHART_TRANSITION_TIME_MS = 750;
    static readonly NAY_COLOR = '#f44336';
    static readonly ABSTAIN_COLOR = '#aaaaaa';
    static readonly AYE_COLOR = '#4caf50';
    // format
    static readonly BALANCE_FORMAT_DECIMALS = 4;
    static readonly DECIMAL_SEPARATOR = '.';
    static readonly THOUSANDS_SEPARATOR = ',';
    static readonly MAX_IDENTITY_DISPLAY_LENGTH = 24;
    // networks
    static readonly POLKADOT_ID = 1;
    static readonly KUSAMA_ID = 2;
    static readonly BLOCK_TIME_MS = 6000;
}
