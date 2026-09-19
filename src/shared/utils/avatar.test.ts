import { afterEach, describe, expect, it, vi } from 'vitest';

import { compareUnityVersion, getPlatformInfo, parseAvatarUrl } from './avatar';

describe('avatar utils', () => {
    afterEach(() => {
        vi.restoreAllMocks();
    });

    it('parses avatar ids from VRChat avatar URLs', () => {
        expect(parseAvatarUrl('https://vrchat.com/home/avatar/avtr_123')).toBe(
            'avtr_123'
        );
        expect(
            parseAvatarUrl('https://vrchat.com/home/world/wrld_123')
        ).toBeNull();
    });

    it('compares legacy unity sort numbers against SDK unity versions', () => {
        expect(compareUnityVersion('20220306000', '2022.3.6f1')).toBe(true);
        expect(compareUnityVersion('20220307000', '2022.3.6f1')).toBe(false);
        expect(compareUnityVersion('50304010', '5.3.4p1')).toBe(true);
    });

    it('returns false for missing or invalid SDK unity versions', () => {
        const errorSpy = vi
            .spyOn(console, 'error')
            .mockImplementation(() => {});

        expect(compareUnityVersion('20220306000', '')).toBe(false);
        expect(compareUnityVersion('20220306000', '2022.3')).toBe(false);
        expect(errorSpy).toHaveBeenCalledTimes(2);
    });

    it('keeps the best platform package and ignores unsupported variants', () => {
        const pcGood = {
            platform: 'standalonewindows',
            performanceRating: 'Good',
            variant: 'standard'
        };
        const pcNone = {
            platform: 'standalonewindows',
            performanceRating: 'None',
            variant: 'standard'
        };
        const android = {
            platform: 'android',
            performanceRating: 'Medium',
            variant: 'security'
        };
        const iosUnsupported = {
            platform: 'ios',
            performanceRating: 'Good',
            variant: 'impostor'
        };

        expect(
            getPlatformInfo([pcGood, pcNone, android, iosUnsupported])
        ).toEqual({
            pc: pcGood,
            android,
            ios: {}
        });
    });
});
