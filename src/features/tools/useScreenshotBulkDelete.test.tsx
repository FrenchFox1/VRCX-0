// @vitest-environment jsdom

import { act, cleanup, render } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    confirm: vi.fn(),
    deleteScreenshotFile: vi.fn(),
    dismiss: vi.fn(),
    toastError: vi.fn(),
    toastSuccess: vi.fn(),
    toastWarning: vi.fn(),
    update: vi.fn()
}));

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key })
}));

vi.mock('sonner', () => ({
    toast: {
        error: mocks.toastError,
        success: mocks.toastSuccess,
        warning: mocks.toastWarning
    }
}));

vi.mock('@/repositories/mediaRepository', () => ({
    default: { deleteScreenshotFile: mocks.deleteScreenshotFile }
}));

vi.mock('@/state/modalStore', () => ({
    useModalStore: (selector: (state: { confirm: unknown }) => unknown) =>
        selector({ confirm: mocks.confirm })
}));

vi.mock('./galleryBulkProgressToast', () => ({
    startGalleryBulkProgressToast: () => ({
        update: mocks.update,
        dismiss: mocks.dismiss
    })
}));

import { useScreenshotBulkDelete } from './useScreenshotBulkDelete';

type HookValue = ReturnType<typeof useScreenshotBulkDelete>;

function renderHarness() {
    const removeGalleryImages = vi.fn();
    const refreshGalleryTree = vi.fn();
    let value: HookValue | null = null;

    function Harness() {
        value = useScreenshotBulkDelete({
            selectedFolder: 'C:\\VRChat\\2026-07',
            removeGalleryImages,
            refreshGalleryTree
        });
        return null;
    }

    render(<Harness />);
    return {
        refreshGalleryTree,
        removeGalleryImages,
        deleteScreenshots: (paths: string[]) =>
            act(async () => {
                await value!.deleteScreenshots(paths);
            })
    };
}

describe('useScreenshotBulkDelete', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mocks.confirm.mockResolvedValue({ ok: true });
        mocks.deleteScreenshotFile.mockResolvedValue(null);
    });

    afterEach(cleanup);

    it('removes every deleted path in one update and refreshes the folder tree', async () => {
        const harness = renderHarness();

        await harness.deleteScreenshots(['a.png', 'b.png']);

        expect(mocks.deleteScreenshotFile).toHaveBeenCalledTimes(2);
        expect(harness.removeGalleryImages).toHaveBeenCalledTimes(1);
        expect(harness.removeGalleryImages).toHaveBeenCalledWith([
            'a.png',
            'b.png'
        ]);
        expect(harness.refreshGalleryTree).toHaveBeenCalledTimes(1);
        expect(mocks.toastSuccess).toHaveBeenCalledTimes(1);
    });

    it('keeps failed paths in the gallery and reports a partial failure', async () => {
        mocks.deleteScreenshotFile.mockImplementation((path: string) =>
            path === 'b.png'
                ? Promise.reject(new Error('locked'))
                : Promise.resolve(null)
        );
        const harness = renderHarness();

        await harness.deleteScreenshots(['a.png', 'b.png', 'c.png']);

        expect(harness.removeGalleryImages).toHaveBeenCalledWith([
            'a.png',
            'c.png'
        ]);
        expect(mocks.toastError).toHaveBeenCalledTimes(1);
        expect(mocks.toastSuccess).not.toHaveBeenCalled();
    });

    it('does nothing when the confirmation is dismissed', async () => {
        mocks.confirm.mockResolvedValue({ ok: false });
        const harness = renderHarness();

        await harness.deleteScreenshots(['a.png']);

        expect(mocks.deleteScreenshotFile).not.toHaveBeenCalled();
        expect(harness.removeGalleryImages).not.toHaveBeenCalled();
        expect(harness.refreshGalleryTree).not.toHaveBeenCalled();
    });
});
