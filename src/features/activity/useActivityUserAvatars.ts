import { useEffect, useRef, useState } from 'react';

import userProfileRepository from '@/repositories/userProfileRepository';
import { userImage } from '@/services/entityMediaService';
import { useFriendRosterStore } from '@/state/friendRosterStore';

import { resolveMissingEntities } from './resolveMissingEntities';

export function useActivityUserAvatars(
    userIds: string[]
): (userId: string) => string {
    const friendsById = useFriendRosterStore((state) => state.friendsById);
    const [fetched, setFetched] = useState<Map<string, string>>(new Map());
    const requestedRef = useRef(new Set<string>());
    const userIdsKey = userIds.join(',');

    useEffect(() => {
        const ids = (userIdsKey ? userIdsKey.split(',') : []).filter(
            (userId) => userId && !requestedRef.current.has(userId)
        );
        if (ids.length === 0) {
            return;
        }
        for (const userId of ids) {
            requestedRef.current.add(userId);
        }
        let active = true;

        void resolveMissingEntities({
            ids,
            isActive: () => active,
            fetchOne: async (userId) => {
                const profile = await userProfileRepository.getUserProfile({
                    userId
                });
                const image = userImage(profile, true, '128');
                return image || null;
            },
            onResolved: (userId, image) => {
                setFetched((previous) => new Map(previous).set(userId, image));
            }
        });

        return () => {
            active = false;
        };
    }, [userIdsKey]);

    return (userId: string) => {
        const friend = friendsById[userId];
        if (friend) {
            const image = userImage(friend, true, '128');
            if (image) {
                return image;
            }
        }
        return fetched.get(userId) ?? '';
    };
}
