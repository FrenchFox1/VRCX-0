import { userImage } from '@/services/entityMediaService';
import { useFriendRosterStore } from '@/state/friendRosterStore';

export function useActivityUserAvatars(): (userId: string) => string {
    const friendsById = useFriendRosterStore((state) => state.friendsById);

    return (userId: string) => {
        const friend = friendsById[userId];
        return friend ? userImage(friend, true, '128') : '';
    };
}
