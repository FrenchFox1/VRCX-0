import {
    FolderOpenIcon,
    FolderSearchIcon,
    ImageIcon,
    ImageOffIcon,
    ImagesIcon,
    RefreshCwIcon,
    ShuffleIcon
} from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

import { formatDateFilter, formatRelativeTime } from '@/lib/dateTime';
import { cn } from '@/lib/utils';
import type {
    BackgroundImageCustomSource,
    BackgroundImageMode,
    BackgroundImageProviderId,
    BackgroundImageSnapshot
} from '@/platform/tauri/bindings';
import {
    backgroundImageRemoteProviders,
    chooseBackgroundImageFiles,
    chooseBackgroundImageFolder,
    isBackgroundImageCustomSourceRotating,
    refreshBackgroundImage,
    setBackgroundImageCustomRotationIntervalMinutes,
    setBackgroundImageMode,
    setBackgroundImageProvider
} from '@/services/background-image/backgroundImageService';
import { openFolderAndSelectItem } from '@/services/shellIntegrationService';
import { useBackgroundImageStore } from '@/state/backgroundImageStore';
import { Button } from '@/ui/shadcn/button';
import { Card, CardContent } from '@/ui/shadcn/card';
import {
    InputGroup,
    InputGroupAddon,
    InputGroupInput,
    InputGroupText
} from '@/ui/shadcn/input-group';
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue
} from '@/ui/shadcn/select';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/ui/shadcn/tooltip';

const DEFAULT_ROTATION_INTERVAL_MINUTES = 60;
const MIN_ROTATION_INTERVAL_MINUTES = 1;
const MAX_ROTATION_INTERVAL_MINUTES = 24 * 60;

type RotationPresetValue = '15' | '30' | '60' | '180';
type RotationChoice = RotationPresetValue | 'custom';

const ROTATION_PRESETS: {
    value: RotationPresetValue;
    minutes: number;
}[] = [
    { value: '15', minutes: 15 },
    { value: '30', minutes: 30 },
    { value: '60', minutes: 60 },
    { value: '180', minutes: 180 }
];

function rotationChoiceFromMinutes(minutes: number): RotationChoice {
    return (
        ROTATION_PRESETS.find((preset) => preset.minutes === minutes)?.value ??
        'custom'
    );
}

function fileNameFromPath(path?: string | null): string {
    const normalizedPath = path ?? '';
    return (
        normalizedPath.split(/[\\/]/).filter(Boolean).pop() || normalizedPath
    );
}

function directoryFromPath(path: string): string {
    const separatorAt = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'));
    return separatorAt > 0 ? path.slice(0, separatorAt) : path;
}

function formatResolvedAt(value: string): string {
    const formatted = formatDateFilter(value, 'long');
    return formatted === '-' ? value : formatted;
}

function resolveProviderName(providerId?: BackgroundImageProviderId): string {
    return (
        backgroundImageRemoteProviders.find(
            (provider) => provider.id === providerId
        )?.name ||
        providerId ||
        ''
    );
}

