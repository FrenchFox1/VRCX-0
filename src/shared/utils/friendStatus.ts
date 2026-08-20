import { normalizeString } from './string';

const FRIEND_STATUSES = [
    'join me',
    'active',
    'ask me',
    'busy',
    'offline'
] as const;

type FriendStatus = (typeof FRIEND_STATUSES)[number];

function isFriendStatus(value: string): value is FriendStatus {
    return FRIEND_STATUSES.some((status) => status === value);
}

function normalizeUserStatus(value: unknown): string {
    const status = normalizeString(value).toLowerCase();
    if (status === 'joinme') {
        return 'join me';
    }
    if (status === 'askme') {
        return 'ask me';
    }
    if (status === 'offline:offline' || status.startsWith('offline ')) {
        return 'offline';
    }
    return status;
}

function userStatusFromValue(value: unknown): FriendStatus | '' {
    const status = normalizeUserStatus(value);
    return isFriendStatus(status) ? status : '';
}

function sortStatus(
    a: FriendStatus | string,
    b: FriendStatus | string
): number {
    switch (b) {
        case 'join me':
            switch (a) {
                case 'active':
                case 'ask me':
                case 'busy':
                case 'offline':
                    return 1;
            }
            break;
        case 'active':
            switch (a) {
                case 'join me':
                    return -1;
                case 'ask me':
                case 'busy':
                case 'offline':
                    return 1;
            }
            break;
        case 'ask me':
            switch (a) {
                case 'join me':
                case 'active':
                    return -1;
                case 'busy':
                case 'offline':
                    return 1;
            }
            break;
        case 'busy':
            switch (a) {
                case 'join me':
                case 'active':
                case 'ask me':
                    return -1;
                case 'offline':
                    return 1;
            }
            break;
        case 'offline':
            switch (a) {
                case 'join me':
                case 'active':
                case 'ask me':
                case 'busy':
                    return -1;
            }
            break;
    }
    return 0;
}

export {
    FRIEND_STATUSES,
    isFriendStatus,
    normalizeUserStatus,
    sortStatus,
    userStatusFromValue
};
export type { FriendStatus };
