import {
    closestCenter,
    DndContext,
    KeyboardSensor,
    PointerSensor,
    useSensor,
    useSensors,
    type DragEndEvent
} from '@dnd-kit/core';
import {
    rectSortingStrategy,
    sortableKeyboardCoordinates,
    SortableContext
} from '@dnd-kit/sortable';
import { RefreshCwIcon, UsersRoundIcon } from 'lucide-react';
import { useRef } from 'react';
import { useTranslation } from 'react-i18next';

import { groupIdForRow } from '@/components/dialogs/user-dialog/userDialogGroupRows';
import {
    userDialogGroupSortingOptions,
    type UserDialogGroupSort
} from '@/components/dialogs/user-dialog/userDialogListOptions';
import {
    EmptyState,
    LoadingState,
    PageBody,
    PageHeader,
    PageScaffold,
    PageTitle,
    PageToolbar,
    PageToolbarRow
} from '@/components/layout/PageScaffold';
import { cn } from '@/lib/utils';
import type { GroupMemberVisibility } from '@/platform/tauri/bindings';
import { Button } from '@/ui/shadcn/button';
import { Input } from '@/ui/shadcn/input';
import { ScrollArea } from '@/ui/shadcn/scroll-area';
import {
    Select,
    SelectContent,
    SelectGroup,
    SelectItem,
    SelectTrigger,
    SelectValue
} from '@/ui/shadcn/select';
import { Spinner } from '@/ui/shadcn/spinner';

import { MyGroupCard } from './components/MyGroupCard';
import { MyGroupsSelectionBar } from './components/MyGroupsSelectionBar';
import { useMyGroupsBatchController } from './useMyGroupsBatchController';
import {
    useMyGroupsPageState,
    type MyGroupRow as MyGroupRowModel
} from './useMyGroupsPageState';

