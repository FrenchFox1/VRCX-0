import { ActivityIcon } from 'lucide-react';
import { useEffect, useMemo, useState } from 'react';
import type { CSSProperties, ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

import { getDisplayDayLabels } from '@/components/dialogs/user-dialog/userActivityPanelModel';
import {
    EmptyState,
    LoadingState,
    PageBody,
    PageScaffold,
    PageToolbar,
    PageToolbarRow
} from '@/components/layout/PageScaffold';
import {
    ToolbarActions,
    ToolbarRefreshButton,
    ToolbarSegmented,
    type ToolbarSegmentOption
} from '@/components/layout/ToolbarControls';
import configRepository from '@/repositories/configRepository';
import { getResolvedThemeMode } from '@/services/themeService';
import { usePreferencesStore } from '@/state/preferencesStore';
import { useRuntimeStore } from '@/state/runtimeStore';
import { useShellStore } from '@/state/shellStore';

import {
    ACTIVITY_PAGE_RANGE_KEY,
    ACTIVITY_RANGE_OPTIONS,
    DEFAULT_ACTIVITY_RANGE,
    hasAnyActivity,
    normalizeActivityRange,
    type ActivityRange
} from './activityPageModel';
import { ActivityAccessSplit } from './components/ActivityAccessSplit';
import { ActivityPeopleExhibit } from './components/ActivityPeopleExhibit';
import { ActivityRhythmExhibit } from './components/ActivityRhythmExhibit';
import { ActivityTimeExhibit } from './components/ActivityTimeExhibit';
import { ActivityWorldsExhibit } from './components/ActivityWorldsExhibit';
import { useActivityHeatmap } from './useActivityHeatmap';
import { useActivityPageResource } from './useActivityPageResource';
import { useActivityPalette } from './useActivityPalette';

function Staggered({
    index,
    children
}: {
    index: number;
    children: ReactNode;
}) {
    return (
        <div
            className="activity-enter mb-3 break-inside-avoid"
            style={{ '--activity-enter-index': index } as CSSProperties}
        >
            {children}
        </div>
    );
}

export function ActivityPageImpl() {
    const { t } = useTranslation();
    const ownerUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const weekStartsOn = usePreferencesStore((state) => state.weekStartsOn);
    const themeMode = useShellStore((state) => state.themeMode);
    const isDarkMode = getResolvedThemeMode(themeMode) === 'dark';
    const [range, setRange] = useState<ActivityRange>(DEFAULT_ACTIVITY_RANGE);
    const [skinElement, setSkinElement] = useState<HTMLDivElement | null>(null);
    const palette = useActivityPalette(skinElement, isDarkMode);

    useEffect(() => {
        let active = true;
        void configRepository
            .getString(ACTIVITY_PAGE_RANGE_KEY, null)
            .then((stored) => {
                if (active) {
                    setRange(normalizeActivityRange(stored));
                }
            });
        return () => {
            active = false;
        };
    }, []);

    const { view, loading, error, refresh } = useActivityPageResource(
        ownerUserId ?? '',
        range
    );
    const heatmap = useActivityHeatmap(ownerUserId ?? '', range);

    const rangeOptions = useMemo<ToolbarSegmentOption<ActivityRange>[]>(
        () =>
            ACTIVITY_RANGE_OPTIONS.map((option) => ({
                value: option,
                label:
                    option === 'all'
                        ? t('view.activity.range.all')
                        : t('view.activity.range.days', { days: option })
            })),
        [t]
    );

    const displayDayLabels = useMemo(
        () =>
            getDisplayDayLabels(
                [
                    t('dialog.user.activity.days.sun'),
                    t('dialog.user.activity.days.mon'),
                    t('dialog.user.activity.days.tue'),
                    t('dialog.user.activity.days.wed'),
                    t('dialog.user.activity.days.thu'),
                    t('dialog.user.activity.days.fri'),
                    t('dialog.user.activity.days.sat')
                ],
                weekStartsOn
            ),
        [t, weekStartsOn]
    );

    function onRangeChange(next: ActivityRange) {
        setRange(next);
        void configRepository.setString(ACTIVITY_PAGE_RANGE_KEY, next);
    }

    return (
        <PageScaffold>
            <PageToolbar>
                <PageToolbarRow>
                    <ToolbarSegmented
                        value={range}
                        onValueChange={onRangeChange}
                        options={rangeOptions}
                    />
                    <ToolbarActions>
                        <ToolbarRefreshButton
                            onRefresh={refresh}
                            loading={loading}
                        />
                    </ToolbarActions>
                </PageToolbarRow>
            </PageToolbar>
            <PageBody className="overflow-y-auto">
                <div className="activity-skin" ref={setSkinElement}>
                    {error ? (
                        <EmptyState
                            icon={ActivityIcon}
                            title={t('view.activity.error.failed_to_load')}
                            description={error}
                        />
                    ) : loading && !view ? (
                        <LoadingState />
                    ) : !hasAnyActivity(view) ? (
                        <EmptyState
                            icon={ActivityIcon}
                            title={t('view.activity.empty.title')}
                            description={t('view.activity.empty.description')}
                        />
                    ) : view ? (
                        <div
                            key={range}
                            className="mx-auto max-w-[120rem] [columns:1] gap-3 pb-4 [column-fill:balance] min-[1600px]:[columns:3] xl:[columns:2]"
                        >
                            <Staggered index={0}>
                                <ActivityTimeExhibit
                                    summary={view.summary}
                                    series={view.series}
                                    isDarkMode={isDarkMode}
                                />
                            </Staggered>
                            <Staggered index={1}>
                                <ActivityRhythmExhibit
                                    rawBuckets={heatmap.rawBuckets}
                                    normalizedBuckets={
                                        heatmap.normalizedBuckets
                                    }
                                    displayDayLabels={displayDayLabels}
                                    weekStartsOn={weekStartsOn}
                                    isDarkMode={isDarkMode}
                                    palette={palette}
                                />
                            </Staggered>
                            <Staggered index={2}>
                                <ActivityWorldsExhibit worlds={view.worlds} />
                            </Staggered>
                            <Staggered index={3}>
                                <ActivityPeopleExhibit people={view.people} />
                            </Staggered>
                            <Staggered index={4}>
                                <section className="activity-card p-6">
                                    <ActivityAccessSplit
                                        slices={view.accessSplit}
                                    />
                                </section>
                            </Staggered>
                            <p className="text-muted-foreground break-inside-avoid px-1 pt-1 text-xs">
                                {t('view.activity.caveat.recorded_since', {
                                    date: view.coverage.firstSourceAt.slice(
                                        0,
                                        10
                                    )
                                })}
                            </p>
                        </div>
                    ) : null}
                </div>
            </PageBody>
        </PageScaffold>
    );
}
