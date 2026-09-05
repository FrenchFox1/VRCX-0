import { ChevronDownIcon } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { CurrentUserSocialStatusDialog } from '@/components/dialogs/user-dialog/UserSelfEditDialogs';
import { useLocationMetadata } from '@/components/location/useLocationMetadata';
import { AccountSwitcherPopover } from '@/components/sidebar/friends-sidebar/AccountSwitcherPopover';
import {
    CurrentUserActionItems,
    resolveCurrentUserStatusLabelKey
} from '@/components/sidebar/friends-sidebar/FriendsSidebarActionItems';
import { resolveFriendRowDisplay } from '@/components/sidebar/friends-sidebar/FriendsSidebarFriendRow';
import {
    resolveFriendRowLocationState,
    StaticSidebarLocation
} from '@/components/sidebar/friends-sidebar/FriendsSidebarLocation';
import { resolveSidebarStatusDotClassName } from '@/components/sidebar/friends-sidebar/friendsSidebarModel';
import { buildCurrentUserDisplayRecord } from '@/components/sidebar/friends-sidebar/friendsSidebarVirtualRowBuilder';
import { useFriendsSidebarActions } from '@/components/sidebar/friends-sidebar/useFriendsSidebarActions';
import { useFriendsSidebarPreferences } from '@/components/sidebar/friends-sidebar/useFriendsSidebarPreferences';
import { useFriendsSidebarDisplayPreferences } from '@/components/sidebar/useFriendsSidebarDisplayPreferences';
import { useFriendsSidebarRuntimeSnapshot } from '@/components/sidebar/useFriendsSidebarRuntimeSnapshot';
import { UserDetailContent } from '@/components/UserDetailTile';
import { cn } from '@/lib/utils';
import { userStatusIndicatorClassName } from '@/shared/utils/userStatus';
import { useModalStore } from '@/state/modalStore';
import { Button } from '@/ui/shadcn/button';
import {
    ContextMenu,
    ContextMenuCheckboxItem,
    ContextMenuContent,
    ContextMenuGroup,
    ContextMenuItem,
    ContextMenuSeparator,
    ContextMenuSub,
    ContextMenuSubContent,
    ContextMenuSubTrigger,
    ContextMenuTrigger
} from '@/ui/shadcn/context-menu';
import {
    DropdownMenu,
    DropdownMenuCheckboxItem,
    DropdownMenuContent,
    DropdownMenuGroup,
    DropdownMenuItem,
    DropdownMenuSeparator,
    DropdownMenuSub,
    DropdownMenuSubContent,
    DropdownMenuSubTrigger,
    DropdownMenuTrigger
} from '@/ui/shadcn/dropdown-menu';

const STATUS_DESCRIPTION_MAX_LENGTH = 32;

const CONTEXT_MENU_SLOTS = {
    MenuItem: ContextMenuItem,
    CheckboxItem: ContextMenuCheckboxItem,
    Group: ContextMenuGroup,
    Separator: ContextMenuSeparator,
    Sub: ContextMenuSub,
    SubTrigger: ContextMenuSubTrigger,
    SubContent: ContextMenuSubContent
};

const DROPDOWN_MENU_SLOTS = {
    MenuItem: DropdownMenuItem,
    CheckboxItem: DropdownMenuCheckboxItem,
    Group: DropdownMenuGroup,
    Separator: DropdownMenuSeparator,
    Sub: DropdownMenuSub,
    SubTrigger: DropdownMenuSubTrigger,
    SubContent: DropdownMenuSubContent
};

