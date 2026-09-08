import { beforeEach, describe, expect, it, vi } from 'vitest';

const mocks = vi.hoisted(() => ({
    openInstanceInGame: vi.fn(),
    openWorldDialog: vi.fn()
}));

vi.mock('@/repositories/vrchatInstanceRepository', () => ({
    default: {
        getInstanceShortName: vi.fn()
    }
}));

vi.mock('@/repositories/vrchatSearchRepository', () => ({
    default: {}
}));

vi.mock('@/services/dialogService', () => ({
    openAvatarDialog: vi.fn(),
    openGroupDialog: vi.fn(),
    openUserDialog: vi.fn(),
    openWorldDialog: mocks.openWorldDialog
}));

vi.mock('@/services/instanceActionService', () => ({
    openInstanceInGame: mocks.openInstanceInGame
}));

import {
    directAccessParse,
    tryOpenLaunchLocation
} from './directAccessService';

const WORLD_ID = 'wrld_12345678-1234-1234-1234-1234567890ab';
const AVATAR_ID = 'avtr_12345678-1234-1234-1234-1234567890ab';
const INSTANCE_ID = '12345~hidden(usr_owner)';
const LOCATION = `${WORLD_ID}:${INSTANCE_ID}`;

describe('directAccessService', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('normalizes launch URLs before trying to open the instance', async () => {
        mocks.openInstanceInGame.mockResolvedValue(true);
        const launchUrl = `https://vrchat.com/home/launch?worldId=${WORLD_ID}&instanceId=${encodeURIComponent(INSTANCE_ID)}&shortName=freshTok`;

        await expect(
            tryOpenLaunchLocation(launchUrl, 'freshTok')
        ).resolves.toBe(true);

        expect(mocks.openInstanceInGame).toHaveBeenCalledWith(
            LOCATION,
            'freshTok'
        );
    });

    it('accepts vrchat launch scheme URLs through direct access', async () => {
        mocks.openInstanceInGame.mockResolvedValue(true);
        const launchUrl = `vrchat://launch?id=${encodeURIComponent(LOCATION)}&shortName=freshTok`;

        await expect(directAccessParse(launchUrl)).resolves.toBe(true);

        expect(mocks.openInstanceInGame).toHaveBeenCalledWith(
            LOCATION,
            'freshTok'
        );
        expect(mocks.openWorldDialog).not.toHaveBeenCalled();
    });
});

describe('directAccessParse detect mode', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('recognises links and prefixed ids without side effects', async () => {
        const cases = [
            `https://vrchat.com/home/world/${WORLD_ID}`,
            `https://open.vrcx-0.dev/world/${WORLD_ID}`,
            `https://open.vrcx-0.dev/avatar/${AVATAR_ID}`,
            `https://vrchat.com/home/launch?worldId=${WORLD_ID}`,
            `https://vrchat.com/home/launch?worldId=${WORLD_ID}&instanceId=x`,
            'https://vrchat.com/home/user/usr_id',
            'https://vrchat.com/home/avatar/avtr_id',
            'https://vrchat.com/home/group/grp_id',
            'https://vrch.at/abcd1234',
            'https://vrc.group/vrcx.1234',
            `vrchat://launch?id=${encodeURIComponent(LOCATION)}`,
            WORLD_ID,
            'usr_12345678-1234-1234-1234-1234567890ab',
            'avtr_12345678-1234-1234-1234-1234567890ab',
            'grp_12345678-1234-1234-1234-1234567890ab',
            'vrcx.1234'
        ];

        for (const value of cases) {
            await expect(directAccessParse(value, 'detect')).resolves.toBe(
                true
            );
        }

        expect(mocks.openInstanceInGame).not.toHaveBeenCalled();
        expect(mocks.openWorldDialog).not.toHaveBeenCalled();
    });

    it('rejects bare tokens that collide with name searches', async () => {
        await expect(directAccessParse('Kagamine', 'detect')).resolves.toBe(
            false
        );
        await expect(directAccessParse('abcd1234', 'detect')).resolves.toBe(
            false
        );
        await expect(directAccessParse('MapleNagis', 'detect')).resolves.toBe(
            false
        );
    });

    it('rejects plain queries and malformed links', async () => {
        for (const value of [
            '',
            '   ',
            'hello world',
            'https://vrchat.com/home',
            'https://open.vrcx-0.dev/world/wrld_invalid',
            `http://open.vrcx-0.dev/world/${WORLD_ID}`,
            `https://open.vrcx-0.dev.example.com/world/${WORLD_ID}`,
            `https://open.vrcx-0.dev//world/${WORLD_ID}`,
            `https://open.vrcx-0.dev/world/${WORLD_ID}/extra`,
            `Open world: https://open.vrcx-0.dev/world/${WORLD_ID}`,
            `https://open.vrcx-0.dev/avatar/${WORLD_ID}`,
            'https://example.com/x'
        ]) {
            await expect(directAccessParse(value, 'detect')).resolves.toBe(
                false
            );
        }
    });

    it('opens VRCX-0 share links in their matching dialogs', async () => {
        const dialogService = await import('@/services/dialogService');

        await expect(
            directAccessParse(`https://open.vrcx-0.dev/world/${WORLD_ID}`)
        ).resolves.toBe(true);
        await expect(
            directAccessParse(`https://open.vrcx-0.dev/avatar/${AVATAR_ID}`)
        ).resolves.toBe(true);

        expect(mocks.openWorldDialog).toHaveBeenCalledWith({
            worldId: WORLD_ID
        });
        expect(dialogService.openAvatarDialog).toHaveBeenCalledWith({
            avatarId: AVATAR_ID
        });
    });

    it('keeps mixed case payloads intact', async () => {
        await expect(
            directAccessParse('https://vrc.group/VRCX.1234', 'detect')
        ).resolves.toBe(true);
        await expect(directAccessParse('VRCX.1234', 'detect')).resolves.toBe(
            true
        );
    });
});