function CurrentBackgroundImageSummary({
    enabled,
    loading,
    mode,
    providerId,
    customSource,
    snapshot,
    onRefresh
}: {
    enabled: boolean;
    loading: boolean;
    mode: BackgroundImageMode;
    providerId: BackgroundImageProviderId;
    customSource: BackgroundImageCustomSource | null;
    snapshot: BackgroundImageSnapshot | null;
    onRefresh: () => void;
}) {
    const { t } = useTranslation();
    const [imageFailed, setImageFailed] = useState(false);
    const [imageReady, setImageReady] = useState(false);

    useEffect(() => {
        setImageFailed(false);
        setImageReady(false);
    }, [snapshot?.imageUrl]);

    async function showCurrentImageInFolder() {
        if (!snapshot?.imagePath) {
            return;
        }
        try {
            await openFolderAndSelectItem(snapshot.imagePath, false);
        } catch {
            toast.error(t('view.background_image.toast.failed_to_open_folder'));
        }
    }

    const providerName = resolveProviderName(
        snapshot?.providerId || providerId
    );
    const imageCount = snapshot?.imageCount || customSource?.paths.length || 0;
    const localPath =
        snapshot?.imagePath ||
        (customSource?.kind === 'folder'
            ? customSource.folderPath
            : customSource?.paths[0]);
    const title =
        snapshot?.mode === 'custom'
            ? snapshot.title || fileNameFromPath(snapshot.imagePath)
            : snapshot?.title;
    const isCustom = snapshot?.mode === 'custom' || mode === 'custom';
    const sourceType = !isCustom
        ? providerName
        : customSource?.kind === 'folder'
          ? t('view.background_image.settings.source_type_folder')
          : imageCount > 1
            ? t('view.background_image.settings.source_type_files')
            : t('view.background_image.settings.source_type_file');
    const isFolderSource = mode === 'custom' && customSource?.kind === 'folder';
    const relativeResolvedAt = snapshot
        ? formatRelativeTime(snapshot.resolvedAt)
        : '';
    const metaParts = !snapshot
        ? []
        : (isCustom
              ? [
                    imageCount > 1
                        ? t('view.background_image.settings.image_count', {
                              count: imageCount
                          })
                        : '',
                    relativeResolvedAt
                ]
              : [
                    snapshot.author,
                    snapshot.license,
                    snapshot.source,
                    relativeResolvedAt
                ]
          ).filter((part) => part && part !== sourceType);

    return (
        <div className="border-border/70 bg-muted/20 flex min-w-0 flex-col gap-3 rounded-lg border p-2.5 sm:flex-row">
            <div className="bg-muted text-muted-foreground grid size-24 shrink-0 place-items-center overflow-hidden rounded-md border">
                {snapshot?.imageUrl && !imageFailed ? (
                    <img
                        src={snapshot.imageUrl}
                        alt={
                            title ||
                            t('view.background_image.settings.current_image')
                        }
                        className={cn(
                            'size-full object-cover transition-opacity duration-150 ease-out',
                            imageReady ? 'opacity-100' : 'opacity-0'
                        )}
                        loading="lazy"
                        ref={(node) => {
                            if (node?.complete) {
                                setImageReady(true);
                            }
                        }}
                        onLoad={() => setImageReady(true)}
                        onError={() => setImageFailed(true)}
                    />
                ) : (
                    <ImageOffIcon className="size-6 opacity-70" />
                )}
            </div>
            <div className="grid min-w-0 flex-1 content-start gap-1 text-sm">
                <div className="flex min-w-0 flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                    <div className="flex min-w-0 flex-wrap items-center gap-2">
                        <div className="truncate font-medium" title={title}>
                            {title ||
                                t('view.background_image.settings.no_image')}
                        </div>
                        <span
                            className="bg-muted text-muted-foreground rounded px-1.5 py-0.5 text-xs"
                            title={
                                isFolderSource
                                    ? customSource.folderPath
                                    : undefined
                            }
                        >
                            {sourceType}
                        </span>
                    </div>
                    {enabled ? (
                        <div className="flex shrink-0 flex-wrap gap-2 self-start">
                            {isFolderSource && snapshot?.imagePath ? (
                                <Button
                                    type="button"
                                    variant="outline"
                                    size="sm"
                                    className="h-7"
                                    disabled={loading}
                                    onClick={showCurrentImageInFolder}
                                >
                                    <FolderSearchIcon data-icon="inline-start" />
                                    {t(
                                        'view.background_image.action.show_in_folder'
                                    )}
                                </Button>
                            ) : null}
                            <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                className="h-7"
                                disabled={loading}
                                onClick={onRefresh}
                            >
                                {isFolderSource ? (
                                    <ShuffleIcon data-icon="inline-start" />
                                ) : (
                                    <RefreshCwIcon data-icon="inline-start" />
                                )}
                                {t(
                                    isFolderSource
                                        ? 'view.background_image.action.change_image'
                                        : 'view.background_image.action.refresh'
                                )}
                            </Button>
                        </div>
                    ) : null}
                </div>
                {snapshot ? (
                    <>
                        {isCustom && localPath ? (
                            <div
                                className="text-muted-foreground truncate text-xs"
                                title={localPath}
                            >
                                {directoryFromPath(localPath)}
                            </div>
                        ) : null}
                        <div
                            className="text-muted-foreground truncate text-xs"
                            title={`${t('view.background_image.settings.resolved_at')}: ${formatResolvedAt(snapshot.resolvedAt)}`}
                        >
                            {metaParts.join(' · ')}
                        </div>
                    </>
                ) : (
                    <div className="text-muted-foreground text-xs">
                        {t(
                            'view.background_image.settings.no_image_description'
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}

export function BackgroundImageSection() {
    const { t } = useTranslation();
    const mode = useBackgroundImageStore((state) => state.mode);
    const enabled = useBackgroundImageStore((state) => state.enabled);
    const providerId = useBackgroundImageStore((state) => state.providerId);
    const customSource = useBackgroundImageStore((state) => state.customSource);
    const snapshot = useBackgroundImageStore((state) => state.snapshot);
    const loading = useBackgroundImageStore((state) => state.loading);
    const rotationIntervalMinutes =
        customSource?.rotationIntervalMinutes ??
        DEFAULT_ROTATION_INTERVAL_MINUTES;
    const [rotationChoice, setRotationChoice] = useState<RotationChoice>(() =>
        rotationChoiceFromMinutes(rotationIntervalMinutes)
    );
    const [rotationIntervalDraft, setRotationIntervalDraft] = useState(
        String(rotationIntervalMinutes)
    );
    const showRotation = isBackgroundImageCustomSourceRotating(
        customSource,
        snapshot?.imageCount
    );

    useEffect(() => {
        setRotationChoice(rotationChoiceFromMinutes(rotationIntervalMinutes));
        setRotationIntervalDraft(String(rotationIntervalMinutes));
    }, [rotationIntervalMinutes]);

    async function updateMode(nextMode: BackgroundImageMode) {
        try {
            const updated = await setBackgroundImageMode(nextMode);
            if (updated) {
                toast.success(t('view.background_image.toast.enabled'));
            }
        } catch (error) {
            toast.error(
                error instanceof Error
                    ? error.message
                    : t('view.background_image.toast.failed')
            );
        }
    }

    async function updateProvider(nextProviderId: BackgroundImageProviderId) {
        try {
            await setBackgroundImageProvider(nextProviderId);
            if (enabled && mode === 'daily') {
                toast.success(t('view.background_image.toast.enabled'));
                return;
            }
            toast.success(t('common.settings_saved'));
        } catch (error) {
            toast.error(
                error instanceof Error
                    ? error.message
                    : t('view.background_image.toast.failed')
            );
        }
    }

    async function refreshBackground() {
        try {
            const refreshed = await refreshBackgroundImage();
            if (
                !refreshed ||
                (mode === 'custom' && customSource?.kind === 'folder')
            ) {
                return;
            }
            toast.success(t('view.background_image.toast.refreshed'));
        } catch (error) {
            toast.error(
                error instanceof Error
                    ? error.message
                    : t('view.background_image.toast.failed')
            );
        }
    }

    async function selectFiles() {
        try {
            const selected = await chooseBackgroundImageFiles();
            if (selected) {
                toast.success(t('view.background_image.toast.enabled'));
            }
        } catch (error) {
            toast.error(
                error instanceof Error
                    ? error.message
                    : t('view.background_image.toast.no_images')
            );
        }
    }

    async function selectFolder() {
        try {
            const selected = await chooseBackgroundImageFolder();
            if (selected) {
                toast.success(t('view.background_image.toast.enabled'));
            }
        } catch (error) {
            toast.error(
                error instanceof Error
                    ? error.message
                    : t('view.background_image.toast.no_images')
            );
        }
    }

    async function updateRotationIntervalMinutes(value: number) {
        try {
            await setBackgroundImageCustomRotationIntervalMinutes(value);
            toast.success(t('common.settings_saved'));
        } catch (error) {
            setRotationChoice(
                rotationChoiceFromMinutes(rotationIntervalMinutes)
            );
            setRotationIntervalDraft(String(rotationIntervalMinutes));
            toast.error(
                error instanceof Error
                    ? error.message
                    : t('view.background_image.toast.failed')
            );
        }
    }

    async function commitRotationIntervalDraft() {
        const value = Number(rotationIntervalDraft);
        if (
            !Number.isInteger(value) ||
            value < MIN_ROTATION_INTERVAL_MINUTES ||
            value > MAX_ROTATION_INTERVAL_MINUTES
        ) {
            setRotationIntervalDraft(String(rotationIntervalMinutes));
            return;
        }
        if (value !== rotationIntervalMinutes) {
            await updateRotationIntervalMinutes(value);
        }
    }

    return (
        <Card>
            <CardContent className="flex flex-col gap-3 p-3">
                <div className="flex min-w-0 flex-col gap-2 lg:flex-row lg:items-center lg:justify-between">
                    <div className="flex min-w-0 items-center gap-2 text-sm font-medium">
                        <ImageIcon data-icon="inline-start" />
                        {t('view.background_image.settings.header')}
                    </div>
                    <div className="flex min-w-0 flex-wrap gap-2">
                        <Select<BackgroundImageMode>
                            value={mode === 'custom' ? 'custom' : 'daily'}
                            items={[
                                {
                                    value: 'daily',
                                    label: t('view.background_image.mode.daily')
                                },
                                {
                                    value: 'custom',
                                    label: t(
                                        'view.background_image.mode.custom'
                                    )
                                }
                            ]}
                            disabled={loading}
                            onValueChange={(value) => {
                                if (value) {
                                    updateMode(value);
                                }
                            }}
                        >
                            <SelectTrigger size="sm" className="min-w-40">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectGroup>
                                    <SelectItem value="daily">
                                        {t('view.background_image.mode.daily')}
                                    </SelectItem>
                                    <SelectItem value="custom">
                                        {t('view.background_image.mode.custom')}
                                    </SelectItem>
                                </SelectGroup>
                            </SelectContent>
                        </Select>
                        {mode === 'daily' ? (
                            <Select<BackgroundImageProviderId>
                                value={providerId}
                                items={backgroundImageRemoteProviders.map(
                                    (provider) => ({
                                        value: provider.id,
                                        label: provider.name
                                    })
                                )}
                                disabled={loading}
                                onValueChange={(value) => {
                                    if (value) {
                                        updateProvider(value);
                                    }
                                }}
                            >
                                <SelectTrigger size="sm" className="min-w-52">
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    <SelectGroup>
                                        {backgroundImageRemoteProviders.map(
                                            (provider) => (
                                                <SelectItem
                                                    key={provider.id}
                                                    value={provider.id}
                                                >
                                                    {provider.name}
                                                </SelectItem>
                                            )
                                        )}
                                    </SelectGroup>
                                </SelectContent>
                            </Select>
                        ) : null}
                    </div>
                </div>
                {providerId === 'nasa-apod-safe' && mode === 'daily' ? (
                    <p className="text-muted-foreground text-xs italic">
                        {t('view.background_image.settings.apod_note')}
                    </p>
                ) : null}
                <CurrentBackgroundImageSummary
                    enabled={enabled}
                    loading={loading}
                    mode={mode}
                    providerId={providerId}
                    customSource={customSource}
                    snapshot={enabled ? snapshot : null}
                    onRefresh={refreshBackground}
                />
                {mode === 'custom' ? (
                    <div className="border-border/70 flex min-w-0 flex-wrap items-center gap-2 border-t pt-3">
                        <Button
                            type="button"
                            variant="outline"
                            size="sm"
                            disabled={loading}
                            onClick={selectFiles}
                        >
                            <ImagesIcon data-icon="inline-start" />
                            {t('view.background_image.action.select_images')}
                        </Button>
                        <Tooltip>
                            <TooltipTrigger
                                render={
                                    <Button
                                        type="button"
                                        variant="outline"
                                        size="sm"
                                        disabled={loading}
                                        onClick={selectFolder}
                                    >
                                        <FolderOpenIcon data-icon="inline-start" />
                                        {t(
                                            'view.background_image.action.select_folder'
                                        )}
                                    </Button>
                                }
                            />
                            <TooltipContent>
                                {t(
                                    'view.background_image.settings.folder_recursive_note'
                                )}
                            </TooltipContent>
                        </Tooltip>
                        {showRotation ? (
                            <div className="flex flex-wrap items-center gap-2 sm:ml-auto">
                                <span className="text-muted-foreground text-xs">
                                    {t(
                                        'view.background_image.settings.rotation'
                                    )}
                                </span>
                                <Select<RotationChoice>
                                    value={rotationChoice}
                                    items={[
                                        ...ROTATION_PRESETS.map((preset) => ({
                                            value: preset.value,
                                            label: `${preset.minutes} ${t('view.background_image.rotation.minutes')}`
                                        })),
                                        {
                                            value: 'custom',
                                            label: t(
                                                'view.background_image.rotation.custom'
                                            )
                                        }
                                    ]}
                                    disabled={loading}
                                    onValueChange={(value) => {
                                        if (!value) {
                                            return;
                                        }
                                        setRotationChoice(value);
                                        if (value !== 'custom') {
                                            setRotationIntervalDraft(value);
                                            void updateRotationIntervalMinutes(
                                                Number(value)
                                            );
                                        }
                                    }}
                                >
                                    <SelectTrigger
                                        size="sm"
                                        className="min-w-36"
                                    >
                                        <SelectValue />
                                    </SelectTrigger>
                                    <SelectContent>
                                        <SelectGroup>
                                            {ROTATION_PRESETS.map((preset) => (
                                                <SelectItem
                                                    key={preset.value}
                                                    value={preset.value}
                                                >
                                                    {preset.minutes}{' '}
                                                    {t(
                                                        'view.background_image.rotation.minutes'
                                                    )}
                                                </SelectItem>
                                            ))}
                                            <SelectItem value="custom">
                                                {t(
                                                    'view.background_image.rotation.custom'
                                                )}
                                            </SelectItem>
                                        </SelectGroup>
                                    </SelectContent>
                                </Select>
                                {rotationChoice === 'custom' ? (
                                    <InputGroup className="w-32">
                                        <InputGroupInput
                                            type="number"
                                            min={MIN_ROTATION_INTERVAL_MINUTES}
                                            max={MAX_ROTATION_INTERVAL_MINUTES}
                                            step={1}
                                            disabled={loading}
                                            value={rotationIntervalDraft}
                                            onChange={(event) =>
                                                setRotationIntervalDraft(
                                                    event.currentTarget.value
                                                )
                                            }
                                            onBlur={() => {
                                                void commitRotationIntervalDraft();
                                            }}
                                            onKeyDown={(event) => {
                                                if (event.key === 'Enter') {
                                                    event.currentTarget.blur();
                                                }
                                            }}
                                            aria-label={t(
                                                'view.background_image.settings.rotation'
                                            )}
                                        />
                                        <InputGroupAddon align="inline-end">
                                            <InputGroupText>
                                                {t(
                                                    'view.background_image.rotation.minutes'
                                                )}
                                            </InputGroupText>
                                        </InputGroupAddon>
                                    </InputGroup>
                                ) : null}
                            </div>
                        ) : null}
                    </div>
                ) : null}
            </CardContent>
        </Card>
    );
}
