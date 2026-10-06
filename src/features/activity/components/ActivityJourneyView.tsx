import { CalendarDaysIcon, GlobeIcon, ImageIcon } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router';

import { EmptyState, LoadingState } from '@/components/layout/PageScaffold';
import { Location } from '@/components/Location';
import { FadeInImage } from '@/components/media/FadeInImage';
import {
    Timeline,
    TimelineDate,
    TimelineIndicator,
    TimelineItem,
    TimelineSeparator
} from '@/components/reui/timeline';
import { UserHoverCard } from '@/components/user-hover-card/UserHoverCard';
import { buildFavoriteIdSet } from '@/domain/favorites/favoriteIdSet';
import { useFriendsLocationsWorldSummaries } from '@/features/friends/useFriendsLocationsWorldSummaries';
import {
    formatDateLabel,
    toLocalDayKey
} from '@/features/instance-history/instance-activity/instanceActivityDate';
import { formatClock, formatDateTime, timeToText } from '@/lib/dateTime';
import { cn } from '@/lib/utils';
import { convertFileSrc } from '@/platform/tauri/assets';
import type {
    ActivityJourneyVisit,
    ScreenshotLibraryImage,
    ScreenshotWindowImages
} from '@/platform/tauri/bindings';
import { openUserDialog, openWorldDialog } from '@/services/dialogService';
import { requestScreenshotThumbnail } from '@/services/screenshotThumbnailQueueService';
import { useFavoriteStore } from '@/state/favoriteStore';
import { useModalStore } from '@/state/modalStore';
import { Spinner } from '@/ui/shadcn/spinner';

import {
    buildJourneyVisitView,
    groupJourneyVisitsByDay,
    JOURNEY_DENSITY_LIMITS,
    journeyDayBounds,
    journeyVisitKey,
    summarizeJourneyDay,
    type JourneyDensity,
    type JourneyPerson,
    type JourneyVisitView
} from '../activityJourneyModel';
import { useActivityUserAvatars } from '../useActivityUserAvatars';
import { Face } from './ActivityPeopleExhibit';

type JourneyPhotos = ReadonlyMap<string, ScreenshotWindowImages>;

const ITEM_OFFSET = 'group-data-[orientation=vertical]/timeline:ms-24';
const TIME_COLUMN = 'absolute top-0 -left-24 w-16 text-right tabular-nums';
const AXIS = 'group-data-[orientation=vertical]/timeline:-left-4';

const DENSITY_STYLES = {
    standard: {
        itemGap: 'group-data-[orientation=vertical]/timeline:not-last:pb-10',
        contentGap: 'gap-3',
        worldImage: 'w-44',
        worldName: 'text-foreground text-[15px] font-semibold',
        avatar: 'size-8',
        twoLine: true,
        people: 'flex min-w-0 flex-wrap items-center gap-x-4 gap-y-1',
        photo: 'w-36',
        photosBelow: true
    },
    compact: {
        itemGap: 'group-data-[orientation=vertical]/timeline:not-last:pb-6',
        contentGap: 'gap-2.5',
        worldImage: 'w-40',
        worldName: 'text-foreground text-sm font-semibold',
        avatar: 'size-6',
        twoLine: false,
        people: 'flex min-w-0 flex-wrap items-center gap-x-3 gap-y-0.5',
        photo: 'w-28',
        photosBelow: true
    },
    dense: {
        itemGap: 'group-data-[orientation=vertical]/timeline:not-last:pb-4',
        contentGap: 'gap-2',
        worldImage: 'w-32',
        worldName: 'text-foreground text-sm font-semibold',
        avatar: 'size-6',
        twoLine: false,
        people: 'flex min-w-0 flex-wrap items-center gap-x-3 gap-y-0.5',
        photo: 'w-24',
        photosBelow: false
    }
} as const;

type DensityStyle = (typeof DENSITY_STYLES)[JourneyDensity];

function PersonChip({
    person,
    avatarUrl,
    style
}: {
    person: JourneyPerson;
    avatarUrl: string;
    style: DensityStyle;
}) {
    return (
        <UserHoverCard userId={person.userId} side="bottom">
            <button
                type="button"
                onClick={() =>
                    openUserDialog({
                        userId: person.userId,
                        title: person.displayName
                    })
                }
                className={cn(
                    'flex max-w-52 min-w-0 items-center gap-2 rounded-full text-left transition-opacity duration-100 ease-out hover:opacity-85 active:opacity-70',
                    style.twoLine ? 'h-10' : 'h-7'
                )}
            >
                <Face url={avatarUrl} className={style.avatar} />
                <span className="flex min-w-0 flex-col leading-4">
                    <span className="text-foreground/90 truncate text-sm">
                        {person.displayName}
                    </span>
                    {style.twoLine && person.sharedMs > 0 ? (
                        <span className="text-muted-foreground text-xs tabular-nums">
                            {timeToText(person.sharedMs)}
                        </span>
                    ) : null}
                </span>
            </button>
        </UserHoverCard>
    );
}

