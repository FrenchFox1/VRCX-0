import { useEffect, useState, type ReactNode } from 'react';

import type { InstanceRosterTimestamp } from '@/domain/instances/instanceRoster';
import { timeToText } from '@/lib/dateTime';
import { useFriendLocationTimeEpoch } from '@/lib/useFriendLocationTimeEpoch';
import { cn } from '@/lib/utils';
import { timestampMsFromValue } from '@/shared/utils/dateTime';
import { useShellStore } from '@/state/shellStore';
import { Spinner } from '@/ui/shadcn/spinner';

const SUB_MINUTE_STEP_MS = 30_000;
const MINUTE_STEP_MS = 60_000;

export function FriendInstanceTimer({
    epoch,
    traveling = false,
    className
}: {
    epoch?: InstanceRosterTimestamp | null;
    traveling?: boolean;
    className?: string;
}) {
    const timeUnitLabels = useShellStore((state) => state.timeUnitLabels);
    const [now, setNow] = useState(() => Date.now());
    const normalizedEpoch = timestampMsFromValue(epoch);
    const elapsedMs = normalizedEpoch ? Math.max(0, now - normalizedEpoch) : 0;
    const isSubMinute = elapsedMs < MINUTE_STEP_MS;
    const stepMs = isSubMinute ? SUB_MINUTE_STEP_MS : MINUTE_STEP_MS;
    const displayedMs = Math.floor(elapsedMs / stepMs) * stepMs;
    const nextStepMs = displayedMs + stepMs;
    const text = normalizedEpoch
        ? timeToText(displayedMs, isSubMinute, timeUnitLabels)
        : '-';

    useEffect(() => {
        if (!normalizedEpoch) {
            return;
        }
        const timeoutId = window.setTimeout(
            () => setNow(Date.now()),
            Math.max(1, nextStepMs - elapsedMs)
        );
        return () => window.clearTimeout(timeoutId);
    }, [elapsedMs, nextStepMs, normalizedEpoch]);

    return (
        <span className="inline-flex min-w-0 items-center">
            {traveling ? <Spinner className="mr-1 size-3 shrink-0" /> : null}
            <span
                className={cn(
                    'truncate tabular-nums',
                    isSubMinute && normalizedEpoch ? 'text-foreground' : null,
                    className
                )}
            >
                {text}
            </span>
        </span>
    );
}

export function FriendLocationTimer({
    userId,
    location,
    traveling = false,
    fallback = null,
    className
}: {
    userId: string;
    location: string;
    traveling?: boolean;
    fallback?: ReactNode;
    className?: string;
}) {
    const epoch = useFriendLocationTimeEpoch(userId, location);
    return epoch ? (
        <FriendInstanceTimer
            epoch={epoch}
            traveling={traveling}
            className={className}
        />
    ) : (
        fallback
    );
}