export function MyGroupsPage() {
    const { t } = useTranslation();
    const state = useMyGroupsPageState();
    const dragClickSuppressedRef = useRef(false);
    const sensors = useSensors(
        useSensor(PointerSensor, {
            activationConstraint: {
                distance: 6
            }
        }),
        useSensor(KeyboardSensor, {
            coordinateGetter: sortableKeyboardCoordinates
        })
    );
    const batch = useMyGroupsBatchController({
        onCompleted: () => {
            state.clearSelection();
            void state.load(true);
        }
    });

    const selectedGroups = state.visibleGroups.filter((group) =>
        state.selectedIds.has(groupIdForRow(group))
    );
    const isOwnGroup = (group: MyGroupRowModel) =>
        Boolean(group.ownerId) && group.ownerId === state.currentUserId;
    const toBatchTargets = (groups: MyGroupRowModel[]) =>
        groups.map((group) => ({
            groupId: groupIdForRow(group),
            name: group.name || groupIdForRow(group)
        }));
    const leavableSelected = selectedGroups.filter(
        (group) => !isOwnGroup(group)
    );

    function releaseDragClickSuppression() {
        window.setTimeout(() => {
            dragClickSuppressedRef.current = false;
        }, 0);
    }

    function handleDragEnd({ active, over }: DragEndEvent) {
        if (over && active.id !== over.id) {
            void state.moveGroup(String(active.id), String(over.id));
        }
        releaseDragClickSuppression();
    }

    return (
        <PageScaffold className="relative">
            <PageHeader>
                <PageTitle>{t('view.my_groups.title')}</PageTitle>
                <p className="text-muted-foreground text-sm">
                    {t('view.my_groups.description')}
                </p>
            </PageHeader>
            <PageToolbar>
                <PageToolbarRow className="justify-between">
                    <div className="flex min-w-0 flex-wrap items-center gap-2">
                        <Button
                            type="button"
                            size="icon-sm"
                            variant="ghost"
                            aria-label={t('common.actions.refresh')}
                            disabled={state.status === 'running' || batch.busy}
                            onClick={() => void state.load(true)}
                        >
                            {state.status === 'running' ? (
                                <Spinner className="size-4" />
                            ) : (
                                <RefreshCwIcon data-icon="icon" />
                            )}
                        </Button>
                        <span className="text-muted-foreground text-sm tabular-nums">
                            {t('view.my_groups.group_count', {
                                count: state.groups.length
                            })}
                        </span>
                        <Input
                            value={state.search}
                            placeholder={t('dialog.user.action.search_groups')}
                            className="h-8 w-48"
                            disabled={state.editMode}
                            onChange={(event) =>
                                state.setSearch(event.target.value)
                            }
                        />
                        <Select<UserDialogGroupSort>
                            value={state.sort}
                            onValueChange={(value) => {
                                if (value) {
                                    state.setSort(value);
                                }
                            }}
                            items={userDialogGroupSortingOptions.map(
                                (option) => ({
                                    value: option.value,
                                    label: t(option.name)
                                })
                            )}
                        >
                            <SelectTrigger
                                size="sm"
                                className="w-36"
                                disabled={state.editMode}
                            >
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectGroup>
                                    {userDialogGroupSortingOptions.map(
                                        (option) => (
                                            <SelectItem
                                                key={option.value}
                                                value={option.value}
                                                disabled={
                                                    option.value === 'inGame' &&
                                                    !state.orderCapable
                                                }
                                            >
                                                {t(option.name)}
                                            </SelectItem>
                                        )
                                    )}
                                </SelectGroup>
                            </SelectContent>
                        </Select>
                    </div>
                    <Button
                        type="button"
                        size="sm"
                        variant="outline"
                        disabled={batch.busy || state.orderSaving}
                        onClick={
                            state.editMode
                                ? state.exitEditMode
                                : state.enterEditMode
                        }
                    >
                        {state.editMode
                            ? t('view.my_groups.exit_edit_mode')
                            : t('view.my_groups.edit_mode')}
                    </Button>
                </PageToolbarRow>
                {state.editMode ? (
                    <p className="text-muted-foreground px-1.5 text-xs">
                        {!state.orderCapable
                            ? t('view.my_groups.order_unavailable', {
                                  reason:
                                      state.registryPrefs.reason ||
                                      t(
                                          'view.my_groups.order_unavailable_reason_fallback'
                                      )
                              })
                            : state.isGameRunning
                              ? t('view.my_groups.order_game_running')
                              : t('view.my_groups.order_hint')}
                    </p>
                ) : null}
            </PageToolbar>
            <PageBody>
                {state.status === 'running' && !state.groups.length ? (
                    <LoadingState variant="page" />
                ) : state.status === 'error' ? (
                    <EmptyState description={state.error}>
                        <Button
                            type="button"
                            size="sm"
                            variant="outline"
                            onClick={() => void state.load(true)}
                        >
                            {t('common.action.retry')}
                        </Button>
                    </EmptyState>
                ) : state.visibleGroups.length ? (
                    <ScrollArea className="min-h-0 flex-1">
                        <DndContext
                            sensors={sensors}
                            collisionDetection={closestCenter}
                            onDragStart={() => {
                                dragClickSuppressedRef.current = true;
                            }}
                            onDragEnd={handleDragEnd}
                            onDragCancel={releaseDragClickSuppression}
                        >
                            <SortableContext
                                items={state.visibleGroups.map((group) =>
                                    groupIdForRow(group)
                                )}
                                strategy={rectSortingStrategy}
                            >
                                <div
                                    className={cn(
                                        'grid grid-cols-[repeat(auto-fill,minmax(220px,1fr))] gap-2 p-0.5',
                                        selectedGroups.length && 'pb-14'
                                    )}
                                >
                                    {state.visibleGroups.map((group, index) => {
                                        const groupId = groupIdForRow(group);
                                        return (
                                            <MyGroupCard
                                                key={groupId}
                                                group={group}
                                                editMode={state.editMode}
                                                orderEditable={
                                                    state.orderEditable
                                                }
                                                orderIndex={index}
                                                orderBusy={state.orderSaving}
                                                selected={state.selectedIds.has(
                                                    groupId
                                                )}
                                                actionsDisabled={batch.busy}
                                                isOwner={isOwnGroup(group)}
                                                onToggleSelected={(
                                                    targetId
                                                ) => {
                                                    if (
                                                        !dragClickSuppressedRef.current
                                                    ) {
                                                        state.toggleSelected(
                                                            targetId
                                                        );
                                                    }
                                                }}
                                                onSetVisibility={(
                                                    target,
                                                    visibility: GroupMemberVisibility
                                                ) =>
                                                    void batch.setVisibility(
                                                        toBatchTargets([
                                                            target
                                                        ]),
                                                        visibility
                                                    )
                                                }
                                                onLeave={(target) =>
                                                    void batch.leaveGroups(
                                                        toBatchTargets([target])
                                                    )
                                                }
                                            />
                                        );
                                    })}
                                </div>
                            </SortableContext>
                        </DndContext>
                    </ScrollArea>
                ) : (
                    <EmptyState
                        icon={UsersRoundIcon}
                        title={t('view.my_groups.empty_title')}
                        description={
                            state.search
                                ? t('view.my_groups.empty_search')
                                : t('view.my_groups.empty_description')
                        }
                    />
                )}
            </PageBody>
            {state.editMode ? (
                <MyGroupsSelectionBar
                    selectedCount={selectedGroups.length}
                    leavableCount={leavableSelected.length}
                    allSelected={state.allSelected}
                    busy={batch.busy}
                    progress={batch.progress}
                    onSelectAll={state.toggleSelectAll}
                    onClearSelection={state.clearSelection}
                    onSetVisibility={(visibility) =>
                        void batch.setVisibility(
                            toBatchTargets(selectedGroups),
                            visibility
                        )
                    }
                    onLeave={() =>
                        void batch.leaveGroups(toBatchTargets(leavableSelected))
                    }
                />
            ) : null}
        </PageScaffold>
    );
}
