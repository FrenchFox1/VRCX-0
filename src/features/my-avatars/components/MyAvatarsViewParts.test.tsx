// @vitest-environment jsdom

import {
    cleanup,
    fireEvent,
    render,
    screen,
    within
} from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key })
}));

vi.mock('@/services/dialogService', () => ({
    openAvatarDialog: vi.fn()
}));

import { getMyAvatarsGridDensityConfig } from '../myAvatarsGrid';
import type { MyAvatarActionHandler, MyAvatarRow } from '../myAvatarsTypes';
import { AvatarActionMenuItems, MyAvatarGridCard } from './MyAvatarGridCard';
import { PlatformBadges } from './MyAvatarsViewParts';

describe('My Avatars view parts', () => {
    afterEach(cleanup);

    it('labels icon-only platform badges', () => {
        render(
            <PlatformBadges
                unityPackages={[
                    { platform: 'standalonewindows', variant: 'standard' },
                    { platform: 'android', variant: 'standard' },
                    { platform: 'ios', variant: 'standard' }
                ]}
            />
        );

        expect(screen.getByLabelText('PC')).toBeTruthy();
        expect(screen.getByLabelText('Android')).toBeTruthy();
        expect(screen.getByLabelText('iOS')).toBeTruthy();
    });

    it('identifies the target avatar without changing menu actions', () => {
        const avatar: MyAvatarRow = {
            id: 'avtr_example',
            name: 'Example Avatar',
            releaseStatus: 'private'
        };
        const onAction = vi.fn<MyAvatarActionHandler>();

        render(
            <AvatarActionMenuItems
                avatar={avatar}
                isActive={false}
                disabled={false}
                Item="button"
                Group="div"
                Label="div"
                Separator="hr"
                onAction={onAction}
            />
        );

        expect(screen.getByText('Example Avatar')).toBeTruthy();

        fireEvent.click(screen.getByText('dialog.avatar.actions.make_public'));

        expect(onAction).toHaveBeenCalledWith('makePublic', avatar);
    });

    it('opens the grid card action menu', () => {
        render(
            <MyAvatarGridCard
                avatar={{
                    id: 'avtr_example',
                    name: 'Example Avatar',
                    releaseStatus: 'private'
                }}
                densityConfig={getMyAvatarsGridDensityConfig('standard')}
                isUpdating={false}
                onAction={vi.fn<MyAvatarActionHandler>()}
            />
        );

        fireEvent.click(
            screen.getByRole('button', {
                name: 'view.my_avatars.action.open_avatar_actions'
            })
        );

        const menu = screen.getByRole('menu');
        expect(menu).toBeTruthy();
        expect(
            within(menu).getByText('common.actions.view_details')
        ).toBeTruthy();
    });

    it('keeps a grid card without a thumbnail stretched to the full column width', () => {
        const { container } = render(
            <MyAvatarGridCard
                avatar={{
                    id: 'avtr_without_thumbnail',
                    name: 'Avatar Without Thumbnail',
                    releaseStatus: 'private'
                }}
                densityConfig={getMyAvatarsGridDensityConfig('standard')}
                isUpdating={false}
                onAction={vi.fn<MyAvatarActionHandler>()}
            />
        );

        const cardButton = container.querySelector('.group\\/card > button');

        expect(cardButton).toBeTruthy();
        expect(cardButton?.classList.contains('w-full')).toBe(true);
    });

    it('stretches a thumbnail across the full grid card', () => {
        const { container } = render(
            <MyAvatarGridCard
                avatar={{
                    id: 'avtr_with_thumbnail',
                    name: 'Avatar With Thumbnail',
                    releaseStatus: 'private',
                    thumbnailImageUrl: 'https://example.test/thumbnail.png'
                }}
                densityConfig={getMyAvatarsGridDensityConfig('standard')}
                isUpdating={false}
                onAction={vi.fn<MyAvatarActionHandler>()}
            />
        );

        const cardButton = container.querySelector('.group\\/card > button');
        const thumbnail = screen.getByRole('img', {
            name: 'Avatar With Thumbnail'
        });

        expect(cardButton?.classList.contains('w-full')).toBe(true);
        expect(thumbnail.classList.contains('w-full')).toBe(true);
        expect(thumbnail.classList.contains('object-cover')).toBe(true);
    });
});
