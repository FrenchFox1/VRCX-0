import {
    compareByLastActive,
    compareByLastSeen,
    compareByLocation,
    compareByLocationAt,
    compareByName,
    compareByPrivate,
    compareByStatus,
    type ComparableRecord
} from './compare';
import { sortStatus } from './friendStatus';

type FriendSortMethod =
    | 'Sort Alphabetically'
    | 'Sort Private to Bottom'
    | 'Sort by Status'
    | 'Sort by Last Active'
    | 'Sort by Last Seen'
    | 'Sort by Time in Instance'
    | 'Sort by Location'
    | 'None';

type FriendSortItem = ComparableRecord & {
    pendingOffline?: unknown;
};
type FriendComparator = (a: FriendSortItem, b: FriendSortItem) => number;

function getFriendsSortFunction(
    sortMethods: FriendSortMethod[]
): FriendComparator {
    const sorts: FriendComparator[] = [];
    for (const sortMethod of sortMethods) {
        switch (sortMethod) {
            case 'Sort Alphabetically':
                sorts.push(compareByName);
                break;
            case 'Sort Private to Bottom':
                sorts.push(compareByPrivate);
                break;
            case 'Sort by Status':
                sorts.push(compareByStatus);
                break;
            case 'Sort by Last Active':
                sorts.push(compareByLastActive);
                break;
            case 'Sort by Last Seen':
                sorts.push(compareByLastSeen);
                break;
            case 'Sort by Time in Instance':
                sorts.push((a: FriendSortItem, b: FriendSortItem) => {
                    if (
                        typeof a.ref === 'undefined' ||
                        typeof b.ref === 'undefined'
                    ) {
                        return 0;
                    }
                    if (a.pendingOffline && !b.pendingOffline) {
                        return 1;
                    }
                    if (a.pendingOffline && b.pendingOffline) {
                        return 0;
                    }
                    if (!a.pendingOffline && b.pendingOffline) {
                        return -1;
                    }
                    if (a.state !== 'online' || b.state !== 'online') {
                        return 0;
                    }

                    return compareByLocationAt(b.ref, a.ref);
                });
                break;
            case 'Sort by Location':
                sorts.push(compareByLocation);
                break;
            case 'None':
                sorts.push(() => 0);
                break;
        }
    }

    return (a: FriendSortItem, b: FriendSortItem) => {
        let res = 0;
        for (const sort of sorts) {
            res = sort(a, b);
            if (res !== 0) {
                return res;
            }
        }
        return res;
    };
}

export { getFriendsSortFunction, sortStatus };
export type { FriendSortItem, FriendSortMethod };
