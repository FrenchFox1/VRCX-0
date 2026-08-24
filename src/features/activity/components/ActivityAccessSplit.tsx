import { useTranslation } from 'react-i18next';

import type { ActivityPageAccessSlice } from '@/repositories/activityPageRepository';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/ui/shadcn/tooltip';

import {
    accessBucketLabelKey,
    accessShare,
    visibleAccessSlices
} from '../activityPageModel';

const ACCESS_TINTS: Record<string, string> = {
    public: 'var(--act-heat-4)',
    friendsPlus: 'var(--act-heat-3)',
    friends: 'var(--act-heat-2)',
    group: 'var(--act-heat-1)',
    invitePlus: 'var(--act-heat-0)',
    invite: 'var(--act-heat-min)',
    unknown: 'var(--act-track)'
};

function accessTint(access: string) {
    return ACCESS_TINTS[access] ?? ACCESS_TINTS.unknown;
}

export function ActivityAccessSplit({
    slices
}: {
    slices: ActivityPageAccessSlice[];
}) {
    const { t } = useTranslation();
    const totalMinutes = slices.reduce((sum, slice) => sum + slice.minutes, 0);
    const visible = visibleAccessSlices(slices);

    if (totalMinutes <= 0 || visible.length === 0) {
        return null;
    }

    const describe = (slice: ActivityPageAccessSlice) =>
        `${t(accessBucketLabelKey(slice.access))} · ${accessShare(slice.minutes, totalMinutes)}% · ${(slice.minutes / 60).toFixed(1)}${t('view.activity.unit.hours')}`;

    return (
        <div className="flex flex-col gap-2.5">
            <span className="text-muted-foreground text-xs">
                {t('view.activity.section.instance_types')}
            </span>
            <div className="flex h-2 w-full gap-0.5 overflow-hidden">
                {visible.map((slice) => (
                    <Tooltip key={slice.access}>
                        <TooltipTrigger
                            render={
                                <span
                                    style={{
                                        width: `${accessShare(slice.minutes, totalMinutes)}%`,
                                        backgroundColor: accessTint(
                                            slice.access
                                        )
                                    }}
                                />
                            }
                        />
                        <TooltipContent>{describe(slice)}</TooltipContent>
                    </Tooltip>
                ))}
            </div>
            <div className="flex flex-wrap gap-x-4 gap-y-1.5">
                {visible.map((slice) => (
                    <Tooltip key={slice.access}>
                        <TooltipTrigger
                            render={
                                <span className="flex items-center gap-1.5 text-xs">
                                    <span
                                        className="size-2.5"
                                        style={{
                                            backgroundColor: accessTint(
                                                slice.access
                                            )
                                        }}
                                    />
                                    <span className="text-foreground font-medium">
                                        {t(accessBucketLabelKey(slice.access))}
                                    </span>
                                    <span className="text-muted-foreground tabular-nums">
                                        {accessShare(
                                            slice.minutes,
                                            totalMinutes
                                        )}
                                        %
                                    </span>
                                </span>
                            }
                        />
                        <TooltipContent>{describe(slice)}</TooltipContent>
                    </Tooltip>
                ))}
            </div>
        </div>
    );
}
