import { GlobeIcon, UserIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { AffinityBadge } from '@/components/affinity/AffinityBadge';
import { CurrentInstanceBadge } from '@/components/instances/CurrentInstanceBadge';
import { RegionCodeBadge } from '@/components/location/RegionCodeBadge';
import { useLocationMetadata } from '@/components/location/useLocationMetadata';
import { FadeInImage } from '@/components/media/FadeInImage';
import { resolveSidebarStatusDotClassName } from '@/components/sidebar/friends-sidebar/friendsSidebarModel';
import { UserHoverCard } from '@/components/user-hover-card/UserHoverCard';
import { UserStatusDot } from '@/components/UserStatusDot';
import type { FriendRecord } from '@/domain/friends/types';
import { cn } from '@/lib/utils';
import { userImage } from '@/services/entityMediaService';
import { accessTypeLocaleKeyMap } from '@/shared/constants/accessType';
import { parseLocation, translateAccessType } from '@/shared/utils/location';
import { normalizeString } from '@/shared/utils/string';
import { useRuntimeStore } from '@/state/runtimeStore';
import { Avatar, AvatarFallback, AvatarImage } from '@/ui/shadcn/avatar';

import type { getFriendsLocationsDensityConfig } from '../friendsLocationsDensity';
import type {
    FriendsLocationsWorldGroup,
    FriendsLocationsWorldInstance
} from '../friendsLocationsWorlds';
import type { FriendsLocationsWorldSummary } from '../useFriendsLocationsWorldSummaries';

type FriendsLocationsWorldSectionProps = {
    group: FriendsLocationsWorldGroup;
    summary?: FriendsLocationsWorldSummary;
    densityConfig: ReturnType<typeof getFriendsLocationsDensityConfig>;
    currentUserId?: string | null;
    favoriteIds: ReadonlySet<string>;
    onOpenWorld: (group: FriendsLocationsWorldGroup, name: string) => void;
    onOpenGroup: (groupId: string) => void;
    onOpenUser: (friend: FriendRecord) => void;
};

function FriendChip({
    friend,
    isFavorite,
    statusDotClassName,
    twoLine,
    onOpen
}: {
    friend: FriendRecord;
    isFavorite: boolean;
    statusDotClassName: string;
    twoLine: boolean;
    onOpen: () => void;
}) {
    const avatarUrl = userImage(friend);
    const statusDescription = twoLine
        ? normalizeString(friend.statusDescription)
        : '';

    return (
        <UserHoverCard userId={friend.id} seed={friend}>
            <button
                type="button"
                className={cn(
                    'bg-muted/40 hover:bg-muted focus-visible:ring-ring/50 flex min-w-0 cursor-pointer items-center gap-2 rounded-md pr-3 pl-1 text-sm outline-none focus-visible:ring-3',
                    twoLine ? 'h-10 max-w-64' : 'h-8 max-w-56'
                )}
                onClick={onOpen}
            >
                <Avatar
                    size={twoLine ? 'default' : 'sm'}
                    className="shrink-0 after:hidden"
                >
                    {avatarUrl ? (
                        <AvatarImage src={avatarUrl} alt="" loading="lazy" />
                    ) : null}
                    <AvatarFallback>
                        <UserIcon aria-hidden="true" className="size-3" />
                    </AvatarFallback>
                    <UserStatusDot
                        statusDotClassName={statusDotClassName}
                        className="absolute -right-0.5 -bottom-0.5 z-10 size-3"
                    />
                </Avatar>
                <span className="flex min-w-0 flex-col items-start">
                    <span className="flex max-w-full min-w-0 items-center gap-1 leading-4">
                        <span className="min-w-0 truncate">
                            {friend.displayName}
                        </span>
                        {isFavorite ? (
                            <AffinityBadge isFavorite iconOnly />
                        ) : null}
                    </span>
                    {statusDescription ? (
                        <span className="text-muted-foreground max-w-full truncate text-xs leading-4">
                            {statusDescription}
                        </span>
                    ) : null}
                </span>
            </button>
        </UserHoverCard>
    );
}

export function FriendsLocationsFriendChips({
    friends,
    currentUserId,
    favoriteIds,
    twoLine,
    onOpenUser
}: {
    friends: FriendRecord[];
    currentUserId?: string | null;
    favoriteIds: ReadonlySet<string>;
    twoLine: boolean;
    onOpenUser: (friend: FriendRecord) => void;
}) {
    const currentUserSnapshot = useRuntimeStore(
        (state) => state.auth.currentUserSnapshot
    );
    const isGameRunning = useRuntimeStore(
        (state) => state.gameState.isGameRunning === true
    );

    return (
        <div className="flex min-w-0 flex-wrap items-center gap-1.5">
            {friends.map((friend) => (
                <FriendChip
                    key={friend.id}
                    friend={friend}
                    isFavorite={favoriteIds.has(friend.id)}
                    twoLine={twoLine}
                    statusDotClassName={resolveSidebarStatusDotClassName(
                        friend,
                        currentUserSnapshot,
                        friend.id === currentUserId,
                        { hideNonFriend: false, isGameRunning }
                    )}
                    onOpen={() => onOpenUser(friend)}
                />
            ))}
        </div>
    );
}

function InstanceRow({
    instance,
    currentUserId,
    favoriteIds,
    twoLine,
    onOpenGroup,
    onOpenUser
}: {
    instance: FriendsLocationsWorldInstance;
    currentUserId?: string | null;
    favoriteIds: ReadonlySet<string>;
    twoLine: boolean;
    onOpenGroup: (groupId: string) => void;
    onOpenUser: (friend: FriendRecord) => void;
}) {
    const { t } = useTranslation();
    const parsed = parseLocation(instance.location);
    const metadata = useLocationMetadata({
        locationInfo: parsed,
        currentLocation: instance.location,
        groupHint: instance.groupName
    });
    const groupName = metadata.groupName || instance.groupName;
    const label = [
        translateAccessType(parsed.accessTypeName, t, accessTypeLocaleKeyMap),
        parsed.instanceName ? `#${parsed.instanceName}` : ''
    ]
        .filter(Boolean)
        .join(' · ');

    return (
        <>
            <div className="text-muted-foreground grid min-h-8 min-w-0 grid-cols-[auto_minmax(0,1fr)] content-center gap-y-0.5 text-xs leading-4">
                <RegionCodeBadge region={parsed.region} />
                <span className="col-start-2 flex min-w-0 items-center gap-1.5">
                    <span className="min-w-0 truncate">{label}</span>
                    {instance.isCurrent ? (
                        <CurrentInstanceBadge className="shrink-0" />
                    ) : null}
                </span>
                {groupName ? (
                    <span
                        role="button"
                        tabIndex={0}
                        className="hover:text-primary col-start-2 min-w-0 cursor-pointer truncate"
                        onClick={() => onOpenGroup(instance.groupId)}
                        onKeyDown={(event) => {
                            if (event.key === 'Enter' || event.key === ' ') {
                                event.preventDefault();
                                onOpenGroup(instance.groupId);
                            }
                        }}
                    >
                        ({groupName})
                    </span>
                ) : null}
            </div>
            <FriendsLocationsFriendChips
                friends={instance.friends}
                currentUserId={currentUserId}
                favoriteIds={favoriteIds}
                twoLine={twoLine}
                onOpenUser={onOpenUser}
            />
        </>
    );
}

export function FriendsLocationsWorldSection({
    group,
    summary,
    densityConfig,
    currentUserId,
    favoriteIds,
    onOpenWorld,
    onOpenGroup,
    onOpenUser
}: FriendsLocationsWorldSectionProps) {
    const { t } = useTranslation();
    const name = summary?.name || group.nameHint || group.worldId;
    const thumbnailWidth = densityConfig.worldThumbnailWidth;
    const thumbnailHeight = Math.round((thumbnailWidth * 3) / 4);

    return (
        <section className="flex min-w-0 gap-4">
            <button
                type="button"
                className="bg-muted focus-visible:ring-ring/50 after:ring-foreground/10 relative flex shrink-0 cursor-pointer items-center justify-center self-start overflow-hidden rounded-lg outline-none after:pointer-events-none after:absolute after:inset-0 after:rounded-lg after:ring-1 after:ring-inset focus-visible:ring-3"
                style={{ width: thumbnailWidth, height: thumbnailHeight }}
                aria-label={name}
                onClick={() => onOpenWorld(group, name)}
            >
                {summary?.thumbnailUrl ? (
                    <FadeInImage
                        src={summary.thumbnailUrl}
                        alt=""
                        loading="lazy"
                        className="size-full object-cover"
                        fallback={
                            <GlobeIcon className="text-muted-foreground size-5" />
                        }
                    />
                ) : (
                    <GlobeIcon className="text-muted-foreground size-5" />
                )}
            </button>
            <div className="flex min-w-0 flex-1 flex-col gap-2">
                <div className="flex h-6 min-w-0 items-baseline gap-2.5">
                    <button
                        type="button"
                        className="hover:text-foreground/80 focus-visible:text-foreground/80 min-w-0 cursor-pointer truncate text-left text-sm font-semibold outline-none"
                        onClick={() => onOpenWorld(group, name)}
                    >
                        {name}
                    </button>
                    <span className="text-muted-foreground ml-auto shrink-0 pl-3 text-xs tabular-nums">
                        {t('view.friends_locations.world_friends', {
                            count: group.friendCount
                        })}
                        {group.instances.length > 1
                            ? ` · ${t('view.friends_locations.world_instances', { count: group.instances.length })}`
                            : null}
                    </span>
                </div>
                <div className="grid min-w-0 grid-cols-[fit-content(224px)_minmax(0,1fr)] gap-x-3 gap-y-2">
                    {group.instances.map((instance) => (
                        <InstanceRow
                            key={instance.location}
                            instance={instance}
                            currentUserId={currentUserId}
                            favoriteIds={favoriteIds}
                            twoLine={densityConfig.worldChipLines === 2}
                            onOpenGroup={onOpenGroup}
                            onOpenUser={onOpenUser}
                        />
                    ))}
                </div>
            </div>
        </section>
    );
}
