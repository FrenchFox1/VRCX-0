import { ImageIcon, StarIcon, UsersIcon } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { FadeInImage } from '@/components/media/FadeInImage';
import { formatDateTime, formatRelativeTime } from '@/lib/dateTime';
import type {
    GroupCalendarEventRecord,
    GroupCalendarGroupRecord
} from '@/repositories/vrchatToolsRepository';
import { convertFileUrlToImageUrl } from '@/services/entityMediaService';

const DATE_TIME_OPTIONS: Intl.DateTimeFormatOptions = {
    month: '2-digit',
    day: '2-digit',
    weekday: 'short',
    hour: '2-digit',
    minute: '2-digit'
};
const TIME_OPTIONS: Intl.DateTimeFormatOptions = {
    hour: '2-digit',
    minute: '2-digit'
};

function formatEventRange(event: GroupCalendarEventRecord) {
    if (!event.startsAt) {
        return '';
    }
    const start = formatDateTime(event.startsAt, DATE_TIME_OPTIONS, {
        fallback: ''
    });
    if (!event.endsAt) {
        return start;
    }
    const sameDay =
        new Date(event.startsAt).toDateString() ===
        new Date(event.endsAt).toDateString();
    const end = formatDateTime(
        event.endsAt,
        sameDay ? TIME_OPTIONS : DATE_TIME_OPTIONS,
        { fallback: '' }
    );
    return end ? `${start} - ${end}` : start;
}

function humanize(value: string | undefined) {
    const text = value?.trim().replaceAll('_', ' ') ?? '';
    return text ? text.charAt(0).toUpperCase() + text.slice(1) : '';
}

function eventStatus(event: GroupCalendarEventRecord, nowMs: number) {
    const startMs = new Date(event.startsAt || '').getTime();
    if (Number.isNaN(startMs)) {
        return null;
    }
    const endMs = new Date(event.endsAt || '').getTime();
    const finishMs = Number.isNaN(endMs) ? startMs : endMs;
    if (nowMs >= finishMs) {
        return 'ended';
    }
    return nowMs >= startMs ? 'live' : 'upcoming';
}

export function GroupEventHoverCardContent({
    event,
    groupName,
    groupProfile,
    isFollowing
}: {
    event: GroupCalendarEventRecord;
    groupName: string;
    groupProfile?: GroupCalendarGroupRecord | null;
    isFollowing: boolean;
}) {
    const { t } = useTranslation();
    const [bannerError, setBannerError] = useState(false);
    const [nowMs] = useState(() => Date.now());
    const bannerUrl = bannerError
        ? ''
        : convertFileUrlToImageUrl(
              event.imageUrl ||
                  event.thumbnailImageUrl ||
                  groupProfile?.bannerUrl ||
                  '',
              512
          );
    const iconUrl = convertFileUrlToImageUrl(groupProfile?.iconUrl || '', 64);
    const title =
        event.title?.trim() ||
        t('dialog.group_calendar.event_card.untitled_event');
    const description = event.description?.trim() ?? '';
    const status = eventStatus(event, nowMs);
    const accessLabel =
        event.accessType === 'public'
            ? t('group_event_hover_card.access.public')
            : event.accessType === 'group'
              ? t('group_event_hover_card.access.group')
              : humanize(event.accessType);
    const meta = [
        accessLabel,
        event.category === 'other' ? '' : humanize(event.category)
    ].filter(Boolean);

    return (
        <div className="flex flex-col">
            <div className="bg-muted flex aspect-[16/9] w-full items-center justify-center overflow-hidden">
                {bannerUrl ? (
                    <FadeInImage
                        src={bannerUrl}
                        alt=""
                        className="size-full object-cover"
                        onError={() => setBannerError(true)}
                    />
                ) : (
                    <ImageIcon className="text-muted-foreground size-6" />
                )}
            </div>
            <div className="flex flex-col gap-1.5 p-3">
                <div className="flex min-w-0 items-center gap-1.5">
                    {iconUrl ? (
                        <FadeInImage
                            src={iconUrl}
                            alt=""
                            className="size-4 shrink-0 rounded-sm object-cover"
                        />
                    ) : null}
                    <span className="text-muted-foreground truncate text-xs">
                        {groupName}
                    </span>
                </div>
                <p className="text-foreground line-clamp-2 text-sm font-medium">
                    {title}
                </p>
                <p className="text-muted-foreground flex flex-wrap items-center gap-x-1.5 text-xs tabular-nums">
                    <span>{formatEventRange(event)}</span>
                    {status === 'live' ? (
                        <span className="inline-flex items-center gap-1 font-medium text-[var(--status-online)]">
                            <span className="size-1.5 rounded-full bg-current" />
                            {t('group_event_hover_card.live')}
                        </span>
                    ) : status === 'upcoming' ? (
                        <span className="text-foreground">
                            {t('group_event_hover_card.starts_in', {
                                time: formatRelativeTime(event.startsAt, {
                                    nowMs
                                })
                            })}
                        </span>
                    ) : status === 'ended' ? (
                        <span>{t('group_event_hover_card.ended')}</span>
                    ) : null}
                </p>
                <div className="text-muted-foreground flex flex-wrap items-center gap-x-3 gap-y-1 text-xs">
                    {meta.length ? <span>{meta.join(' · ')}</span> : null}
                    <span className="inline-flex items-center gap-1 tabular-nums">
                        <UsersIcon className="size-3" aria-hidden="true" />
                        {t('group_event_hover_card.interested', {
                            count: event.interestedUserCount ?? 0
                        })}
                    </span>
                    {isFollowing ? (
                        <span className="inline-flex items-center gap-1 text-[var(--status-askme)]">
                            <StarIcon
                                className="size-3 fill-current"
                                aria-hidden="true"
                            />
                            {t('group_event_hover_card.following')}
                        </span>
                    ) : null}
                </div>
                {description ? (
                    <p className="text-muted-foreground mt-1 line-clamp-4 text-xs leading-relaxed whitespace-pre-line">
                        {description}
                    </p>
                ) : null}
            </div>
        </div>
    );
}
