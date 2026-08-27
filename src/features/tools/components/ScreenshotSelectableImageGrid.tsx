import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';

import type { ScreenshotLibraryImage } from '@/platform/tauri/bindings';

import { useScreenshotGalleryGrid } from '../useScreenshotGalleryGrid';
import {
    ScreenshotThumbnailCard,
    useScreenshotThumbnailTitleMap
} from './ScreenshotThumbnailGrid';

export function ScreenshotSelectableImageGrid({
    images,
    initialScrollTop,
    resetKey,
    hasSelection,
    selectedKeysSet,
    onOpen,
    onToggleSelect,
    onScrollPositionChange
}: {
    images: ScreenshotLibraryImage[];
    initialScrollTop: number;
    resetKey: string;
    hasSelection: boolean;
    selectedKeysSet: ReadonlySet<string>;
    onOpen: (path: string) => void;
    onToggleSelect: (path: string, checked: boolean, shift: boolean) => void;
    onScrollPositionChange?: (scrollTop: number) => void;
}) {
    const { t } = useTranslation();
    const {
        gridColumnCount,
        gridGap,
        gridMinWidth,
        totalHeight,
        viewportRef,
        visibleRows
    } = useScreenshotGalleryGrid({
        initialScrollTop,
        items: images,
        resetKey
    });
    const visibleItems = useMemo(
        () => visibleRows.flatMap((row) => row.items),
        [visibleRows]
    );
    const titleMap = useScreenshotThumbnailTitleMap(visibleItems);

    return (
        <div
            ref={viewportRef}
            className="min-h-0 flex-1 overflow-auto p-0.5 pr-1"
            onScroll={(event) =>
                onScrollPositionChange?.(event.currentTarget.scrollTop)
            }
        >
            <div className="relative" style={{ height: totalHeight }}>
                {visibleRows.map((row) => (
                    <div
                        key={row.key}
                        className="absolute right-0 left-0 grid"
                        style={{
                            top: row.top,
                            gridTemplateColumns: `repeat(${gridColumnCount}, minmax(${gridMinWidth}px, 1fr))`,
                            gap: gridGap
                        }}
                    >
                        {row.items.map((item: ScreenshotLibraryImage) => (
                            <ScreenshotThumbnailCard
                                key={item.path}
                                item={item}
                                onOpen={onOpen}
                                title={titleMap.get(item.path)}
                                selectable
                                selected={selectedKeysSet.has(item.path)}
                                selectionActive={hasSelection}
                                selectLabel={`${t('common.actions.select')} ${item.fileName}`}
                                onToggleSelect={(checked, shift) =>
                                    onToggleSelect(item.path, checked, shift)
                                }
                            />
                        ))}
                    </div>
                ))}
            </div>
        </div>
    );
}
