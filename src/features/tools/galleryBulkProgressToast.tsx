import { toast } from 'sonner';

import { Progress } from '@/ui/shadcn/progress';

const GALLERY_BULK_PROGRESS_TOAST_ID = 'gallery-bulk-progress';

export type GalleryBulkProgressToast = {
    update(done: number): void;
    dismiss(): void;
};

export function startGalleryBulkProgressToast({
    total,
    buildMessage,
    cancelLabel,
    onCancel
}: {
    total: number;
    buildMessage(done: number): string;
    cancelLabel: string;
    onCancel(): void;
}): GalleryBulkProgressToast {
    let cancelled = false;

    function render(done: number) {
        if (cancelled) {
            return;
        }
        toast.loading(buildMessage(done), {
            id: GALLERY_BULK_PROGRESS_TOAST_ID,
            duration: Infinity,
            description: (
                <Progress
                    value={total > 0 ? Math.round((done / total) * 100) : 0}
                />
            ),
            cancel: {
                label: cancelLabel,
                onClick: () => {
                    cancelled = true;
                    onCancel();
                }
            }
        });
    }

    render(0);

    return {
        update: render,
        dismiss() {
            cancelled = true;
            toast.dismiss(GALLERY_BULK_PROGRESS_TOAST_ID);
        }
    };
}
