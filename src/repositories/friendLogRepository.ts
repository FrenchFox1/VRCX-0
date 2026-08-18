import {
    commands,
    type FriendLogCurrentOutput
} from '@/platform/tauri/bindings';

export interface FriendLogCurrentRow {
    userId: string;
    displayName: string;
    trustLevel: string;
    friendNumber: number;
}

export interface FriendLogCurrentEntry {
    userId?: string | null;
    displayName?: string | null;
    trustLevel?: string | null;
    friendNumber?: number | string | null;
}

type FriendLogSourceRow = FriendLogCurrentOutput;

function normalizeFriendLogRow(row: FriendLogSourceRow): FriendLogCurrentRow {
    return {
        userId: row.userId,
        displayName: row.displayName,
        trustLevel: row.trustLevel || 'Visitor',
        friendNumber: row.friendNumber
    };
}

async function getFriendLogCurrent(
    userId: unknown
): Promise<FriendLogCurrentRow[]> {
    const rows = await commands.appFriendLogCurrentList(
        typeof userId === 'string' ? userId.trim() : String(userId ?? '').trim()
    );

    return rows
        .map(normalizeFriendLogRow)
        .filter((row) => typeof row.userId === 'string' && row.userId.trim());
}

const friendLogRepository = {
    getFriendLogCurrent
};

export { getFriendLogCurrent };
export default friendLogRepository;
