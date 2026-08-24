import { UserIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { FadeInImage } from '@/components/media/FadeInImage';
import { UserHoverCard } from '@/components/user-hover-card/UserHoverCard';
import { cn } from '@/lib/utils';
import type {
    ActivityPageCompanionRow,
    ActivityPagePeople
} from '@/repositories/activityPageRepository';
import { openUserDialog } from '@/services/dialogService';

import { useActivityUserAvatars } from '../useActivityUserAvatars';
import { Exhibit } from './ActivityExhibit';
import { ActivityFadingList } from './ActivityFadingList';

function hours(minutes: number) {
    return (minutes / 60).toFixed(1);
}

function Face({ url, className }: { url: string; className: string }) {
    const fallback = (
        <span className="flex size-full items-center justify-center rounded-full bg-[var(--act-track)]">
            <UserIcon className="text-muted-foreground size-1/2" />
        </span>
    );
    return (
        <span
            className={cn(
                'block shrink-0 overflow-hidden rounded-full',
                className
            )}
        >
            {url ? (
                <FadeInImage
                    src={url}
                    alt=""
                    loading="lazy"
                    decoding="async"
                    className="size-full rounded-full object-cover"
                    fallback={fallback}
                />
            ) : (
                fallback
            )}
        </span>
    );
}

export function ActivityPeopleExhibit({
    people
}: {
    people: ActivityPagePeople;
}) {
    const { t } = useTranslation();
    const avatarOf = useActivityUserAvatars();
    const [lead, ...rest] = people.companions;

    if (!lead) {
        return null;
    }

    const open = (row: ActivityPageCompanionRow) =>
        openUserDialog({ userId: row.userId, title: row.displayName });
    const hoursUnit = t('view.activity.unit.hours');

    return (
        <Exhibit
            label={t('view.activity.section.companions')}
            aside={
                <span className="text-muted-foreground text-xs tabular-nums">
                    {t('view.activity.people.encountered_total', {
                        count: people.encounteredCount
                    })}
                </span>
            }
            footer={<ActivityFadingList rows={people.fading} />}
        >
            <UserHoverCard userId={lead.userId} side="bottom">
                <button
                    type="button"
                    onClick={() => open(lead)}
                    className="flex w-full items-center gap-4 text-left transition-opacity duration-100 ease-out hover:opacity-85 active:opacity-70"
                >
                    <Face url={avatarOf(lead.userId)} className="size-16" />
                    <span className="min-w-0 flex-1">
                        <span className="text-foreground block truncate text-xl font-semibold tracking-[-0.015em]">
                            {lead.displayName || lead.userId}
                        </span>
                        <span className="text-muted-foreground mt-1.5 block text-xs tabular-nums">
                            {t('view.activity.people.lead_caption', {
                                days: lead.coDays,
                                hours: hours(lead.minutes)
                            })}
                        </span>
                    </span>
                </button>
            </UserHoverCard>

            {rest.length > 0 ? (
                <div className="mt-4 flex flex-col border-t border-[var(--act-edge)] pt-2">
                    {rest.map((row) => (
                        <UserHoverCard
                            key={row.userId || row.displayName}
                            userId={row.userId}
                        >
                            <button
                                type="button"
                                onClick={() => open(row)}
                                className="-mx-2 flex items-center gap-3 px-2 py-1.5 text-left transition-colors duration-100 ease-out hover:bg-[var(--act-track)]"
                            >
                                <Face
                                    url={avatarOf(row.userId)}
                                    className="size-7"
                                />
                                <span className="text-foreground min-w-0 flex-1 truncate text-sm">
                                    {row.displayName || row.userId}
                                </span>
                                <span className="text-muted-foreground shrink-0 text-xs tabular-nums">
                                    {t('view.activity.people.co_days', {
                                        count: row.coDays
                                    })}
                                </span>
                                <span className="text-foreground w-16 shrink-0 text-right text-xs tabular-nums">
                                    {hours(row.minutes)}
                                    {hoursUnit}
                                </span>
                            </button>
                        </UserHoverCard>
                    ))}
                </div>
            ) : null}
        </Exhibit>
    );
}
