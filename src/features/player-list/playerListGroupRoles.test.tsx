// @vitest-environment jsdom
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, cleanup, renderHook, waitFor } from '@testing-library/react';
import type { ReactNode } from 'react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { useAppTable } from '@/components/data-table/appTable';
import type { GroupProfileRecord } from '@/domain/entities/group';
import { commands } from '@/platform/tauri/bindings';
import groupProfileRepository from '@/repositories/groupProfileRepository';
import { useRuntimeStore } from '@/state/runtimeStore';

import { usePlayerListColumns } from './components/PlayerListColumns';
import { playerGroupRoles, playerGroupRoster } from './playerListGroupRoles';
import type { PlayerListRow } from './playerListTypes';
import { usePlayerListGroupRoles } from './usePlayerListGroupRoles';

vi.mock('@/platform/tauri/bindings', () => ({
    commands: { appVrchatGroupMemberGet: vi.fn() }
}));
vi.mock('@/repositories/groupProfileRepository', () => ({
    default: { getGroupProfile: vi.fn() }
}));
vi.mock('react-i18next', async (importOriginal) => ({
    ...(await importOriginal<typeof import('react-i18next')>()),
    useTranslation: () => ({ t: (key: string) => key })
}));

const roles = [
    { id: 'member', name: 'Member', order: 5 },
    { id: 'staff', name: 'Staff', order: 1 },
    { id: 'unknown', name: 'Unknown' }
];
const group = { ownerId: '', roles } as unknown as GroupProfileRecord;
const roster = playerGroupRoster(group);

function player(userId: string): PlayerListRow {
    return {
        userId,
        displayName: userId,
        userRef: null,
        trustLevel: '',
        trustSortNum: 0,
        trustClass: '',
        platformLabel: '',
        platformIcon: null,
        platformClassName: '',
        inVRMode: null,
        status: '',
        statusDescription: '',
        languages: [],
        bioLinks: [],
        note: '',
        avatarUrl: '',
        isCurrentUser: false,
        isFriend: false,
        isFavorite: false,
        isBlocked: false,
        isMuted: false,
        isAvatarInteractionDisabled: false,
        isChatBoxMuted: false,
        timeoutTime: 0,
        moderationSeverity: '',
        ageVerified: false,
        timerMs: 0,
        worldName: '',
        location: ''
    };
}

function member(userId: string, roleIds: string[]) {
    return {
        id: `gmem_${userId}`,
        groupId: 'grp_a',
        userId,
        roleIds,
        mRoleIds: roleIds,
        joinedAt: '2026-01-01',
        user: {
            id: userId,
            displayName: userId,
            thumbnailUrl: 'https://x/y.png'
        }
    };
}

function response(value: unknown) {
    return { status: 200, data: JSON.stringify(value) };
}
const rateLimited = {
    status: 429,
    data: '{"error":{"message":"Too many requests","status_code":429}}'
};

function renderRoles(initialRows: PlayerListRow[]) {
    useRuntimeStore.setState((state) => ({
        auth: { ...state.auth, currentUserId: 'owner', currentUserEndpoint: '' }
    }));
    vi.mocked(groupProfileRepository.getGroupProfile).mockResolvedValue(group);
    const client = new QueryClient({
        defaultOptions: { queries: { retry: false } }
    });
    const wrapper = ({ children }: { children: ReactNode }) => (
        <QueryClientProvider client={client}>{children}</QueryClientProvider>
    );
    const rendered = renderHook(
        ({ rows }) =>
            usePlayerListGroupRoles('wrld_test:1~group(grp_a)', rows, true),
        {
            initialProps: { rows: initialRows },
            wrapper
        }
    );
    return { ...rendered, client };
}

function roleNames(rows: readonly PlayerListRow[]) {
    return rows.map((row) => row.groupRoles?.map((role) => role.name) ?? null);
}

afterEach(() => {
    cleanup();
    vi.clearAllMocks();
});