function StrangerName({ person }: { person: JourneyPerson }) {
    return (
        <button
            type="button"
            disabled={!person.userId}
            onClick={() =>
                openUserDialog({
                    userId: person.userId,
                    title: person.displayName
                })
            }
            className="text-muted-foreground enabled:hover:text-foreground max-w-40 truncate text-left text-sm transition-colors"
        >
            {person.displayName}
        </button>
    );
}

function PersonEntry({
    person,
    avatarOf,
    style
}: {
    person: JourneyPerson;
    avatarOf: (userId: string) => string;
    style: DensityStyle;
}) {
    return person.userId && (person.isFriend || person.isFavorite) ? (
        <PersonChip
            person={person}
            avatarUrl={avatarOf(person.userId)}
            style={style}
        />
    ) : (
        <StrangerName person={person} />
    );
}

function JourneyPhoto({
    image,
    className,
    onOpen
}: {
    image: ScreenshotLibraryImage;
    className: string;
    onOpen: () => void;
}) {
    const [url, setUrl] = useState('');
    const [failed, setFailed] = useState(false);

    useEffect(() => {
        let active = true;
        setUrl('');
        setFailed(false);
        const request = requestScreenshotThumbnail(image.path);
        request.promise
            .then((thumbnailPath) => {
                if (active) {
                    setUrl(
                        convertFileSrc(
                            String(thumbnailPath || ''),
                            'vrcx-0-thumb'
                        )
                    );
                }
            })
            .catch(() => {
                if (active) {
                    setFailed(true);
                }
            });
        return () => {
            active = false;
            request.cancel();
        };
    }, [image.modifiedAt, image.path, image.sizeBytes]);

    return (
        <button
            type="button"
            onClick={onOpen}
            aria-label={image.fileName}
            className={cn(
                'relative aspect-video shrink-0 overflow-hidden rounded-md bg-[var(--act-track)] transition-opacity duration-100 ease-out hover:opacity-90 active:opacity-75',
                className
            )}
        >
            {url ? (
                <FadeInImage
                    src={url}
                    alt=""
                    loading="lazy"
                    decoding="async"
                    className="size-full object-cover"
                />
            ) : failed ? (
                <ImageIcon className="text-muted-foreground absolute inset-0 m-auto size-4" />
            ) : null}
        </button>
    );
}

function JourneyPhotos({
    worldName,
    photos,
    limit,
    style
}: {
    worldName: string;
    photos: ScreenshotWindowImages;
    limit: number;
    style: DensityStyle;
}) {
    const { t } = useTranslation();
    const navigate = useNavigate();
    const openImagePreview = useModalStore((state) => state.openImagePreview);
    const shownImages = photos.images.slice(0, limit);
    const hiddenCount = photos.total - shownImages.length;

    function openInManager() {
        const first = photos.images[0];
        if (!first) {
            return;
        }
        const params = new URLSearchParams();
        params.set('path', first.path);
        navigate(`/tools/screenshot-metadata?${params.toString()}`);
    }

    return (
        <div className="flex min-w-0 flex-wrap gap-1.5">
            {shownImages.map((image) => (
                <JourneyPhoto
                    key={image.path}
                    image={image}
                    className={style.photo}
                    onOpen={() =>
                        openImagePreview({
                            url: convertFileSrc(image.path, 'vrcx-0-img'),
                            title: worldName,
                            fileName: image.fileName,
                            sourcePath: image.path
                        })
                    }
                />
            ))}
            {hiddenCount > 0 ? (
                <button
                    type="button"
                    onClick={openInManager}
                    aria-label={t('view.activity.journey.open_in_screenshots')}
                    title={t('view.activity.journey.open_in_screenshots')}
                    className="text-muted-foreground hover:text-foreground flex w-12 shrink-0 items-center justify-center rounded-md bg-[var(--act-track)] text-sm tabular-nums transition-colors"
                >
                    +{hiddenCount}
                </button>
            ) : null}
        </div>
    );
}

