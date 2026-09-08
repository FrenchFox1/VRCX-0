import { ChevronDownIcon } from 'lucide-react';
import { useEffect, useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { CurrentUserSocialStatusDialog } from '@/components/dialogs/user-dialog/UserSelfEditDialogs';
import { useLocationMetadata } from '@/components/location/useLocationMetadata';
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
import { SidePanelSelfAccountMenu } from '@/components/sidebar/side-panel/SidePanelSelfAccountMenu';
import { useFriendsSidebarDisplayPreferences } from '@/components/sidebar/useFriendsSidebarDisplayPreferences';
import { useFriendsSidebarRuntimeSnapshot } from '@/components/sidebar/useFriendsSidebarRuntimeSnapshot';
import { UserStatusAvatar } from '@/components/UserStatusAvatar';
import { cn } from '@/lib/utils';
import { useModalStore } from '@/state/modalStore';
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
import { Tooltip, TooltipContent, TooltipTrigger } from '@/ui/shadcn/tooltip';

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
    const statusLabel = t(resolveCurrentUserStatusLabelKey(statusValue));

    function commitDescription() {
        setIsEditingDescription(false);
        const nextDescription = descriptionDraft.trim();
        if (nextDescription === statusDescription) {
            return;
        }
        setCurrentUserStatusDescription(nextDescription);
    }

    const openSelf = () => openFriend(selfRow);

    const renderActionItems = (
        slots: typeof CONTEXT_MENU_SLOTS,
        showOpen: boolean
    ) => {
        return (
            <CurrentUserActionItems
                friend={selfRow}
                onOpen={openSelf}
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
        <div className="vrcx-0-side-panel-self ml-2 flex shrink-0 flex-col py-2 pr-1.5 pl-2">
            <ContextMenu>
                <ContextMenuTrigger
                    render={
                        <div className="flex w-full min-w-0 items-center gap-2.5 p-1.5">
                            <button
                                type="button"
                                aria-label={displayName}
                                className="focus-visible:ring-ring shrink-0 cursor-pointer rounded-full outline-none focus-visible:ring-2"
                                onClick={openSelf}
                            >
                                <UserStatusAvatar
                                    className="size-13"
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
                                />
                            </button>
                            <div className="flex min-w-0 flex-1 flex-col gap-0.5">
                                <DropdownMenu>
                                    <DropdownMenuTrigger
                                        render={
                                            <button
                                                type="button"
                                                aria-label={t(
                                                    'side_panel.self_menu'
                                                )}
                                                className="focus-visible:ring-ring text-content-tertiary flex min-w-0 cursor-pointer items-center gap-0.5 rounded-md text-left outline-none focus-visible:ring-2"
                                            />
                                        }
                                    >
                                        <span
                                            style={nameStyle}
                                            className="min-w-0 truncate text-sm leading-5 font-medium"
                                        >
                                            {displayName}
                                        </span>
                                        <ChevronDownIcon className="size-3.5 shrink-0" />
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent
                                        align="start"
                                        className="w-56"
                                    >
                                        {renderActionItems(
                                            DROPDOWN_MENU_SLOTS,
                                            true
                                        )}
                                    </DropdownMenuContent>
                                </DropdownMenu>
                                {isEditingDescription ? (
                                    <input
                                        ref={descriptionInputRef}
                                        type="text"
                                        maxLength={
                                            STATUS_DESCRIPTION_MAX_LENGTH
                                        }
                                        value={descriptionDraft}
                                        aria-label={editDescriptionLabel}
                                        className="ring-ring text-content-primary -mx-1 h-4 min-w-0 rounded-md bg-transparent px-1 text-xs outline-none focus-visible:ring-2"
                                        onChange={(event) =>
                                            setDescriptionDraft(
                                                event.target.value
                                            )
                                        }
                                        onBlur={commitDescription}
                                        onKeyDown={(event) => {
                                            if (event.key === 'Enter') {
                                                event.currentTarget.blur();
                                                return;
                                            }
                                            if (event.key === 'Escape') {
                                                setDescriptionDraft(
                                                    statusDescription
                                                );
                                                setIsEditingDescription(false);
                                            }
                                        }}
                                    />
                                ) : (
                                    <Tooltip>
                                        <TooltipTrigger
                                            render={
                                                <button
                                                    type="button"
                                                    aria-label={
                                                        editDescriptionLabel
                                                    }
                                                    className={cn(
                                                        'focus-visible:ring-ring -mx-1 h-4 min-w-0 cursor-text truncate rounded-md px-1 text-left text-xs leading-4 outline-none focus-visible:ring-2',
                                                        statusDescription
                                                            ? 'text-content-secondary'
                                                            : 'text-content-tertiary'
                                                    )}
                                                    onClick={() => {
                                                        setDescriptionDraft(
                                                            statusDescription
                                                        );
                                                        setIsEditingDescription(
                                                            true
                                                        );
                                                    }}
                                                >
                                                    {statusDescription ||
                                                        editDescriptionLabel}
                                                </button>
                                            }
                                        />
                                        <TooltipContent>
                                            {statusDescription ||
                                                editDescriptionLabel}
                                        </TooltipContent>
                                    </Tooltip>
                                )}
                                <div className="text-content-tertiary flex h-4 min-w-0 items-center text-xs leading-4">
                                    {showLocationSubline ? (
                                        <StaticSidebarLocation
                                            location={displayLocation}
                                            traveling={displayTraveling}
                                            hint={metadataHint}
                                            metadata={locationMetadata}
                                            link
                                            showInstanceIdInLocation={
                                                showInstanceIdInLocation
                                            }
                                            ageGatedInstancesVisible={
                                                ageGatedInstancesVisible
                                            }
                                        />
                                    ) : (
                                        <span className="truncate">
                                            {statusLabel}
                                        </span>
                                    )}
                                </div>
                            </div>
                            <SidePanelSelfAccountMenu />
                        </div>
                    }
                />
                <ContextMenuContent className="w-56">
                    {renderActionItems(CONTEXT_MENU_SLOTS, true)}
                </ContextMenuContent>
            </ContextMenu>
            <CurrentUserSocialStatusDialog controller={socialStatusDialog} />
        </div>
    );
}
