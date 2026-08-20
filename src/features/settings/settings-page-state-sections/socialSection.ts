import type { BuildSettingsPageStateSectionsInput } from '../settingsPageStateSections';

export function buildSocialSection({
    prefs,
    selectedFavoriteFriendGroupLabel,
    favoriteFriendGroupOptions,
    remoteFavoriteFriendGroupOptions,
    localFavoriteFriendGroupOptions,
    localFavoriteFriendsGroups,
    commit,
    addFeedHiddenUser,
    removeFeedHiddenUser,
    setRecentActionCooldownEnabledPreference,
    setRecentActionCooldownMinutesPreference,
    toggleLocalFavoriteFriendsGroup,
    setPrefs,
    saveBoolPreference,
    savePreferenceValue,
    normalizeRecentActionCooldownMinutes
}: BuildSettingsPageStateSectionsInput) {
    return {
        selectedFavoriteFriendGroupLabel,
        favoriteFriendGroupOptions,
        remoteFavoriteFriendGroupOptions,
        localFavoriteFriendGroupOptions,
        localFavoriteFriendsGroups,
        feedHiddenUsers: prefs.feedHiddenUsers,
        commit,
        onAddFeedHiddenUser: addFeedHiddenUser,
        onRemoveFeedHiddenUser: removeFeedHiddenUser,
        setRecentActionCooldownEnabledPreference,
        setRecentActionCooldownMinutesPreference,
        toggleLocalFavoriteFriendsGroup,
        setPrefs,
        onHideUnfriendsChange: (checked: boolean) => {
            saveBoolPreference('hideUnfriends', 'hideUnfriends', checked);
        },
        onRecentActionCooldownEnabledChange: (checked: boolean) => {
            savePreferenceValue('recentActionCooldownEnabled', checked, () =>
                setRecentActionCooldownEnabledPreference(checked)
            );
        },
        onRecentActionCooldownMinutesChange: (value: string) => {
            setPrefs((current) => ({
                ...current,
                recentActionCooldownMinutes: value
            }));
        },
        onRecentActionCooldownMinutesBlur: (value: string) => {
            const nextValue = normalizeRecentActionCooldownMinutes(value);
            savePreferenceValue('recentActionCooldownMinutes', nextValue, () =>
                setRecentActionCooldownMinutesPreference(nextValue)
            );
        },
        onToggleLocalFavoriteFriendsGroup: (
            groupKey: string,
            checked: boolean
        ) => {
            toggleLocalFavoriteFriendsGroup(groupKey, checked);
        }
    };
}