function WorldImage({
    visit,
    imageUrl,
    className
}: {
    visit: ActivityJourneyVisit;
    imageUrl: string;
    className: string;
}) {
    const fallback = <GlobeIcon className="text-muted-foreground size-5" />;
    return (
        <button
            type="button"
            disabled={!visit.worldId}
            aria-label={visit.worldName}
            onClick={() =>
                openWorldDialog({
                    worldId: visit.worldId,
                    title: visit.worldName
                })
            }
            className={cn(
                'relative flex aspect-[4/3] shrink-0 items-center justify-center self-start overflow-hidden rounded-lg bg-[var(--act-track)] transition-opacity duration-100 ease-out enabled:hover:opacity-90 enabled:active:opacity-75',
                className
            )}
        >
            {imageUrl ? (
                <FadeInImage
                    src={imageUrl}
                    alt=""
                    loading="lazy"
                    decoding="async"
                    className="size-full object-cover"
                    fallback={fallback}
                />
            ) : (
                fallback
            )}
        </button>
    );
}

function VisitClock({
    visit,
    dayStartMs
}: {
    visit: ActivityJourneyVisit;
    dayStartMs: number;
}) {
    return visit.startMs < dayStartMs
        ? formatDateTime(visit.startMs, {
              month: 'numeric',
              day: 'numeric',
              hour: '2-digit',
              minute: '2-digit'
          })
        : formatClock(visit.startMs);
}

function VisitItem({
    view,
    step,
    photos,
    photoLimit,
    worldImageUrl,
    dayStartMs,
    style,
    avatarOf
}: {
    view: JourneyVisitView;
    step: number;
    photos: ScreenshotWindowImages | undefined;
    photoLimit: number;
    worldImageUrl: string;
    dayStartMs: number;
    style: DensityStyle;
    avatarOf: (userId: string) => string;
}) {
    const photoStrip =
        photos && photos.total > 0 ? (
            <JourneyPhotos
                worldName={view.visit.worldName}
                photos={photos}
                limit={photoLimit}
                style={style}
            />
        ) : null;
    const { t } = useTranslation();
    const navigate = useNavigate();
    const { visit } = view;
    const unlistedCount = view.peopleCount - view.shownPeople.length;

    function openInInstanceHistory() {
        const params = new URLSearchParams();
        params.set('mode', 'day');
        params.set('day', toLocalDayKey(visit.endMs));
        params.set('location', visit.location);
        params.set('at', String(visit.endMs));
        navigate(`/instance-history?${params.toString()}`);
    }

    return (
        <TimelineItem
            step={step}
            render={<li />}
            className={cn(ITEM_OFFSET, style.itemGap)}
        >
            <TimelineDate className={cn(TIME_COLUMN, 'mb-0')}>
                <span className="text-foreground block text-sm font-semibold">
                    <VisitClock visit={visit} dayStartMs={dayStartMs} />
                </span>
                <span className="block font-normal">
                    {timeToText(view.durationMs)}
                </span>
            </TimelineDate>
            <TimelineIndicator
                className={cn(
                    AXIS,
                    'border-muted-foreground/50 size-3 bg-(--vrcx-0-main-content-surface) group-data-[orientation=vertical]/timeline:top-1'
                )}
            />
            <TimelineSeparator
                className={cn(
                    AXIS,
                    'bg-[var(--act-edge)] group-data-[orientation=vertical]/timeline:h-[calc(100%-1rem)] group-data-[orientation=vertical]/timeline:translate-y-5'
                )}
            />
            <div className="flex min-w-0 flex-wrap gap-x-5 gap-y-2 md:flex-nowrap">
                <WorldImage
                    visit={visit}
                    imageUrl={worldImageUrl}
                    className={style.worldImage}
                />
                <div
                    className={cn(
                        'flex min-w-0 flex-1 flex-col',
                        style.contentGap
                    )}
                >
                    <button
                        type="button"
                        onClick={openInInstanceHistory}
                        className="min-w-0 self-start text-left transition-opacity duration-100 ease-out hover:opacity-85 active:opacity-70"
                    >
                        <Location
                            location={visit.location}
                            hint={visit.worldName}
                            link={false}
                            asButton={false}
                            disableTooltip
                            showInstanceIdInLocation={false}
                            className="min-w-0 text-xs"
                            worldNameClassName={style.worldName}
                        />
                    </button>
                    {view.shownPeople.length > 0 ? (
                        <div className={style.people}>
                            {view.shownPeople.map((person) => (
                                <PersonEntry
                                    key={person.key}
                                    person={person}
                                    avatarOf={avatarOf}
                                    style={style}
                                />
                            ))}
                        </div>
                    ) : null}
                    {view.peopleCount === 0 ? (
                        <p className="text-muted-foreground text-xs">
                            {t('view.activity.journey.alone')}
                        </p>
                    ) : unlistedCount > 0 ? (
                        <p className="text-muted-foreground text-xs">
                            {t('view.activity.journey.others_count', {
                                count: unlistedCount
                            })}
                        </p>
                    ) : null}
                    {style.photosBelow && photoStrip ? (
                        <div className="pt-1">{photoStrip}</div>
                    ) : null}
                </div>
                {!style.photosBelow && photoStrip ? (
                    <div className="w-full md:w-auto md:shrink-0">
                        {photoStrip}
                    </div>
                ) : null}
            </div>
        </TimelineItem>
    );
}

