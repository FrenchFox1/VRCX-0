import type { InstanceRosterRow } from '@/domain/instances/instanceRoster';
import { buildLegacyInstanceTag, getLaunchURL } from '@/shared/utils/instance';
import { parseLocation } from '@/shared/utils/location';
import { isRecord } from '@/shared/utils/record';

type DynamicRecord = Record<string, unknown>;

export type WorldInstanceRecord = DynamicRecord & {
    accessType?: string;
    capacity?: number;
    creatorGroup?: DynamicRecord | null;
    creatorGroupId?: string;
    creatorUser?: DynamicRecord | null;
    creatorUserId?: string;
    group?: DynamicRecord | null;
    groupId?: string;
    group_id?: string;
    id?: string;
    instanceId?: string;
    location?: string;
    occupants?: number;
    owner?: DynamicRecord | null;
    ownerId?: string;
    playerCount?: number;
    players?: Array<Record<string, unknown> | string>;
    ref?: WorldInstanceRecord | null;
    secureName?: string;
    shortName?: string | null;
    tag?: string;
    userCount?: number;
    userIds?: string[];
    userList?: Array<Record<string, unknown> | string>;
    users?: InstanceRosterRow[];
    usersById?: Record<string, unknown>;
};

export type CreatedInstanceFallback = DynamicRecord & {
    accessType?: string;
    group?: DynamicRecord | null;
    groupId?: string;
    ownerId?: string | null;
};

type LegacyInstanceForm = {
    accessType?: string;
    ageGate?: boolean;
    groupAccessType?: string;
    groupId?: string;
    groupName?: string;
    instanceName?: string;
    legacyUserId?: string;
    region?: string;
    strict?: boolean;
};

type BuildLegacyCreatedInstanceInput = {
    worldId: string;
    form: LegacyInstanceForm;
    currentUserId: string;
    legacySeed: string;
};

function record(value: unknown): DynamicRecord {
    return isRecord(value) ? value : {};
}

export function normalizeEntityId(value: unknown) {
    return typeof value === 'string'
        ? value.trim()
        : String(value ?? '').trim();
}

export function parseRoleIds(value: unknown) {
    return String(value || '')
        .split(',')
        .map((entry) => entry.trim())
        .filter(Boolean);
}

export function resolveInstanceLocation(worldId: unknown, instance: unknown) {
    const source = record(instance);
    if (typeof source.location === 'string' && source.location.trim()) {
        return source.location.trim();
    }
    const rawId = normalizeEntityId(source.id);
    if (rawId.includes(':')) {
        return rawId;
    }
    const instanceId = normalizeEntityId(source.instanceId || rawId);
    const normalizedWorldId = normalizeEntityId(worldId);
    return normalizedWorldId && instanceId
        ? `${normalizedWorldId}:${instanceId}`
        : '';
}

export function buildLegacyCreatedInstance({
    worldId,
    form,
    currentUserId,
    legacySeed
}: BuildLegacyCreatedInstanceInput) {
    const legacyUserId =
        normalizeEntityId(form.legacyUserId) ||
        normalizeEntityId(currentUserId);
    const instanceName =
        normalizeEntityId(form.instanceName).replace(/[^A-Za-z0-9]/g, '') ||
        legacySeed;
    const accessType = form.accessType || 'public';
    const instanceId = buildLegacyInstanceTag({
        instanceName,
        userId: legacyUserId,
        accessType,
        groupId: form.groupId || '',
        groupAccessType: form.groupAccessType || 'plus',
        region: form.region || 'US West',
        ageGate: Boolean(form.ageGate),
        strict: Boolean(
            form.strict && (accessType === 'invite' || accessType === 'friends')
        )
    });
    const location = `${worldId}:${instanceId}`;
    const parsedLocation = parseLocation(location);
    return {
        location: parsedLocation.tag || location,
        shortName: '',
        secureOrShortName: '',
        url: getLaunchURL(parsedLocation),
        accessType,
        ownerId: parsedLocation.groupId || legacyUserId,
        groupId: parsedLocation.groupId || '',
        group: parsedLocation.groupId
            ? {
                  id: parsedLocation.groupId,
                  groupId: parsedLocation.groupId,
                  name: form.groupName || parsedLocation.groupId
              }
            : null
    };
}

export function buildCreatedInstanceDetails(
    location: string,
    instance: unknown,
    fallback: CreatedInstanceFallback = {}
) {
    const source = record(instance);
    const owner = record(source.owner);
    const group = record(source.group);
    const parsedLocation = parseLocation(location);
    const shortName = normalizeEntityId(
        source.shortName || parsedLocation.shortName
    );
    const secureOrShortName = shortName || normalizeEntityId(source.secureName);
    const launchLocation = parsedLocation.tag || normalizeEntityId(location);
    const groupId =
        normalizeEntityId(source.groupId) ||
        normalizeEntityId(source.group_id) ||
        normalizeEntityId(group.id) ||
        normalizeEntityId(group.groupId) ||
        normalizeEntityId(fallback.groupId) ||
        normalizeEntityId(parsedLocation.groupId);
    return {
        location: launchLocation,
        shortName,
        secureOrShortName,
        accessType:
            normalizeEntityId(source.accessType) ||
            normalizeEntityId(fallback.accessType) ||
            parsedLocation.accessType,
        ownerId:
            normalizeEntityId(source.ownerId) ||
            normalizeEntityId(owner.id) ||
            normalizeEntityId(source.creatorId) ||
            normalizeEntityId(fallback.ownerId) ||
            normalizeEntityId(parsedLocation.userId),
        groupId,
        group:
            source.group ||
            fallback.group ||
            (groupId ? { id: groupId, groupId, name: groupId } : null),
        url: getLaunchURL({
            ...parsedLocation,
            shortName
        })
    };
}
