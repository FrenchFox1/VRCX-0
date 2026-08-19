import { describe, expect, it, vi } from 'vitest';

import { createInstanceUserRow } from '@/domain/instances/instanceRoster';

import { enrichLocationUsersWithProfiles } from './useUserDialogLocationPanel';

describe('enrichLocationUsersWithProfiles', () => {
    it('uses the friend roster image without loading the remote profile', async () => {
        const loadUserProfile = vi.fn();
        const users = [
            createInstanceUserRow({
                id: 'usr_friend',
                displayName: 'Friend'
            })
        ];

        const enriched = await enrichLocationUsersWithProfiles({
            friendsById: {
                usr_friend: {
                    id: 'usr_friend',
                    displayName: 'Friend',
                    currentAvatarThumbnailImageUrl:
                        'https://example.test/friend.webp'
                }
            },
            knownUsersById: new Map(),
            loadUserProfile,
            users
        });

        expect(loadUserProfile).not.toHaveBeenCalled();
        expect(enriched[0].currentAvatarThumbnailImageUrl).toBe(
            'https://example.test/friend.webp'
        );
    });

    it('loads a missing image for a nonfriend', async () => {
        const loadUserProfile = vi.fn().mockResolvedValue({
            id: 'usr_nonfriend',
            displayName: 'Nonfriend',
            currentAvatarThumbnailImageUrl:
                'https://example.test/nonfriend.webp'
        });
        const users = [
            createInstanceUserRow({
                id: 'usr_nonfriend',
                displayName: 'Nonfriend'
            })
        ];

        const enriched = await enrichLocationUsersWithProfiles({
            friendsById: {},
            knownUsersById: new Map(),
            loadUserProfile,
            users
        });

        expect(loadUserProfile).toHaveBeenCalledWith({
            userId: 'usr_nonfriend'
        });
        expect(enriched[0].currentAvatarThumbnailImageUrl).toBe(
            'https://example.test/nonfriend.webp'
        );
    });
});
