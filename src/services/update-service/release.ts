import type { ReleaseBranchKey } from '@/shared/constants/settings';
import { isRecord } from '@/shared/utils/record';
import {
    compareReleaseVersions,
    parseReleaseVersion
} from '@/shared/utils/releaseVersion';

import type { GitHubRelease, NormalizedRelease } from './types';

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
    branch: ReleaseBranchKey,
    releases: unknown
): NormalizedRelease[] {
    return (Array.isArray(releases) ? releases : [releases])
        .map((release) => normalizeGitHubRelease(asGitHubRelease(release)))
        .filter(
            (release): release is NormalizedRelease =>
                release !== null &&
                release.channel === branch &&
                release.prerelease === false
        )
        .sort((left, right) =>
            compareReleaseVersions(
                right.canonicalVersion,
                left.canonicalVersion
            )
        );
}

export function sanitizeBranch(_branch?: ReleaseBranchKey): ReleaseBranchKey {
    return 'Stable';
}
