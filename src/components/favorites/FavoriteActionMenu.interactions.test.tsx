// @vitest-environment jsdom

import {
    cleanup,
    fireEvent,
    render,
    screen,
    waitFor
} from '@testing-library/react';
import { createContext, useContext } from 'react';
import type { PropsWithChildren, ReactNode } from 'react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { FavoriteKind } from '@/domain/favorites/types';

const mocks = vi.hoisted(() => ({
    addLocalFavorite: vi.fn(),
    confirm: vi.fn(),
    createLocalFavoriteGroup: vi.fn(),
    prompt: vi.fn()
}));

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key })
}));

vi.mock('@/components/favorites/useLocalWorldFavorites', () => ({
    useLocalWorldFavorites: () => ({
        favoritesByGroup: {},
        groupNames: [],
        reload: vi.fn(),
        status: 'ready'
    })
}));

vi.mock('@/repositories/favoritePersistenceRepository', () => ({
    default: {
        addLocalFavorite: mocks.addLocalFavorite,
        createLocalFavoriteGroup: mocks.createLocalFavoriteGroup,
        removeLocalFavorite: vi.fn()
    }
}));

vi.mock('@/repositories/vrchatFavoriteRepository', () => ({
    default: {
        addFavorite: vi.fn(),
        deleteFavorite: vi.fn()
    }
}));

vi.mock('@/services/favoriteAvatarCacheService', () => ({
    persistAvatarDetails: vi.fn()
}));

vi.mock('@/services/favoriteWorldCacheService', () => ({
    persistWorldDetails: vi.fn()
}));

vi.mock('@/state/favoriteStore', () => ({
    useFavoriteStore: <T,>(selector: (state: FavoriteStoreState) => T) =>
        selector({
            favoriteAvatarGroups: [],
            favoriteFriendGroups: [],
            favoriteWorldGroups: [],
            localAvatarFavoriteGroups: [],
            localAvatarFavorites: {},
            localFriendFavoriteGroups: [],
            localFriendFavorites: {},
            remoteFavoritesByObjectId: {}
        })
}));

vi.mock('@/state/modalStore', () => ({
    useModalStore: <T,>(selector: (state: ModalStoreState) => T) =>
        selector({ confirm: mocks.confirm, prompt: mocks.prompt })
}));

vi.mock('@/ui/shadcn/button', () => ({
    Button: ({
        children,
        size: _size,
        variant: _variant,
        ...props
    }: PropsWithChildren<{ size?: unknown; variant?: unknown }>) => (
        <button {...props}>{children}</button>
    )
}));

vi.mock('@/ui/shadcn/dropdown-menu', () => {
    const Container = ({ children }: PropsWithChildren) => (
        <div>{children}</div>
    );
    const GroupContext = createContext(false);
    const Group = ({ children }: PropsWithChildren) => (
        <GroupContext.Provider value>{children}</GroupContext.Provider>
    );
    const Label = ({ children }: PropsWithChildren) => {
        if (!useContext(GroupContext)) {
            throw new Error('MenuGroupContext is missing');
        }
        return <div>{children}</div>;
    };
    const Item = ({
        children,
        closeOnClick: _closeOnClick,
        variant: _variant,
        ...props
    }: PropsWithChildren<{ closeOnClick?: boolean; variant?: unknown }>) => (
        <button {...props}>{children}</button>
    );

    return {
        DropdownMenu: Container,
        DropdownMenuCheckboxItem: Item,
        DropdownMenuContent: Container,
        DropdownMenuGroup: Group,
        DropdownMenuItem: Item,
        DropdownMenuLabel: Label,
        DropdownMenuSeparator: () => <hr />,
        DropdownMenuTrigger: ({ render }: { render?: ReactNode }) => render
    };
});

vi.mock('@/ui/shadcn/spinner', () => ({
    Spinner: () => <span />
}));

import { FavoriteActionMenu } from './FavoriteActionMenu';

type FavoriteStoreState = {
    favoriteAvatarGroups: [];
    favoriteFriendGroups: [];
    favoriteWorldGroups: [];
    localAvatarFavoriteGroups: [];
    localAvatarFavorites: {};
    localFriendFavoriteGroups: [];
    localFriendFavorites: {};
    remoteFavoritesByObjectId: {};
};

type ModalStoreState = {
    confirm: typeof mocks.confirm;
    prompt: typeof mocks.prompt;
};

describe('FavoriteActionMenu local group creation', () => {
    beforeEach(() => {
        mocks.addLocalFavorite.mockReset().mockResolvedValue(1);
        mocks.confirm.mockReset();
        mocks.createLocalFavoriteGroup.mockReset().mockResolvedValue(undefined);
        mocks.prompt.mockReset().mockResolvedValue({
            ok: true,
            value: 'Frequently used'
        });
    });

    afterEach(() => {
        cleanup();
    });

    it.each<[FavoriteKind, string]>([
        ['friend', 'usr_friend'],
        ['world', 'wrld_world'],
        ['avatar', 'avtr_avatar']
    ])(
        'creates a local %s group and adds the current entity',
        async (kind, entityId) => {
            render(
                <FavoriteActionMenu
                    kind={kind}
                    entityId={entityId}
                    entity={{ id: entityId, name: entityId }}
                    iconOnly
                />
            );

            fireEvent.click(
                screen.getByRole('button', {
                    name: 'view.favorite.worlds.new_group'
                })
            );

            await waitFor(() =>
                expect(mocks.createLocalFavoriteGroup).toHaveBeenCalledWith({
                    kind,
                    groupName: 'Frequently used'
                })
            );
            expect(mocks.addLocalFavorite).toHaveBeenCalledWith({
                kind,
                entityId,
                groupName: 'Frequently used'
            });
        }
    );
});
