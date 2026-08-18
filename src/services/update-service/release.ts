import { branches } from '@/shared/constants/settings';
import {
    compareReleaseVersions,
    parseReleaseVersion
} from '@/shared/utils/releaseVersion';

import type { GitHubRelease, NormalizedRelease } from './types';

function isRecord(value: unknown): value is Record<string, unknown> {
    return Boolean(value && typeof value === 'object');
}

function asGitHubRelease(value: unknown): GitHubRelease {
    return isRecord(value) ? value : {};
}

export function normalizeGitHubRelease(
    release: GitHubRelease
): NormalizedRelease | null {
    const parsedVersion = parseReleaseVersion(String(release?.tag_name || ''));
    if (!parsedVersion) {
        return null;
    }

    return {
        canonicalVersion: parsedVersion.canonicalVersion,
        channel: 'Stable',
        displayVersion: parsedVersion.displayVersion,
        htmlUrl: release.html_url || '',
        tagName: release.tag_name || '',
        displayName: release.name || `VRCX-0 ${parsedVersion.displayVersion}`,
        prerelease: Boolean(release.prerelease),
        publishedAt: release.published_at || '',
        body: release.body || '',
        updaterType: 'manual'
    };
}

export function normalizeReleaseList(
    branch: unknown,
    releases: unknown
): NormalizedRelease[] {
    const normalizedBranch = sanitizeBranch(branch);
    return (Array.isArray(releases) ? releases : [releases])
        .map((release) => normalizeGitHubRelease(asGitHubRelease(release)))
        .filter(
            (release): release is NormalizedRelease =>
                release !== null &&
                release.channel === normalizedBranch &&
                release.prerelease === false
        )
        .sort((left, right) =>
            compareReleaseVersions(
                right.canonicalVersion,
                left.canonicalVersion
            )
        );
}

export function sanitizeBranch(_branch?: unknown): keyof typeof branches {
    return 'Stable';
}