function BriefVisitItem({
    view,
    step,
    dayStartMs
}: {
    view: JourneyVisitView;
    step: number;
    dayStartMs: number;
}) {
    const { visit } = view;
    return (
        <TimelineItem
            step={step}
            render={<li />}
            className={cn(
                ITEM_OFFSET,
                'group-data-[orientation=vertical]/timeline:not-last:pb-3'
            )}
        >
            <TimelineDate className={cn(TIME_COLUMN, 'mb-0 font-normal')}>
                <VisitClock visit={visit} dayStartMs={dayStartMs} />
            </TimelineDate>
            <TimelineIndicator
                className={cn(
                    AXIS,
                    'bg-muted-foreground/40 size-1.5 border-0 group-data-[orientation=vertical]/timeline:top-1.5'
                )}
            />
            <TimelineSeparator
                className={cn(
                    AXIS,
                    'bg-[var(--act-edge)] group-data-[orientation=vertical]/timeline:h-[calc(100%-0.5rem)] group-data-[orientation=vertical]/timeline:translate-y-3'
                )}
            />
            <div className="text-muted-foreground flex min-w-0 items-center gap-2 text-xs">
                <Location
                    location={visit.location}
                    hint={visit.worldName}
                    disableTooltip
                    showInstanceIdInLocation={false}
                    className="min-w-0"
                />
                <span className="shrink-0 tabular-nums">
                    {timeToText(view.durationMs)}
                </span>
            </div>
        </TimelineItem>
    );
}

function JourneyDay({
    dayKey,
    dayVisits,
    photos,
    favoriteIdSet,
    homeWorldId,
    density,
    worldImageOf,
    avatarOf
}: {
    dayKey: string;
    dayVisits: ActivityJourneyVisit[];
    photos: JourneyPhotos;
    favoriteIdSet: ReadonlySet<string>;
    homeWorldId: string;
    density: JourneyDensity;
    worldImageOf: (visit: ActivityJourneyVisit) => string;
    avatarOf: (userId: string) => string;
}) {
    const { t } = useTranslation();
    const style = DENSITY_STYLES[density];
    const limits = JOURNEY_DENSITY_LIMITS[density];
    const dayStartMs = useMemo(() => journeyDayBounds(dayKey).fromMs, [dayKey]);
    const views = useMemo(
        () =>
            dayVisits.map((visit) =>
                buildJourneyVisitView(
                    visit,
                    favoriteIdSet,
                    photos.get(journeyVisitKey(visit))?.total ?? 0,
                    homeWorldId,
                    limits
                )
            ),
        [dayVisits, favoriteIdSet, homeWorldId, limits, photos]
    );
    const summary = useMemo(
        () => summarizeJourneyDay(dayVisits, favoriteIdSet),
        [dayVisits, favoriteIdSet]
    );
    const photoCount = dayVisits.reduce(
        (sum, visit) => sum + (photos.get(journeyVisitKey(visit))?.total ?? 0),
        0
    );
    const caption = [
        t('view.activity.journey.world_count', { count: summary.worldCount }),
        summary.friendCount > 0
            ? t('view.activity.journey.friend_count', {
                  count: summary.friendCount
              })
            : '',
        photoCount > 0
            ? t('view.activity.journey.photo_count', { count: photoCount })
            : ''
    ]
        .filter(Boolean)
        .join(' · ');

    return (
        <section className="pb-6">
            <header className="sticky top-0 z-10 mb-4 flex items-baseline gap-3 border-b border-[var(--act-edge)] bg-(--vrcx-0-main-content-surface) py-2.5">
                <h3 className="text-foreground text-sm font-semibold">
                    {formatDateLabel(dayKey)}
                </h3>
                <span className="text-muted-foreground text-xs">{caption}</span>
            </header>
            <Timeline value={0} render={<ol />}>
                {views.map((view, index) =>
                    view.brief ? (
                        <BriefVisitItem
                            key={view.key}
                            view={view}
                            step={index + 1}
                            dayStartMs={dayStartMs}
                        />
                    ) : (
                        <VisitItem
                            key={view.key}
                            view={view}
                            step={index + 1}
                            photos={photos.get(view.key)}
                            photoLimit={limits.photos}
                            worldImageUrl={worldImageOf(view.visit)}
                            dayStartMs={dayStartMs}
                            style={style}
                            avatarOf={avatarOf}
                        />
                    )
                )}
            </Timeline>
        </section>
    );
}

