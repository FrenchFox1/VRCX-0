import { EyeIcon, LogOutIcon, XIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import type { GroupMemberVisibility } from '@/platform/tauri/bindings';
import { Button } from '@/ui/shadcn/button';
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger
} from '@/ui/shadcn/dropdown-menu';
import { Spinner } from '@/ui/shadcn/spinner';

const visibilityLabelKeys: Record<GroupMemberVisibility, string> = {
    visible: 'dialog.group.actions.visibility_everyone',
    friends: 'dialog.group.actions.visibility_friends',
    hidden: 'dialog.group.actions.visibility_hidden'
};

const visibilityOptions: GroupMemberVisibility[] = [
    'visible',
    'friends',
    'hidden'
];

export function MyGroupsSelectionBar({
    selectedCount,
    leavableCount,
    allSelected,
    busy,
    progress,
    onSelectAll,
    onClearSelection,
    onSetVisibility,
    onLeave
}: {
    selectedCount: number;
    leavableCount: number;
    allSelected: boolean;
    busy: boolean;
    progress: { current: number; total: number } | null;
    onSelectAll(): void;
    onClearSelection(): void;
    onSetVisibility(visibility: GroupMemberVisibility): void;
    onLeave(): void;
}) {
    const { t } = useTranslation();

    if (selectedCount === 0) {
        return null;
    }

    return (
        <div className="pointer-events-none absolute inset-x-0 bottom-3 z-20 flex justify-center px-2">
            <div className="bg-popover text-popover-foreground pointer-events-auto flex max-w-full flex-wrap items-center gap-1.5 rounded-full border px-3 py-1.5 text-sm shadow-lg">
                <span className="text-muted-foreground px-1.5 font-medium whitespace-nowrap tabular-nums">
                    {busy && progress
                        ? t('view.my_groups.batch_progress', {
                              current: progress.current,
                              total: progress.total
                          })
                        : t('view.my_groups.selected_count', {
                              count: selectedCount
                          })}
                </span>
                {busy ? (
                    <Spinner className="size-4" />
                ) : (
                    <>
                        <Button
                            type="button"
                            size="sm"
                            variant="ghost"
                            onClick={onSelectAll}
                        >
                            {allSelected
                                ? t('view.tools.gallery_selection.deselect_all')
                                : t('view.tools.gallery_selection.select_all')}
                        </Button>
                        <DropdownMenu>
                            <DropdownMenuTrigger
                                render={
                                    <Button
                                        type="button"
                                        size="sm"
                                        variant="ghost"
                                    >
                                        <EyeIcon data-icon="inline-start" />
                                        {t('dialog.group.actions.visibility')}
                                    </Button>
                                }
                            />
                            <DropdownMenuContent side="top" align="center">
                                {visibilityOptions.map((option) => (
                                    <DropdownMenuItem
                                        key={option}
                                        onClick={() => onSetVisibility(option)}
                                    >
                                        {t(visibilityLabelKeys[option])}
                                    </DropdownMenuItem>
                                ))}
                            </DropdownMenuContent>
                        </DropdownMenu>
                        <Button
                            type="button"
                            size="sm"
                            variant="ghost"
                            disabled={leavableCount === 0}
                            title={
                                leavableCount === 0
                                    ? t('view.my_groups.leave_owner_locked')
                                    : undefined
                            }
                            onClick={onLeave}
                        >
                            <LogOutIcon data-icon="inline-start" />
                            {leavableCount < selectedCount
                                ? t('view.my_groups.leave_partial', {
                                      count: leavableCount
                                  })
                                : t('view.my_groups.leave')}
                        </Button>
                        <Button
                            type="button"
                            size="icon-xs"
                            variant="ghost"
                            className="rounded-full"
                            aria-label={t('common.actions.clear')}
                            onClick={onClearSelection}
                        >
                            <XIcon data-icon="icon" />
                        </Button>
                    </>
                )}
            </div>
        </div>
    );
}
