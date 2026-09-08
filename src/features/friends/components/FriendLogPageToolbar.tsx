import { SearchIcon, XIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import type { AppTable } from '@/components/data-table/appTable';
import { TableColumnVisibilityMenu } from '@/components/data-table/TableColumnVisibilityMenu';
import { DateRangeFilter } from '@/components/date-range-filter/DateRangeFilter';
import { PageToolbar, PageToolbarRow } from '@/components/layout/PageScaffold';
import {
    ToolbarActions,
    ToolbarRefreshButton,
    ToolbarStatus,
    ToolbarViews
} from '@/components/layout/ToolbarControls';
import {
    InputGroup,
    InputGroupInput,
    InputGroupAddon,
    InputGroupButton
} from '@/ui/shadcn/input-group';

import type { FriendLogRow } from '../friendLogRows';
import { FriendLogTypeFilterDropdown } from './FriendLogViewParts';

export function FriendLogPageToolbar({
    selectedTypes,
    onSelectedTypesChange,
    searchDraft,
    onSearchDraftChange,
    onCommitSearch,
    onClearSearch,
    dateFrom,
    dateTo,
    onDateRangeChange,
    detail,
    currentUserId,
    loadStatus,
    onRefresh,
    table
}: {
    selectedTypes: string[];
    onSelectedTypesChange: (value: string[]) => void;
    searchDraft: string;
    onSearchDraftChange(value: string): void;
    onCommitSearch(): void;
    onClearSearch(): void;
    dateFrom: string;
    dateTo: string;
    onDateRangeChange(from: string, to: string): void;
    detail: string;
    currentUserId: string;
    loadStatus: string;
    onRefresh: () => void;
    table: AppTable<FriendLogRow>;
}) {
    const { t } = useTranslation();

    return (
        <PageToolbar>
            <PageToolbarRow>
                <ToolbarViews>
                    <FriendLogTypeFilterDropdown
                        value={selectedTypes}
                        onChange={onSelectedTypesChange}
                    />
                </ToolbarViews>

                <form
                    className="ml-auto flex max-w-96 min-w-0 flex-1"
                    onSubmit={(event) => {
                        event.preventDefault();
                        onCommitSearch();
                    }}
                >
                    <InputGroup className="h-auto min-h-8 flex-wrap">
                        <InputGroupInput
                            value={searchDraft}
                            onChange={(event) =>
                                onSearchDraftChange(event.target.value)
                            }
                            placeholder={t(
                                'view.friend_log.search_placeholder'
                            )}
                            aria-label={t('view.friend_log.search_placeholder')}
                            className="min-w-16"
                        />
                        <InputGroupAddon
                            align="inline-end"
                            className="ml-auto gap-1"
                        >
                            {searchDraft ? (
                                <InputGroupButton
                                    size="icon-xs"
                                    aria-label={t('empty_state.clear_search')}
                                    onClick={onClearSearch}
                                >
                                    <XIcon />
                                </InputGroupButton>
                            ) : null}
                            <DateRangeFilter
                                dateFrom={dateFrom}
                                dateTo={dateTo}
                                onChange={onDateRangeChange}
                                label={t('view.friend_log.date_range')}
                            />
                            <InputGroupButton
                                type="submit"
                                size="icon-xs"
                                aria-label={t(
                                    'view.friend_log.search_placeholder'
                                )}
                            >
                                <SearchIcon />
                            </InputGroupButton>
                        </InputGroupAddon>
                    </InputGroup>
                </form>

                <ToolbarActions>
                    <ToolbarRefreshButton
                        onRefresh={onRefresh}
                        loading={loadStatus === 'running'}
                        disabled={!currentUserId}
                    />
                    <TableColumnVisibilityMenu table={table} />
                </ToolbarActions>
            </PageToolbarRow>

            {detail ? <ToolbarStatus>{detail}</ToolbarStatus> : null}
        </PageToolbar>
    );
}
