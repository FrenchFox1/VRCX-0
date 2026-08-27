// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { FeedRow } from '../feedTypes';

const mocks = vi.hoisted(() => ({
    getAvatarNameFromImageUrl: vi.fn(),
    openImagePreview: vi.fn()
}));

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key })
}));

vi.mock('sonner', () => ({
    toast: {
        error: vi.fn(),
        warning: vi.fn()
    }
}));

vi.mock('@/components/media/FadeInImage', () => ({
    FadeInImage: ({ alt }: { alt: string }) => <span>{alt}</span>
}));

vi.mock('@/repositories/avatarProfileRepository', () => ({
    default: {
        getAvatarNameFromImageUrl: mocks.getAvatarNameFromImageUrl
    }
}));

vi.mock('@/services/dialogService', () => ({
    openAvatarDialog: vi.fn(),
    openUserDialog: vi.fn()
}));

vi.mock('@/state/modalStore', () => ({
    useModalStore: <T,>(
        selector: (state: {
            openImagePreview: typeof mocks.openImagePreview;
        }) => T
    ): T => selector({ openImagePreview: mocks.openImagePreview })
}));

vi.mock('@/state/runtimeStore', () => ({
    useRuntimeStore: <T,>(
        selector: (state: {
            auth: {
                currentUserEndpoint: string;
                currentUserSnapshot: null;
            };
        }) => T
    ): T =>
        selector({
            auth: {
                currentUserEndpoint: 'https://api.example.test',
                currentUserSnapshot: null
            }
        })
}));

vi.mock('./FeedLocationLink', () => ({
    FeedLocationLink: () => null
}));

import { FeedDetailCell } from './FeedDetailCell';
import { FeedExpandedRow } from './FeedExpandedRow';

const avatarRow: FeedRow = {
    avatarName: '',
    currentAvatarImageUrl: 'https://api.example.test/file/file_avatar/1/file',
    ownerId: '',
    type: 'Avatar',
    userId: 'usr_feed'
};

describe('Feed avatar info loading', () => {
    afterEach(cleanup);

    beforeEach(() => {
        vi.clearAllMocks();
        mocks.getAvatarNameFromImageUrl.mockResolvedValue({
            avatarName: 'Resolved Avatar',
            ownerId: 'usr_owner'
        });
    });

    it('resolves avatar file metadata for the collapsed row', async () => {
        render(<FeedDetailCell row={avatarRow} />);

        await waitFor(() => {
            expect(mocks.getAvatarNameFromImageUrl).toHaveBeenCalledOnce();
            expect(screen.getByText('Resolved Avatar')).toBeTruthy();
        });
    });

    it('resolves avatar file metadata for the expanded row', async () => {
        render(
            <FeedExpandedRow
                loadingHistoryKey=""
                onNewInstance={vi.fn()}
                onOpenPreviousInstances={vi.fn()}
                row={avatarRow}
            />
        );

        await waitFor(() => {
            expect(mocks.getAvatarNameFromImageUrl).toHaveBeenCalledOnce();
            expect(screen.getByText('Resolved Avatar')).toBeTruthy();
        });
    });
});
