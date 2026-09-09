import { Fragment } from 'react';
import { useTranslation } from 'react-i18next';

import { PageToolbar, PageToolbarRow } from '@/components/layout/PageScaffold';
import {
    ToolbarActions,
    ToolbarSearch,
    ToolbarTabs,
    ToolbarViewMenu,
    ToolbarViews,
    type ToolbarSegmentOption
} from '@/components/layout/ToolbarControls';
import { Field, FieldContent, FieldGroup, FieldLabel } from '@/ui/shadcn/field';
import { Switch } from '@/ui/shadcn/switch';
import {
    ToggleGroup,
    ToggleGroupItem,
    ToggleGroupSeparator
} from '@/ui/shadcn/toggle-group';

import { type FriendsLocationsSegment } from '../friendsLocationsConfig';
import {
    FRIENDS_LOCATIONS_DENSITY_OPTIONS,
    sanitizeFriendsLocationsDensity,
    type FriendsLocationsDensity
} from '../friendsLocationsDensity';

type FriendsLocationsSegmentOption = {
    value: FriendsLocationsSegment;
    labelKey: string;
    count: number;
};

type FriendsLocationsToolbarProps = {
    segmentOptions: FriendsLocationsSegmentOption[];
    searchQuery: string;
    showSameInstanceInOnline: boolean;
    density: FriendsLocationsDensity;
    onSearchQueryChange: (value: string) => void;
    onShowSameInstanceInOnlineChange: (value: boolean) => void;
    onDensityChange: (value: FriendsLocationsDensity) => void;
};

export function FriendsLocationsToolbar({
    segmentOptions,
    searchQuery,
    showSameInstanceInOnline,
    density,
    onSearchQueryChange,
    onShowSameInstanceInOnlineChange,
    onDensityChange
}: FriendsLocationsToolbarProps) {
    const { t } = useTranslation();
    const options: ToolbarSegmentOption<FriendsLocationsSegment>[] =
        segmentOptions.map((segment) => ({
            value: segment.value,
            label: t(segment.labelKey),
            count: segment.count
        }));

    return (
        <PageToolbar>
            <PageToolbarRow>
                <ToolbarViews>
                    <ToolbarTabs options={options} />
                </ToolbarViews>

                <ToolbarSearch
                    value={searchQuery}
                    onValueChange={onSearchQueryChange}
                    placeholder={t('view.friends_locations.search_placeholder')}
                />

                <ToolbarActions>
                    <ToolbarViewMenu contentClassName="p-3">
                        <FieldGroup
                            onClick={(event) => event.stopPropagation()}
                        >
                            <Field orientation="horizontal">
                                <FieldContent>
                                    <FieldLabel htmlFor="friends-locations-same-instance">
                                        {t(
                                            'view.friends_locations.show_same_instance_in_online'
                                        )}
                                    </FieldLabel>
                                </FieldContent>
                                <Switch
                                    id="friends-locations-same-instance"
                                    checked={showSameInstanceInOnline}
                                    onCheckedChange={
                                        onShowSameInstanceInOnlineChange
                                    }
                                />
                            </Field>
                            <Field>
                                <FieldContent>
                                    <FieldLabel>
                                        {t('view.friends_locations.density')}
                                    </FieldLabel>
                                </FieldContent>
                                <ToggleGroup
                                    variant="outline"
                                    size="sm"
                                    value={density ? [density] : []}
                                    onValueChange={(nextValue) => {
                                        if (nextValue[0]) {
                                            onDensityChange(
                                                sanitizeFriendsLocationsDensity(
                                                    nextValue[0]
                                                )
                                            );
                                        }
                                    }}
                                    className="w-full [&>[data-slot=toggle]]:min-w-0 [&>[data-slot=toggle]]:flex-1"
                                >
                                    {FRIENDS_LOCATIONS_DENSITY_OPTIONS.map(
                                        (option, index) => (
                                            <Fragment key={option.value}>
                                                {index > 0 ? (
                                                    <ToggleGroupSeparator />
                                                ) : null}
                                                <ToggleGroupItem
                                                    value={option.value}
                                                    aria-label={t(
                                                        option.labelKey
                                                    )}
                                                    className="w-full min-w-0 justify-center px-2"
                                                >
                                                    <span className="truncate">
                                                        {t(option.labelKey)}
                                                    </span>
                                                </ToggleGroupItem>
                                            </Fragment>
                                        )
                                    )}
                                </ToggleGroup>
                            </Field>
                        </FieldGroup>
                    </ToolbarViewMenu>
                </ToolbarActions>
            </PageToolbarRow>
        </PageToolbar>
    );
}