describe('player list group roles', () => {
    it('matches role IDs and uses the group order, telling regular members from unknown ones', () => {
        expect(
            playerGroupRoles(
                roster,
                ['member', 'staff', 'missing'],
                'user'
            )?.map((role) => role.name)
        ).toEqual(['Staff', 'Member']);
        expect(playerGroupRoles(roster, [], 'user')).toEqual([]);
        expect(playerGroupRoles(roster, undefined, 'user')).toBeNull();
        expect(playerGroupRoles(roster, null, 'user')).toBeNull();
    });

    it.each([false, true])(
        'sorts by the highest role and keeps empty values last (descending %s)',
        (desc) => {
            const rows = ['member', 'staff', 'regular', 'unknown'].map(
                (id) => ({
                    ...player(id),
                    groupRoles:
                        id === 'unknown'
                            ? null
                            : playerGroupRoles(
                                  roster,
                                  id === 'regular' ? [] : [id],
                                  id
                              )
                })
            );
            const { result } = renderHook(() =>
                useAppTable({
                    data: rows,
                    columns: usePlayerListColumns(),
                    state: { sorting: [{ id: 'groupRoles', desc }] },
                    getRowId: (row) => row.userId
                })
            );
            expect(
                result.current
                    .getRowModel()
                    .rows.map((row) => row.original.userId)
            ).toEqual(
                desc
                    ? ['regular', 'member', 'staff', 'unknown']
                    : ['staff', 'member', 'regular', 'unknown']
            );
        }
    );

    it('queries each player once, caches only role IDs, blanks failed lookups and refetches on refresh', async () => {
        vi.mocked(commands.appVrchatGroupMemberGet).mockImplementation(
            async ({ userId }) => {
                if (userId === 'b') throw new Error('Forbidden');
                return response(
                    member(userId ?? '', [userId === 'a' ? 'staff' : 'member'])
                );
            }
        );
        const { result, rerender, client } = renderRoles([
            player('a'),
            player('b')
        ]);
        await waitFor(() =>
            expect(roleNames(result.current.rows)).toEqual([['Staff'], null])
        );
        expect(commands.appVrchatGroupMemberGet).toHaveBeenCalledTimes(2);
        expect(
            client.getQueryData(['player-list-group', 'owner', '', 'grp_a'])
        ).toEqual(roster);
        expect(
            client.getQueryData([
                'player-list-group',
                'owner',
                '',
                'grp_a',
                'a'
            ])
        ).toEqual(['staff']);
        expect(
            client.getQueryData([
                'player-list-group',
                'owner',
                '',
                'grp_a',
                'b'
            ])
        ).toBeNull();
        rerender({ rows: [player('a'), player('b'), player('c')] });
        await waitFor(() =>
            expect(roleNames(result.current.rows)[2]).toEqual(['Member'])
        );
        expect(commands.appVrchatGroupMemberGet).toHaveBeenCalledTimes(3);
        expect(commands.appVrchatGroupMemberGet).toHaveBeenLastCalledWith({
            groupId: 'grp_a',
            userId: 'c'
        });
        rerender({ rows: [player('c')] });
        rerender({ rows: [player('a'), player('c')] });
        await waitFor(() =>
            expect(roleNames(result.current.rows)[0]).toEqual(['Staff'])
        );
        expect(commands.appVrchatGroupMemberGet).toHaveBeenCalledTimes(3);
        act(() => result.current.refresh());
        await waitFor(() =>
            expect(commands.appVrchatGroupMemberGet).toHaveBeenCalledTimes(5)
        );
        act(() => result.current.selectGroup('grp_b'));
        expect(result.current.groupId).toBe('grp_b');
        expect(roleNames(result.current.rows)).toEqual([null, null]);
        await waitFor(() =>
            expect(commands.appVrchatGroupMemberGet).toHaveBeenCalledWith({
                groupId: 'grp_b',
                userId: 'a'
            })
        );
        client.clear();
    });

    it('keeps at most three member lookups in flight', async () => {
        const pending = new Map<
            string,
            (value: ReturnType<typeof response>) => void
        >();
        vi.mocked(commands.appVrchatGroupMemberGet).mockImplementation(
            ({ userId }) =>
                new Promise((resolve) => {
                    pending.set(userId ?? '', resolve);
                })
        );
        const { result, client } = renderRoles(
            ['a', 'b', 'c', 'd', 'e'].map(player)
        );
        await waitFor(() =>
            expect(commands.appVrchatGroupMemberGet).toHaveBeenCalledTimes(3)
        );
        await act(async () => {
            pending.get('a')?.(response(member('a', ['staff'])));
        });
        await waitFor(() =>
            expect(commands.appVrchatGroupMemberGet).toHaveBeenCalledTimes(4)
        );
        await waitFor(() =>
            expect(roleNames(result.current.rows)[0]).toEqual(['Staff'])
        );
        await act(async () => {
            for (const id of ['b', 'c', 'd'])
                pending.get(id)?.(response(member(id, ['member'])));
        });
        await waitFor(() =>
            expect(commands.appVrchatGroupMemberGet).toHaveBeenCalledTimes(5)
        );
        client.clear();
    });

    it('retries rate-limited lookups with backoff', async () => {
        vi.mocked(commands.appVrchatGroupMemberGet)
            .mockResolvedValueOnce(rateLimited)
            .mockResolvedValueOnce(response(member('a', ['staff'])));
        const { result, client } = renderRoles([player('a')]);
        await waitFor(
            () => expect(roleNames(result.current.rows)).toEqual([['Staff']]),
            { timeout: 4000 }
        );
        expect(commands.appVrchatGroupMemberGet).toHaveBeenCalledTimes(2);
        client.clear();
    }, 10000);
});
