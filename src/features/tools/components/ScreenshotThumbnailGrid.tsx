import { CameraIcon, ImageIcon } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

import {
    useLocationMetadataBatch,
    type LocationMetadataEntry
} from '@/components/location/useLocationMetadata';
import { FadeInImage } from '@/components/media/FadeInImage';
import { convertFileSrc } from '@/platform/tauri/assets';
import type { ScreenshotLibraryImage } from '@/platform/tauri/bindings';
import { parseLocation } from '@/shared/utils/location';
import { Badge } from '@/ui/shadcn/badge';
import { Button } from '@/ui/shadcn/button';
import { Skeleton } from '@/ui/shadcn/skeleton';
import { Spinner } from '@/ui/shadcn/spinner';

import { formatScreenshotDateTime } from '../screenshotMetadataValues';
import { requestScreenshotThumbnail } from '../screenshotThumbnailQueue';

function firstText(...values: Array<string | null | undefined>) {
    return values.map((value) => (value ?? '').trim()).find(Boolean);
}

type ScreenshotThumbnailItem = ScreenshotLibraryImage;

const WORLD_REFERENCE_PATTERN =
    /(?:^|\b)wrld_[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}(?::|$|\s)/i;

function normalizeThumbnailWorldName(value: string | null | undefined) {
    const normalizedValue = (value ?? '').trim();
    if (!normalizedValue || WORLD_REFERENCE_PATTERN.test(normalizedValue)) {
        return '';
    }
    return normalizedValue;
}

function resolveThumbnailLocation(item: ScreenshotThumbnailItem) {
    const metadataWorld = item.metadata?.world || {};
    return (
        firstText(metadataWorld.instanceId, metadataWorld.id, item.worldId) ||
        ''
    );
}

function resolveDirectThumbnailTitle(
    item: ScreenshotThumbnailItem,
    worldNameHint = ''
) {
    const metadataWorld = item.metadata?.world || {};
    return firstText(
        normalizeThumbnailWorldName(worldNameHint),
        normalizeThumbnailWorldName(item.worldName),
        normalizeThumbnailWorldName(metadataWorld.name)
    );
}

function buildThumbnailLocationEntry(
    item: ScreenshotThumbnailItem
): LocationMetadataEntry | null {
    const directTitle = resolveDirectThumbnailTitle(item);
    if (directTitle) {
        return null;
    }

    const currentLocation = resolveThumbnailLocation(item);
    if (!currentLocation) {
        return null;
    }

    const metadataWorld = item.metadata?.world || {};
    const parsedLocation = parseLocation(currentLocation);
    if (!parsedLocation.worldId) {
        return null;
    }

    return {
        key: item.path,
        locationInfo: parsedLocation,
        currentLocation,
        hint: firstText(item.worldName, metadataWorld.name)
    };
}

export function useScreenshotThumbnailTitleMap(
    items: readonly ScreenshotThumbnailItem[],
    { worldNameHint = '' }: { worldNameHint?: string } = {}
) {
    const entries = useMemo(
        () =>
            items
                .map((item) =>
                    resolveDirectThumbnailTitle(item, worldNameHint)
                        ? null
                        : buildThumbnailLocationEntry(item)
                )
                .filter(Boolean),
        [items, worldNameHint]
    );
    const metadataByKey = useLocationMetadataBatch(entries);

    return useMemo(() => {
        const titleMap = new Map<string, string>();
        for (const item of items) {
            const metadata = metadataByKey.get(item.path);
            titleMap.set(
                item.path,
                firstText(
                    resolveDirectThumbnailTitle(item, worldNameHint),
                    normalizeThumbnailWorldName(metadata?.worldName),
                    normalizeThumbnailWorldName(metadata?.worldNameHint),
                    item.fileName
                ) || item.fileName
            );
        }
        return titleMap;
    }, [items, metadataByKey, worldNameHint]);
}