export function SidePanelSelfHeader() {
    const { t } = useTranslation();
    const [isEditingDescription, setIsEditingDescription] = useState(false);
    const [descriptionDraft, setDescriptionDraft] = useState('');
    const descriptionInputRef = useRef<HTMLInputElement | null>(null);
    const {
        currentEndpoint,
        currentUser,
        currentUserId,
        gameState,
        isDarkMode
    } = useFriendsSidebarRuntimeSnapshot();
    const {
        ageGatedInstancesVisible,
        randomUserColours,
        showInstanceIdInLocation,
        trustColor
    } = useFriendsSidebarDisplayPreferences();
    const { statusPresets } = useFriendsSidebarPreferences();
    const confirm = useModalStore((state) => state.confirm);
    const {
        applyCurrentUserStatusPreset,
        changeCurrentUserStatus,
        editCurrentUserSocialStatus,
        openFriend,
        setCurrentUserStatusDescription,
        socialStatusDialog
    } = useFriendsSidebarActions({
        confirm,
        currentUser,
        currentUserId
    });

    const selfRow = useMemo(
        () => buildCurrentUserDisplayRecord(currentUser, gameState),
        [currentUser, gameState]
    );
    const { displaySource, imageUrl, displayName, nameStyle } =
        resolveFriendRowDisplay(selfRow, {
            randomUserColours,
            isDarkMode,
            trustColor
        });
    const locationMetadata = useLocationMetadata({
        locationInfo: displaySource?.location || '',
        currentLocation: gameState?.currentLocation || '',
        endpoint: currentEndpoint || ''
    });

    useEffect(() => {
        if (!isEditingDescription) {
            return;
        }
        const input = descriptionInputRef.current;
        input?.focus();
        input?.select();
    }, [isEditingDescription]);

    if (!selfRow) {
        return (
            <CurrentUserSocialStatusDialog controller={socialStatusDialog} />
        );
    }

    const statusValue = String(displaySource?.status || '');
    const statusDescription = String(displaySource?.statusDescription || '');
    const {
        displayLocation,
        displayTraveling,
        metadataHint,
        showLocationSubline
    } = resolveFriendRowLocationState({
        friend: selfRow,
        isCurrentUser: true,
        isGroupByInstance: false,
        locationTime: null
    });
    const editDescriptionLabel = t(
        'component.friends_sidebar.modal.edit_status_description'
    );

    function commitDescription() {
        setIsEditingDescription(false);
        const nextDescription = descriptionDraft.trim();
        if (nextDescription === statusDescription) {
            return;
        }
        setCurrentUserStatusDescription(nextDescription);
    }

    const renderActionItems = (
        slots: typeof CONTEXT_MENU_SLOTS,
        showOpen: boolean
    ) => {
        return (
            <CurrentUserActionItems
                friend={selfRow}
                onOpen={() => openFriend(selfRow)}
                onChangeStatus={changeCurrentUserStatus}
                onSetStatusDescription={setCurrentUserStatusDescription}
                onEditSocialStatus={editCurrentUserSocialStatus}
                onApplyStatusPreset={applyCurrentUserStatusPreset}
                statusPresets={statusPresets}
                showOpen={showOpen}
                {...slots}
            />
        );
    };

    return (
        <div className="vrcx-0-side-panel-self -ml-2 flex shrink-0 flex-col gap-1.5 py-2 pr-1.5 pl-2">
            <ContextMenu>
                <ContextMenuTrigger
                    render={
                        <div className="flex w-full min-w-0 items-center gap-0.5">
                            <button
                                type="button"
                                className="focus-visible:ring-ring flex h-auto w-full min-w-0 flex-1 cursor-pointer items-center justify-start gap-2 rounded-md p-1.5 text-left font-normal outline-none focus-visible:ring-2"
                                onClick={() => openFriend(selfRow)}
                            >
                                <UserDetailContent
                                    imageUrl={imageUrl}
                                    statusDotClassName={resolveSidebarStatusDotClassName(
                                        selfRow,
                                        currentUser,
                                        true,
                                        {
                                            isGameRunning:
                                                gameState?.isGameRunning
                                        }
                                    )}
                                    displayName={displayName}
                                    nameStyle={nameStyle}
                                    subline={
                                        showLocationSubline ? (
                                            <StaticSidebarLocation
                                                location={displayLocation}
                                                traveling={displayTraveling}
                                                hint={metadataHint}
                                                metadata={locationMetadata}
                                                tooltips={false}
                                                showInstanceIdInLocation={
                                                    showInstanceIdInLocation
                                                }
                                                ageGatedInstancesVisible={
                                                    ageGatedInstancesVisible
                                                }
                                            />
                                        ) : null
                                    }
                                />
                            </button>
                            <AccountSwitcherPopover />
                        </div>
                    }
                />
                <ContextMenuContent className="w-56">
                    {renderActionItems(CONTEXT_MENU_SLOTS, true)}
                </ContextMenuContent>
            </ContextMenu>
            <div className="flex min-w-0 items-center gap-1.5 pl-1.5">
                <DropdownMenu>
                    <DropdownMenuTrigger
                        render={
                            <Button
                                type="button"
                                variant="ghost"
                                className="h-6 shrink-0 gap-1 rounded-full px-2 text-xs font-normal"
                            />
                        }
                    >
                        <i
                            aria-hidden="true"
                            className={userStatusIndicatorClassName(
                                statusValue
                            )}
                        />
                        {t(resolveCurrentUserStatusLabelKey(statusValue))}
                        <ChevronDownIcon data-icon="inline-end" />
                    </DropdownMenuTrigger>
                    <DropdownMenuContent align="start" className="w-56">
                        {renderActionItems(DROPDOWN_MENU_SLOTS, false)}
                    </DropdownMenuContent>
                </DropdownMenu>
                {isEditingDescription ? (
                    <input
                        ref={descriptionInputRef}
                        type="text"
                        maxLength={STATUS_DESCRIPTION_MAX_LENGTH}
                        value={descriptionDraft}
                        aria-label={editDescriptionLabel}
                        className="ring-ring text-content-primary h-6 min-w-0 flex-1 rounded-md bg-transparent px-2 text-xs outline-none focus-visible:ring-2"
                        onChange={(event) =>
                            setDescriptionDraft(event.target.value)
                        }
                        onBlur={commitDescription}
                        onKeyDown={(event) => {
                            if (event.key === 'Enter') {
                                event.currentTarget.blur();
                                return;
                            }
                            if (event.key === 'Escape') {
                                setDescriptionDraft(statusDescription);
                                setIsEditingDescription(false);
                            }
                        }}
                    />
                ) : (
                    <button
                        type="button"
                        aria-label={editDescriptionLabel}
                        title={statusDescription || editDescriptionLabel}
                        className={cn(
                            'focus-visible:ring-ring h-6 min-w-0 flex-1 cursor-text truncate rounded-md px-2 text-left text-xs outline-none focus-visible:ring-2',
                            statusDescription
                                ? 'text-content-secondary'
                                : 'text-content-tertiary'
                        )}
                        onClick={() => {
                            setDescriptionDraft(statusDescription);
                            setIsEditingDescription(true);
                        }}
                    >
                        {statusDescription || editDescriptionLabel}
                    </button>
                )}
            </div>
            <CurrentUserSocialStatusDialog controller={socialStatusDialog} />
        </div>
    );
}
