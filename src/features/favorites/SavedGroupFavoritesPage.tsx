import { PlusIcon, Trash2Icon, UsersIcon } from 'lucide-react';
import { useCallback, useEffect, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { toast } from 'sonner';

import { PageScaffold } from '@/components/layout/PageScaffold';
import { FadeInImage } from '@/components/media/FadeInImage';
import type { GroupProfileRecord } from '@/domain/entities/group';
import {
    commands,
    type SavedGroupFavoritesSnapshot
} from '@/platform/tauri/bindings';
import groupProfileRepository from '@/repositories/groupProfileRepository';
import { openGroupDialog } from '@/services/dialogService';
import { convertFileUrlToImageUrl } from '@/services/entityMediaService';
import { useRuntimeStore } from '@/state/runtimeStore';
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle
} from '@/ui/shadcn/alert-dialog';
import { Button } from '@/ui/shadcn/button';
import { Input } from '@/ui/shadcn/input';

const SAVED_GROUP_FAVORITES_CHANGED_EVENT = 'saved-group-favorites-changed';

export function SavedGroupFavoritesPage() {
    const { t } = useTranslation();
    const currentUserId = useRuntimeStore((state) => state.auth.currentUserId);
    const [snapshot, setSnapshot] = useState<SavedGroupFavoritesSnapshot>({
        collections: []
    });
    const [profiles, setProfiles] = useState<Map<string, GroupProfileRecord>>(
        new Map()
    );
    const [selectedCollectionId, setSelectedCollectionId] = useState('');
    const [newCollectionName, setNewCollectionName] = useState('');
    const [deletingCollectionId, setDeletingCollectionId] = useState('');
    const [busy, setBusy] = useState(false);

    const load = useCallback(async () => {
        if (!currentUserId) {
            setSnapshot({ collections: [] });
            setProfiles(new Map());
            return;
        }
        const next = await commands.appSavedGroupFavoritesGet();
        setSnapshot(next);
        const groupIds = Array.from(
            new Set(
                next.collections.flatMap((collection) => collection.groupIds)
            )
        );
        const results = await Promise.allSettled(
            groupIds.map((groupId) =>
                groupProfileRepository.fetchGroupProfile({
                    groupId,
                    includeRoles: false
                })
            )
        );
        setProfiles(
            new Map(
                results.flatMap((result) =>
                    result.status === 'fulfilled'
                        ? [[result.value.id, result.value] as const]
                        : []
                )
            )
        );
    }, [currentUserId]);

    useEffect(() => {
        const refresh = () => void load().catch(showError);
        refresh();
        window.addEventListener(SAVED_GROUP_FAVORITES_CHANGED_EVENT, refresh);
        return () =>
            window.removeEventListener(
                SAVED_GROUP_FAVORITES_CHANGED_EVENT,
                refresh
            );
    }, [load]);

    useEffect(() => {
        if (
            selectedCollectionId &&
            snapshot.collections.some(
                (collection) => collection.id === selectedCollectionId
            )
        ) {
            return;
        }
        setSelectedCollectionId(snapshot.collections[0]?.id || '');
    }, [selectedCollectionId, snapshot.collections]);

    const selectedCollection = useMemo(
        () =>
            snapshot.collections.find(
                (collection) => collection.id === selectedCollectionId
            ),
        [selectedCollectionId, snapshot.collections]
    );
    const deletingCollection = snapshot.collections.find(
        (collection) => collection.id === deletingCollectionId
    );

    async function mutate(operation: () => Promise<unknown>) {
        setBusy(true);
        try {
            await operation();
            window.dispatchEvent(
                new Event(SAVED_GROUP_FAVORITES_CHANGED_EVENT)
            );
        } catch (error) {
            showError(error);
        } finally {
            setBusy(false);
        }
    }

    function createCollection() {
        const name = newCollectionName.trim();
        if (!name) return;
        void mutate(async () => {
            await commands.appSavedGroupCollectionCreate({ name });
            setNewCollectionName('');
        });
    }

    return (
        <PageScaffold className="flex-1" flushBottom>
            <div className="border-b px-5 py-4">
                <h1 className="text-lg font-semibold">
                    {t('saved_group_favorites.title', {
                        defaultValue: '收藏群组'
                    })}
                </h1>
                <p className="text-muted-foreground mt-1 text-sm">
                    {t('saved_group_favorites.description', {
                        defaultValue:
                            '在本地分组保存常用群组，并用于新实例通知。'
                    })}
                </p>
            </div>
            <div className="grid min-h-0 flex-1 grid-cols-[18rem_minmax(0,1fr)]">
                <aside className="flex min-h-0 flex-col border-r">
                    <div className="flex gap-2 border-b p-3">
                        <Input
                            value={newCollectionName}
                            disabled={busy}
                            placeholder={t(
                                'saved_group_favorites.new_collection',
                                { defaultValue: '新建分组' }
                            )}
                            onChange={(event) =>
                                setNewCollectionName(event.target.value)
                            }
                            onKeyDown={(event) => {
                                if (event.key === 'Enter') createCollection();
                            }}
                        />
                        <Button
                            size="icon"
                            disabled={busy || !newCollectionName.trim()}
                            onClick={createCollection}
                        >
                            <PlusIcon />
                        </Button>
                    </div>
                    <div className="min-h-0 flex-1 space-y-1 overflow-auto p-2">
                        {snapshot.collections.map((collection) => (
                            <div key={collection.id} className="flex gap-1">
                                <Button
                                    type="button"
                                    variant={
                                        collection.id === selectedCollection?.id
                                            ? 'secondary'
                                            : 'ghost'
                                    }
                                    className="min-w-0 flex-1 justify-between"
                                    onClick={() =>
                                        setSelectedCollectionId(collection.id)
                                    }
                                >
                                    <span className="truncate">
                                        {collection.name}
                                    </span>
                                    <span>{collection.groupIds.length}</span>
                                </Button>
                                <Button
                                    size="icon-sm"
                                    variant="ghost"
                                    disabled={busy}
                                    onClick={() =>
                                        setDeletingCollectionId(collection.id)
                                    }
                                >
                                    <Trash2Icon />
                                </Button>
                            </div>
                        ))}
                    </div>
                </aside>
                <main className="min-h-0 overflow-auto p-4">
                    {!selectedCollection ? (
                        <div className="text-muted-foreground rounded-lg border border-dashed p-8 text-center text-sm">
                            {t('saved_group_favorites.empty_collections', {
                                defaultValue: '新建分组后即可收藏群组。'
                            })}
                        </div>
                    ) : selectedCollection.groupIds.length === 0 ? (
                        <div className="text-muted-foreground rounded-lg border border-dashed p-8 text-center text-sm">
                            {t('saved_group_favorites.empty_group')}
                        </div>
                    ) : (
                        <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                            {selectedCollection.groupIds.map((groupId) => {
                                const profile = profiles.get(groupId);
                                return (
                                    <div
                                        key={groupId}
                                        className="bg-card overflow-hidden rounded-lg border"
                                    >
                                        <Button
                                            type="button"
                                            variant="ghost"
                                            className="h-auto w-full justify-start gap-3 rounded-none p-3"
                                            onClick={() =>
                                                openGroupDialog({
                                                    groupId,
                                                    title: profile?.name
                                                })
                                            }
                                        >
                                            <span className="bg-muted flex size-12 shrink-0 items-center justify-center overflow-hidden rounded-md">
                                                {profile?.iconUrl ? (
                                                    <FadeInImage
                                                        src={convertFileUrlToImageUrl(
                                                            profile.iconUrl,
                                                            128
                                                        )}
                                                        alt=""
                                                        className="size-full object-cover"
                                                    />
                                                ) : (
                                                    <UsersIcon className="text-muted-foreground" />
                                                )}
                                            </span>
                                            <span className="min-w-0 flex-1 text-left">
                                                <span className="block truncate font-medium">
                                                    {profile?.name || groupId}
                                                </span>
                                                <span className="text-muted-foreground block truncate text-xs">
                                                    {groupId}
                                                </span>
                                            </span>
                                        </Button>
                                        <div className="border-t p-2 text-right">
                                            <Button
                                                size="sm"
                                                variant="ghost"
                                                disabled={busy}
                                                onClick={() =>
                                                    void mutate(() =>
                                                        commands.appSavedGroupFavoriteRemove(
                                                            { groupId }
                                                        )
                                                    )
                                                }
                                            >
                                                <Trash2Icon />
                                                {t(
                                                    'saved_group_favorites.remove',
                                                    {
                                                        defaultValue: '取消收藏'
                                                    }
                                                )}
                                            </Button>
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </main>
            </div>
            <AlertDialog
                open={Boolean(deletingCollection)}
                onOpenChange={(open) => {
                    if (!open) setDeletingCollectionId('');
                }}
            >
                <AlertDialogContent>
                    <AlertDialogHeader>
                        <AlertDialogTitle>
                            {t('saved_group_favorites.delete_title', {
                                defaultValue: '删除这个收藏分组？'
                            })}
                        </AlertDialogTitle>
                        <AlertDialogDescription>
                            {t('saved_group_favorites.delete_description', {
                                defaultValue:
                                    '删除“{name}”会同时取消其中 {count} 个群组的收藏。',
                                name: deletingCollection?.name,
                                count: deletingCollection?.groupIds.length || 0
                            })}
                        </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                        <AlertDialogCancel>
                            {t('common.actions.cancel')}
                        </AlertDialogCancel>
                        <AlertDialogAction
                            disabled={busy}
                            onClick={() => {
                                if (!deletingCollection) return;
                                void mutate(() =>
                                    commands.appSavedGroupCollectionDelete({
                                        collectionId: deletingCollection.id
                                    })
                                );
                                setDeletingCollectionId('');
                            }}
                        >
                            {t('common.actions.delete')}
                        </AlertDialogAction>
                    </AlertDialogFooter>
                </AlertDialogContent>
            </AlertDialog>
        </PageScaffold>
    );
}

function showError(error: unknown) {
    toast.error(error instanceof Error ? error.message : String(error));
}
