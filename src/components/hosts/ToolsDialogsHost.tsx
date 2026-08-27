import { InviteMessageTemplatesDialog } from '@/components/dialogs/InviteMessageDialog';
import { useRuntimeStore } from '@/state/runtimeStore';

import { AppLauncherDialog } from './tools-dialogs/AppLauncherDialog';
import {
    ExportAvatarsListDialog,
    ExportDiscordNamesDialog,
    ExportFriendsListDialog
} from './tools-dialogs/ExportListDialogs';
import { GroupCalendarDialog } from './tools-dialogs/GroupCalendarDialog';
import { LlmEndpointsDialog } from './tools-dialogs/LlmEndpointsDialog';
import { NoteExportDialog } from './tools-dialogs/NoteExportDialog';
import {
    PresenceInviteRequestsDialog,
    PresenceRoomRulesDialog,
    PresenceScheduleDialog
} from './tools-dialogs/presence-automation/PresenceAutomationDialog';
import { ProfileBackupDialog } from './tools-dialogs/ProfileBackupDialog';
import {
    getCurrentUserId,
    getEndpoint
} from './tools-dialogs/toolsDialogUtils';

export function ToolsDialogsHost() {
    const presenceScheduleOpen = useRuntimeStore(
        (state) => state.systemHosts.presenceScheduleOpen
    );
    const appLauncherOpen = useRuntimeStore(
        (state) => state.systemHosts.appLauncherOpen
    );
    const presenceRoomRulesOpen = useRuntimeStore(
        (state) => state.systemHosts.presenceRoomRulesOpen
    );
    const presenceInviteRequestsOpen = useRuntimeStore(
        (state) => state.systemHosts.presenceInviteRequestsOpen
    );
    const groupCalendarOpen = useRuntimeStore(
        (state) => state.systemHosts.groupCalendarOpen
    );
    const exportDiscordNamesOpen = useRuntimeStore(
        (state) => state.systemHosts.exportDiscordNamesOpen
    );
    const noteExportOpen = useRuntimeStore(
        (state) => state.systemHosts.noteExportOpen
    );
    const exportFriendsListOpen = useRuntimeStore(
        (state) => state.systemHosts.exportFriendsListOpen
    );
    const exportAvatarsListOpen = useRuntimeStore(
        (state) => state.systemHosts.exportAvatarsListOpen
    );
    const editInviteMessagesOpen = useRuntimeStore(
        (state) => state.systemHosts.editInviteMessagesOpen
    );
    const llmEndpointsOpen = useRuntimeStore(
        (state) => state.systemHosts.llmEndpointsOpen
    );
    const profileBackupOpen = useRuntimeStore(
        (state) => state.systemHosts.profileBackupOpen
    );
    const setSystemHostOpen = useRuntimeStore(
        (state) => state.setSystemHostOpen
    );

    return (
        <>
            <AppLauncherDialog
                open={appLauncherOpen}
                onOpenChange={(open: boolean) =>
                    setSystemHostOpen('appLauncherOpen', open)
                }
            />
            <PresenceScheduleDialog
                open={presenceScheduleOpen}
                onOpenChange={(open: boolean) =>
                    setSystemHostOpen('presenceScheduleOpen', open)
                }
            />
            <PresenceRoomRulesDialog
                open={presenceRoomRulesOpen}
                onOpenChange={(open: boolean) =>
                    setSystemHostOpen('presenceRoomRulesOpen', open)
                }
            />
            <PresenceInviteRequestsDialog
                open={presenceInviteRequestsOpen}
                onOpenChange={(open: boolean) =>
                    setSystemHostOpen('presenceInviteRequestsOpen', open)
                }
            />
            <GroupCalendarDialog
                open={groupCalendarOpen}
                onOpenChange={(open: boolean) =>
                    setSystemHostOpen('groupCalendarOpen', open)
                }
            />
            <ExportDiscordNamesDialog
                open={exportDiscordNamesOpen}
                onOpenChange={(open: boolean) =>
                    setSystemHostOpen('exportDiscordNamesOpen', open)
                }
            />
            <NoteExportDialog
                open={noteExportOpen}
                onOpenChange={(open: boolean) =>
                    setSystemHostOpen('noteExportOpen', open)
                }
            />
            <ExportFriendsListDialog
                open={exportFriendsListOpen}
                onOpenChange={(open: boolean) =>
                    setSystemHostOpen('exportFriendsListOpen', open)
                }
            />
            <ExportAvatarsListDialog
                open={exportAvatarsListOpen}
                onOpenChange={(open: boolean) =>
                    setSystemHostOpen('exportAvatarsListOpen', open)
                }
            />
            <InviteMessageTemplatesDialog
                open={editInviteMessagesOpen}
                onOpenChange={(open) =>
                    setSystemHostOpen('editInviteMessagesOpen', open)
                }
                currentUserId={getCurrentUserId()}
                endpoint={getEndpoint()}
            />
            <LlmEndpointsDialog
                open={llmEndpointsOpen}
                onOpenChange={(open) =>
                    setSystemHostOpen('llmEndpointsOpen', open)
                }
            />
            <ProfileBackupDialog
                open={profileBackupOpen}
                onOpenChange={(open) =>
                    setSystemHostOpen('profileBackupOpen', open)
                }
            />
        </>
    );
}
