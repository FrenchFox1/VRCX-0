import {
    commands,
    type ActivityPageBuildInput,
    type ActivityPageView
} from '@/platform/tauri/bindings';

export type {
    ActivityCompanionOrder,
    ActivityPageAccessSlice,
    ActivityPageBuildInput,
    ActivityPageCompanionRow,
    ActivityPageCoverage,
    ActivityPageFadingRow,
    ActivityPagePeople,
    ActivityPagePreviousSummary,
    ActivityPageSeries,
    ActivityPageSummary,
    ActivityPageView,
    ActivityPageWorldRow,
    ActivityPageWorlds,
    ActivitySeriesBucket,
    ActivitySeriesPoint
} from '@/platform/tauri/bindings';

export const activityPageRepository = {
    view(input: ActivityPageBuildInput): Promise<ActivityPageView> {
        return commands.appActivityPageView(input);
    }
};