export function ActivityJourneyView({
    visits,
    photos,
    loading,
    hasMore,
    error,
    filtered,
    homeWorldId,
    density,
    onLoadMore
}: {
    visits: ActivityJourneyVisit[];
    photos: JourneyPhotos;
    loading: boolean;
    hasMore: boolean;
    error: string;
    filtered: boolean;
    homeWorldId: string;
    density: JourneyDensity;
    onLoadMore: () => void;
}) {
    const { t } = useTranslation();
    const sentinelRef = useRef<HTMLDivElement | null>(null);
    const localFriendFavorites = useFavoriteStore(
        (state) => state.localFriendFavorites
    );
    const remoteFavoriteFriendIds = useFavoriteStore(
        (state) => state.favoriteFriendIds
    );
    const favoriteIdSet = useMemo(
        () => buildFavoriteIdSet(remoteFavoriteFriendIds, localFriendFavorites),
        [localFriendFavorites, remoteFavoriteFriendIds]
    );
    const groups = useMemo(() => groupJourneyVisitsByDay(visits), [visits]);
    const missingWorldIds = useMemo(
        () => [
            ...new Set(
                visits
                    .filter((visit) => !visit.worldImageUrl && visit.worldId)
                    .map((visit) => visit.worldId)
            )
        ],
        [visits]
    );
    const fetchedWorlds = useFriendsLocationsWorldSummaries(missingWorldIds);
    const avatarUserIds = useMemo(
        () => [
            ...new Set(
                visits.flatMap((visit) =>
                    visit.companions
                        .filter(
                            (companion) =>
                                companion.userId &&
                                (companion.isFriend ||
                                    favoriteIdSet.has(companion.userId))
                        )
                        .map((companion) => companion.userId)
                )
            )
        ],
        [favoriteIdSet, visits]
    );
    const avatarOf = useActivityUserAvatars(avatarUserIds);

    useEffect(() => {
        const sentinel = sentinelRef.current;
        if (
            !hasMore ||
            loading ||
            !sentinel ||
            typeof IntersectionObserver !== 'function'
        ) {
            return undefined;
        }
        const observer = new IntersectionObserver(
            (entries) => {
                if (entries.some((entry) => entry.isIntersecting)) {
                    onLoadMore();
                }
            },
            { rootMargin: '600px' }
        );
        observer.observe(sentinel);
        return () => {
            observer.disconnect();
        };
    }, [hasMore, loading, onLoadMore, visits.length]);

    if (error && visits.length === 0) {
        return (
            <EmptyState
                icon={CalendarDaysIcon}
                title={t('view.activity.error.failed_to_load')}
                description={error}
            />
        );
    }
    if (loading && visits.length === 0) {
        return <LoadingState />;
    }
    if (visits.length === 0 && !hasMore) {
        return (
            <EmptyState
                icon={CalendarDaysIcon}
                title={t(
                    filtered
                        ? 'view.activity.journey.empty.filtered_title'
                        : 'view.activity.journey.empty.title'
                )}
                description={t('view.activity.journey.empty.description')}
            />
        );
    }

    return (
        <div className="mx-auto w-full max-w-[120rem] px-1">
            {groups.map((group) => (
                <JourneyDay
                    key={group.dayKey}
                    dayKey={group.dayKey}
                    dayVisits={group.visits}
                    photos={photos}
                    favoriteIdSet={favoriteIdSet}
                    homeWorldId={homeWorldId}
                    density={density}
                    worldImageOf={(visit) =>
                        visit.worldImageUrl ||
                        fetchedWorlds.get(visit.worldId)?.thumbnailUrl ||
                        ''
                    }
                    avatarOf={avatarOf}
                />
            ))}
            <div ref={sentinelRef} className="flex justify-center py-4">
                {loading ? <Spinner /> : null}
            </div>
        </div>
    );
}