export function ScreenshotThumbnailCard({
    compact = false,
    item,
    onOpen,
    title = '',
    worldNameHint = ''
}: {
    compact?: boolean;
    item: ScreenshotThumbnailItem;
    onOpen: (path: string) => void;
    title?: string;
    worldNameHint?: string;
}) {
    const { i18n, t } = useTranslation();
    const [thumbnailUrl, setThumbnailUrl] = useState('');
    const [loadState, setLoadState] = useState('idle');

    useEffect(() => {
        let active = true;
        setThumbnailUrl('');
        setLoadState('loading');

        const request = requestScreenshotThumbnail(item.path);
        request.promise
            .then((thumbnailPath) => {
                if (!active) {
                    return;
                }
                setThumbnailUrl(
                    convertFileSrc(String(thumbnailPath || ''), 'vrcx-0-thumb')
                );
                setLoadState('ready');
            })
            .catch(() => {
                if (active) {
                    setLoadState('error');
                }
            });

        return () => {
            active = false;
            request.cancel();
        };
    }, [item.modifiedAt, item.path, item.sizeBytes]);

    const dateLabel = formatScreenshotDateTime(
        item.capturedAt || item.modifiedAt,
        i18n.resolvedLanguage || i18n.language
    );
    const displayTitle =
        title ||
        resolveDirectThumbnailTitle(item, worldNameHint) ||
        item.fileName;
    const cardHeight = compact ? 'h-[156px]' : 'h-[196px]';
    const mediaHeight = compact ? 'h-[94px]' : 'h-[118px]';

    return (
        <Button
            type="button"
            variant="outline"
            className={`bg-card text-card-foreground hover:bg-accent/50 ${cardHeight} min-w-0 flex-col items-stretch justify-start overflow-hidden p-0 text-left has-data-[icon=inline-start]:pl-0`}
            onClick={() => onOpen(item.path)}
        >
            <div
                className={`bg-muted relative flex ${mediaHeight} items-center justify-center overflow-hidden`}
            >
                {thumbnailUrl ? (
                    <FadeInImage
                        src={thumbnailUrl}
                        alt={item.fileName}
                        className="size-full object-cover"
                        loading="lazy"
                        fallback={
                            <div className="text-muted-foreground flex flex-col items-center gap-1 text-xs">
                                <ImageIcon />
                                <span>
                                    {t(
                                        'dialog.screenshot_metadata.thumbnail_failed'
                                    )}
                                </span>
                            </div>
                        }
                    />
                ) : loadState === 'error' ? (
                    <div className="text-muted-foreground flex flex-col items-center gap-1 text-xs">
                        <ImageIcon />
                        <span>
                            {t('dialog.screenshot_metadata.thumbnail_failed')}
                        </span>
                    </div>
                ) : (
                    <>
                        <Skeleton className="size-full rounded-none" />
                        <div className="absolute inset-0 flex items-center justify-center">
                            <Spinner />
                        </div>
                    </>
                )}
            </div>
            <div className="flex min-h-0 flex-1 flex-col gap-1 p-2">
                <div
                    className="truncate text-sm font-medium"
                    title={displayTitle}
                >
                    {displayTitle}
                </div>
                {!compact ? (
                    <div className="text-muted-foreground truncate text-xs">
                        {item.fileName}
                    </div>
                ) : null}
                <div className="text-muted-foreground mt-auto flex items-center gap-1 text-xs">
                    <CameraIcon data-icon="inline-start" />
                    <span className="truncate">{dateLabel}</span>
                </div>
            </div>
        </Button>
    );
}

export function ScreenshotThumbnailGrid({
    compact = false,
    count,
    items,
    onOpen,
    worldNameHint = ''
}: {
    compact?: boolean;
    count?: number;
    items: readonly ScreenshotThumbnailItem[];
    onOpen: (path: string) => void;
    worldNameHint?: string;
}) {
    const { t } = useTranslation();
    const titleMap = useScreenshotThumbnailTitleMap(items, {
        worldNameHint
    });

    return (
        <div className="flex min-h-0 flex-col gap-2">
            {typeof count === 'number' ? (
                <Badge variant="outline" className="w-fit">
                    {t('dialog.screenshot_metadata.image_count', { count })}
                </Badge>
            ) : null}
            <div
                className={
                    compact
                        ? 'grid grid-cols-[repeat(auto-fill,minmax(150px,1fr))] gap-2'
                        : 'grid grid-cols-[repeat(auto-fill,minmax(208px,1fr))] gap-3'
                }
            >
                {items.map((item) => (
                    <ScreenshotThumbnailCard
                        key={item.path}
                        compact={compact}
                        item={item}
                        onOpen={onOpen}
                        title={titleMap.get(item.path)}
                        worldNameHint={worldNameHint}
                    />
                ))}
            </div>
        </div>
    );
}
