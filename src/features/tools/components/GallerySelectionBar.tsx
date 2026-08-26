import { StarIcon, StarOffIcon, Trash2Icon, XIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { Button } from '@/ui/shadcn/button';

type GallerySelectionBarProps = {
    selectedCount: number;
    deletableCount: number;
    isAllSelected: boolean;
    actionsDisabled: boolean;
    favoriteActions?: {
        onFavorite(): void;
        onUnfavorite(): void;
    };
    onSelectAll(): void;
    onClearSelection(): void;
    onDelete(): void;
};

export function GallerySelectionBar({
    selectedCount,
    deletableCount,
    isAllSelected,
    actionsDisabled,
    favoriteActions,
    onSelectAll,
    onClearSelection,
    onDelete
}: GallerySelectionBarProps) {
    const { t } = useTranslation();

    if (selectedCount === 0) {
        return null;
    }

    return (
        <div className="pointer-events-none absolute inset-x-0 bottom-3 z-20 flex justify-center px-2">
            <div className="bg-popover text-popover-foreground pointer-events-auto flex max-w-full flex-wrap items-center gap-1.5 rounded-full border px-3 py-1.5 text-sm shadow-lg">
                <span className="text-muted-foreground px-1.5 font-medium whitespace-nowrap">
                    {t('view.tools.gallery_selection.count', {
                        count: selectedCount
                    })}
                </span>
                <Button
                    type="button"
                    size="sm"
                    variant="ghost"
                    onClick={onSelectAll}
                >
                    {isAllSelected
                        ? t('view.tools.gallery_selection.deselect_all')
                        : t('view.tools.gallery_selection.select_all')}
                </Button>
                {favoriteActions ? (
                    <>
                        <Button
                            type="button"
                            size="sm"
                            variant="ghost"
                            disabled={actionsDisabled}
                            onClick={favoriteActions.onFavorite}
                        >
                            <StarIcon data-icon="inline-start" />
                            {t('view.tools.prints_favorites.favorite')}
                        </Button>
                        <Button
                            type="button"
                            size="sm"
                            variant="ghost"
                            disabled={actionsDisabled}
                            onClick={favoriteActions.onUnfavorite}
                        >
                            <StarOffIcon data-icon="inline-start" />
                            {t('view.tools.prints_favorites.unfavorite')}
                        </Button>
                    </>
                ) : null}
                <Button
                    type="button"
                    size="sm"
                    variant="ghost"
                    disabled={actionsDisabled || deletableCount === 0}
                    title={
                        deletableCount === 0
                            ? t('view.tools.gallery_selection.delete_locked')
                            : undefined
                    }
                    onClick={onDelete}
                >
                    <Trash2Icon data-icon="inline-start" />
                    {deletableCount < selectedCount
                        ? t('view.tools.gallery_selection.delete_unlocked', {
                              count: deletableCount
                          })
                        : t('common.actions.delete')}
                </Button>
                <Button
                    type="button"
                    size="icon-xs"
                    variant="ghost"
                    className="rounded-full"
                    aria-label={t('common.actions.clear')}
                    onClick={onClearSelection}
                >
                    <XIcon data-icon="icon" />
                </Button>
            </div>
        </div>
    );
}
