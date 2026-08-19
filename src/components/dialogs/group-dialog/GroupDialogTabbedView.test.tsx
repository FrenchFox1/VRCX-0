// @vitest-environment jsdom

import {
    cleanup,
    fireEvent,
    render,
    screen,
    waitFor
} from '@testing-library/react';
import type { ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { GroupProfileRecord } from '@/domain/entities/group';

import type {
    GroupDialogControls,
    GroupDialogResource,
    GroupDialogTabCommands,
    GroupDialogTabModel,
    GroupDialogView
} from './groupDialogTypes';

const mocks = vi.hoisted(() => ({
    getFollowingGroupCalendars: vi.fn(),
    getGroupCalendar: vi.fn()
}));

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key })
}));

vi.mock('react-router', () => ({
    useNavigate: () => vi.fn()
}));

vi.mock('@/repositories/vrchatToolsRepository', () => ({
    default: {
        followGroupEvent: vi.fn(),
        getFollowingGroupCalendars: mocks.getFollowingGroupCalendars,
        getGroupCalendar: mocks.getGroupCalendar
    }
}));

vi.mock('@/services/dialogService', () => ({
    openUserDialog: vi.fn()
}));

vi.mock('@/state/dialogStore', () => ({
    useDialogStore: <T,>(
        selector: (state: { closeDialog: () => void }) => T
    ): T => selector({ closeDialog: vi.fn() })
}));

vi.mock('../EntityDialogScaffold', () => ({
    EntityDialogScaffold: ({ children }: { children: ReactNode }) => (
        <div>{children}</div>
    ),
    EntityDialogTwoColumnLayout: ({
        children,
        rail
    }: {
        children: ReactNode;
        rail: ReactNode;
    }) => (
        <div>
            {rail}
            {children}
        </div>
    )
}));

vi.mock('./GroupDialogHeaderSection', () => ({
    GroupDialogHeaderSection: () => null
}));

vi.mock('./GroupDialogTabPanels', () => ({
    GroupDialogTabPanels: ({
        tabCommands,
        tabModel
    }: {
        tabCommands: GroupDialogTabCommands;
        tabModel: GroupDialogTabModel;
    }) => (
        <div>
            <span>{tabModel.activeTab}</span>
            <button
                type="button"
                onClick={() => tabCommands.onChangeTab('events')}
            >
                Open events
            </button>
        </div>
    )
}));

vi.mock('./GroupPostEditorDialog', () => ({
    GroupPostEditorDialog: () => null
}));

vi.mock('./useGroupDialogLanguageRows', () => ({
    useGroupDialogLanguageRows: () => []
}));

vi.mock('./useGroupDialogPosts', () => ({
    useGroupDialogPosts: () => ({
        createGroupPost: vi.fn(),
        deleteGroupPost: vi.fn(),
        editGroupPost: vi.fn(),
        postEditor: null,
        postEditorSubmitting: false,
        setPostEditor: vi.fn(),
        submitGroupPost: vi.fn()
    })
}));

vi.mock('./useGroupDialogTabbedRuntimeState', () => ({
    useGroupDialogTabbedRuntimeState: () => ({
        confirm: vi.fn(),
        currentEndpoint: 'https://api.example.test',
        currentUserId: 'usr_current',
        openImagePreview: vi.fn(),
        prompt: vi.fn()
    })
}));

import { GroupDialogTabbedView } from './GroupDialogTabbedView';

const group: GroupProfileRecord = {
    bannerUrl: '',
    description: '',
    discriminator: '',
    displayName: 'Test Group',
    iconUrl: '',
    id: 'grp_test',
    languages: [],
    links: [],
    memberCount: 1,
    membershipStatus: 'member',
    name: 'Test Group',
    onlineMemberCount: 0,
    ownerDisplayName: 'Owner',
    ownerId: 'usr_owner',
    privacy: 'default',
    roles: [],
    rules: '',
    shortCode: 'TEST',
    tags: [],
    url: ''
};

const groupResource: GroupDialogResource = {
    actionStatus: 'idle',
    detail: '',
    group
};

const groupView: GroupDialogView = {
    bannerUrl: '',
    canJoin: false,
    iconUrl: '',
    isBlocked: false,
    isMember: true,
    isRepresenting: false,
    isSubscribedToAnnouncements: false,
    joinState: 'joined',
    memberStatus: 'member',
    memberVisibility: 'visible',
    ownerDisplayName: 'Owner'
};

const groupControls: GroupDialogControls = {
    onBlock: vi.fn(),
    onCancelRequest: vi.fn(),
    onJoin: vi.fn(),
    onLeave: vi.fn(),
    onPreviousInstancesChange: vi.fn(),
    onRefresh: vi.fn(),
    onRepresent: vi.fn(),
    onSubscribe: vi.fn(),
    onVisibility: vi.fn()
};

describe('GroupDialogTabbedView calendar loading', () => {
    afterEach(cleanup);

    beforeEach(() => {
        vi.clearAllMocks();
        mocks.getGroupCalendar.mockResolvedValue({ results: [] });
        mocks.getFollowingGroupCalendars.mockResolvedValue({ results: [] });
    });

    it('loads following calendars only after opening the Events tab', async () => {
        render(
            <GroupDialogTabbedView
                groupControls={groupControls}
                groupResource={groupResource}
                groupView={groupView}
            />
        );

        await waitFor(() => {
            expect(mocks.getGroupCalendar).toHaveBeenCalledOnce();
        });
        expect(mocks.getFollowingGroupCalendars).not.toHaveBeenCalled();

        fireEvent.click(screen.getByRole('button', { name: 'Open events' }));

        await waitFor(() => {
            expect(mocks.getFollowingGroupCalendars).toHaveBeenCalledOnce();
        });
        expect(mocks.getGroupCalendar).toHaveBeenCalledOnce();
    });
});
