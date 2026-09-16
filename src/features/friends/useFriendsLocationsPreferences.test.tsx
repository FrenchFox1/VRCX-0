// @vitest-environment jsdom

import { act, renderHook, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    boolValues: new Map<string, boolean>(),
    stringValues: new Map<string, string>(),
    getBool: vi.fn(),
    getString: vi.fn(),
    setBool: vi.fn(),
    setString: vi.fn()
}));

vi.mock('@/repositories/configRepository', () => ({
    default: {
        getBool: mocks.getBool,
        getString: mocks.getString,
        setBool: mocks.setBool,
        setString: mocks.setString
    }
}));

import { publishPreferenceChanged } from '@/shared/events/preferenceEvents';

import { useFriendsLocationsPreferences } from './useFriendsLocationsPreferences';

describe('useFriendsLocationsPreferences', () => {
    beforeEach(() => {
        mocks.boolValues.clear();
        mocks.stringValues.clear();
        mocks.getBool
            .mockReset()
            .mockImplementation(
                async (key: string, fallback = false) =>
                    mocks.boolValues.get(key) ?? fallback
            );
        mocks.getString
            .mockReset()
            .mockImplementation(
                async (key: string, fallback = '') =>
                    mocks.stringValues.get(key) ?? String(fallback)
            );
        mocks.setBool.mockReset().mockResolvedValue(undefined);
        mocks.setString.mockReset().mockResolvedValue(undefined);
    });

    it('loads persisted preferences and writes changes back', async () => {
        mocks.stringValues.set('FriendLocationDensity', 'dense');
        mocks.boolValues.set('FriendLocationShowSameInstance', true);
        mocks.stringValues.set('sidebarFavoriteGroups', '["group_a"]');
        mocks.stringValues.set('sidebarSortMethod3', 'Sort by Time');
        const { result } = renderHook(() => useFriendsLocationsPreferences());

        await waitFor(() => expect(result.current.preferencesReady).toBe(true));
        expect(result.current.density).toBe('dense');
        expect(result.current.showSameInstanceInOnline).toBe(true);
        expect(result.current.sidebarFavoritePrefs.selectedGroups).toEqual([
            'group_a'
        ]);
        expect(result.current.sidebarSortMethods).toEqual([
            'Sort by Status',
            'Sort Alphabetically',
            'Sort by Time'
        ]);

        act(() => {
            result.current.changeShowSameInstanceInOnline(false);
        });

        expect(result.current.showSameInstanceInOnline).toBe(false);
        expect(mocks.setBool).toHaveBeenCalledWith(
            'FriendLocationShowSameInstance',
            false
        );
    });

    it('reloads sidebar preferences when they change elsewhere', async () => {
        const { result } = renderHook(() => useFriendsLocationsPreferences());
        await waitFor(() => expect(result.current.preferencesReady).toBe(true));
        expect(result.current.sidebarFavoritePrefs.isDivideByGroup).toBe(false);

        mocks.boolValues.set('isSidebarDivideByFriendGroup', true);
        act(() => {
            publishPreferenceChanged('isSidebarDivideByFriendGroup', true);
        });

        await waitFor(() =>
            expect(result.current.sidebarFavoritePrefs.isDivideByGroup).toBe(
                true
            )
        );
    });
});
