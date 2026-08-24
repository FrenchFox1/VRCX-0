import React, { type ComponentProps } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it, vi } from 'vitest';

vi.mock('./FriendsSidebarFriendRow', () => ({
    FriendRow: ({
        appearance,
        rowModel
    }: {
        appearance: { currentLocationStartedAt?: string | number | null };
        rowModel: { canRequestInvite?: boolean };
    }) => (
        <button
            disabled={!rowModel.canRequestInvite}
            data-current-location-started-at={String(
                appearance.currentLocationStartedAt ?? ''
            )}
        >
            Request invite
        </button>
    )
}));

import { FriendsSidebarVirtualRow } from './FriendsSidebarVirtualRows';

type VirtualRowProps = ComponentProps<typeof FriendsSidebarVirtualRow>;

function renderFriendRow({
    currentLocationStartedAt = null,
    isCurrentUser = false,
    state = 'offline'
}: {
    currentLocationStartedAt?: string | number | null;
    isCurrentUser?: boolean;
    state?: string;
}) {
    const props: VirtualRowProps = {
        appearance: {},
        friendCommands: {
            onOpenFriend: vi.fn(),
            onToggleSection: vi.fn()
        },
        location: { locationMetadataByKey: new Map() },
        row: {
            type: 'friend',
            key: 'friend:test',
            friend: { id: 'usr_friend', state },
            isCurrentUser
        },
        runtime: {
            currentUser: null,
            currentUserId: 'usr_current',
            gameState: { isGameRunning: false, currentLocationStartedAt },
            onlineIdSet: new Set(),
            instanceActionGatesByUserId: new Map([
                [
                    'usr_friend',
                    {
                        key: 'usr_friend',
                        canJoin: false,
                        canOpenInGame: false,
                        canSelfInvite: false,
                        canRequestInvite: false,
                        canInvite: false
                    }
                ]
            ])
        },
        statusCommands: {}
    };

    return renderToStaticMarkup(<FriendsSidebarVirtualRow {...props} />);
}

describe('FriendsSidebarVirtualRow request invite action', () => {
    it.each(['online', 'offline'])(
        'keeps request invite enabled for a %s friend regardless of instance gates',
        (state) => {
            expect(renderFriendRow({ state })).not.toContain('disabled=""');
        }
    );

    it('keeps request invite unavailable for the current user', () => {
        expect(renderFriendRow({ isCurrentUser: true })).toContain(
            'disabled=""'
        );
    });

    it('passes the local room start time through for the current-user row', () => {
        expect(
            renderFriendRow({
                isCurrentUser: true,
                currentLocationStartedAt: 1_700_000_000_000
            })
        ).toContain('data-current-location-started-at="1700000000000"');
    });
});
