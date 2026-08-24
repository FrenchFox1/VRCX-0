import { useTranslation } from 'react-i18next';

import { HeatmapChart } from '@/components/dialogs/user-dialog/components/UserActivityPanelParts';
import { USER_ACTIVITY_HOUR_LABELS } from '@/components/dialogs/user-dialog/userActivityPanelModel';

import { hourTotals, lateNightShare, peakHour } from '../activityPageModel';
import {
    heatmapScaleColors,
    type ActivityPalette
} from '../useActivityPalette';
import { Exhibit } from './ActivityExhibit';

const HEAT_STEPS = [
    'var(--act-heat-0)',
    'var(--act-heat-1)',
    'var(--act-heat-2)',
    'var(--act-heat-3)',
    'var(--act-heat-4)'
];

function HourStrip({ totals }: { totals: number[] }) {
    const peak = totals.reduce((best, value) => Math.max(best, value), 0);

    return (
        <div>
            <div className="flex h-12 items-end gap-[3px]">
                {totals.map((value, hour) => (
                    <span
                        key={hour}
                        className="flex-1"
                        style={{
                            height: `${peak > 0 ? Math.max((value / peak) * 100, 3) : 3}%`,
                            backgroundColor:
                                value > 0 && peak > 0
                                    ? HEAT_STEPS[
                                          Math.min(
                                              HEAT_STEPS.length - 1,
                                              Math.floor(
                                                  (value / peak) *
                                                      HEAT_STEPS.length
                                              )
                                          )
                                      ]
                                    : 'var(--act-track)'
                        }}
                    />
                ))}
            </div>
            <div className="text-muted-foreground mt-2 flex justify-between text-[11px] tabular-nums">
                <span>00</span>
                <span>06</span>
                <span>12</span>
                <span>18</span>
                <span>23</span>
            </div>
        </div>
    );
}

export function ActivityRhythmExhibit({
    rawBuckets,
    normalizedBuckets,
    displayDayLabels,
    weekStartsOn,
    isDarkMode,
    palette
}: {
    rawBuckets: number[];
    normalizedBuckets: number[];
    displayDayLabels: string[];
    weekStartsOn: number;
    isDarkMode: boolean;
    palette: ActivityPalette | null;
}) {
    const { t } = useTranslation();
    const totals = hourTotals(rawBuckets);
    const peak = peakHour(rawBuckets);
    const lateNight = lateNightShare(rawBuckets);

    if (peak === null) {
        return null;
    }

    return (
        <Exhibit
            label={t('view.activity.section.when_you_play')}
            headline={USER_ACTIVITY_HOUR_LABELS[peak]}
            caption={
                lateNight > 0
                    ? t('view.activity.rhythm.late_night_share', {
                          percent: lateNight
                      })
                    : undefined
            }
            detailLabel={t('view.activity.rhythm.open_heatmap')}
            detail={
                palette ? (
                    <HeatmapChart
                        rawBuckets={rawBuckets}
                        normalizedBuckets={normalizedBuckets}
                        dayLabels={displayDayLabels}
                        hourLabels={USER_ACTIVITY_HOUR_LABELS}
                        weekStartsOn={weekStartsOn}
                        isDarkMode={isDarkMode}
                        emptyColor={palette['act-heat-empty']}
                        scaleColors={heatmapScaleColors(palette)}
                        unitLabel={t('view.activity.unit.minutes')}
                        squareCells
                    />
                ) : null
            }
        >
            <HourStrip totals={totals} />
        </Exhibit>
    );
}
