import { beforeEach, describe, expect, it, vi } from 'vitest';

import avatarProfileRepository from '@/repositories/avatarProfileRepository';

import {
    filterRemoteEntityCacheFallbacksById,
    loadRemoteEntityCacheFallbacksById
} from './remoteEntityCacheFallbacks';
import { getAvatarDetailFallbackIds } from './useAvatarDetailFallbacks';

const fetchAvatarById = (avatarId: string) =>
    avatarProfileRepository.getAvatarProfile({ avatarId });

vi.mock('@/repositories/avatarProfileRepository', () => ({
    default: {
        getAvatarProfile: vi.fn()
    }
}));

function cachedAvatar(id: string, name: string, releaseStatus = 'private') {
    return {
        id,
        authorId: 'usr_author',
        authorName: 'Cache Author',
        created_at: '2026-06-01T00:00:00.000Z',
        description: 'Cached description',
        imageUrl: 'https://example.test/image.png',
        name,
        releaseStatus,
        thumbnailImageUrl: 'https://example.test/thumb.png',
        updated_at: '2026-06-02T00:00:00.000Z',
        version: 1,
        tags: [],
        unityPackages: [],
        $isCached: true,
        $memo: '',
        $tags: [],
        $timeSpent: 0
    };
}

function emptyAvatar(id: string) {
    return {
        id,
        authorId: '',
        authorName: '',
        created_at: '',
        description: '',
        imageUrl: '',
        name: '',
        releaseStatus: '',
        thumbnailImageUrl: '',
        updated_at: '',
        version: 0,
        tags: [],
        unityPackages: [],
        $isCached: true,
        $memo: '',
        $tags: [],
        $timeSpent: 0
    };
}

describe('useAvatarDetailFallbacks helpers', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('asks Rust for every favorite avatar with no remote detail', async () => {
        const fallbackIds = getAvatarDetailFallbackIds({
            avatarIds: ['avtr_remote', 'avtr_local', 'avtr_missing'],
            kind: 'avatar',
            remoteEntityDetailsData: {
                avtr_remote: { name: 'Remote Avatar' }
            },
            remoteEntityDetailsStatus: 'ready'
        });

        vi.mocked(avatarProfileRepository.getAvatarProfile).mockResolvedValue(
            cachedAvatar('avtr_missing', 'DB Missing Avatar')
        );

        const fallbacks = await loadRemoteEntityCacheFallbacksById(
            fallbackIds,
            fetchAvatarById
        );

        expect(fallbackIds).toEqual(['avtr_local', 'avtr_missing']);
        expect(avatarProfileRepository.getAvatarProfile).toHaveBeenCalledTimes(
            2
        );
        expect(avatarProfileRepository.getAvatarProfile).toHaveBeenCalledWith({
            avatarId: 'avtr_missing'
        });
        expect(fallbacks).toMatchObject({
            avtr_missing: {
                name: 'DB Missing Avatar',
                releaseStatus: 'private'
            }
        });
    });

    it('ignores empty cache shells returned from avatar_cache', async () => {
        vi.mocked(avatarProfileRepository.getAvatarProfile).mockImplementation(
            async ({ avatarId }) => {
                if (avatarId === 'avtr_cached') {
                    return cachedAvatar(
                        'avtr_cached',
                        'Cached Avatar',
                        'public'
                    );
                }
                if (avatarId === 'avtr_shell') {
                    return emptyAvatar('avtr_shell');
                }
                throw new Error(`Missing avatar: ${String(avatarId)}`);
            }
        );

        const fallbacks = await loadRemoteEntityCacheFallbacksById(
            ['avtr_cached', 'avtr_shell', 'avtr_missing'],
            fetchAvatarById
        );

        expect(fallbacks).toMatchObject({
            avtr_cached: {
                name: 'Cached Avatar',
                releaseStatus: 'public'
            }
        });
        expect(fallbacks).not.toHaveProperty('avtr_shell');
        expect(fallbacks).not.toHaveProperty('avtr_missing');
    });

    it('filters stale fallback rows when the current favorite ids change', () => {
        const fallbacks = filterRemoteEntityCacheFallbacksById(
            {
                avtr_old: cachedAvatar('avtr_old', 'Old Avatar'),
                avtr_new: cachedAvatar('avtr_new', 'New Avatar')
            },
            ['avtr_new']
        );

        expect(fallbacks).toMatchObject({
            avtr_new: {
                name: 'New Avatar'
            }
        });
        expect(fallbacks).not.toHaveProperty('avtr_old');
    });

    it('does not search avatar_cache before remote avatar details are ready', () => {
        expect(
            getAvatarDetailFallbackIds({
                avatarIds: ['avtr_pending'],
                kind: 'avatar',
                remoteEntityDetailsStatus: 'running'
            })
        ).toEqual([]);
    });
});
