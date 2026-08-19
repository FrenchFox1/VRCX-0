import type { InstanceCreateGroupAccessType } from '@/platform/tauri/bindings';

import type { buildCreatedInstanceDetails } from './worldInstances';

export type WorldInstanceAccessType =
    | 'public'
    | 'friends'
    | 'friends+'
    | 'invite'
    | 'invite+'
    | 'group';
export type WorldInstanceRegion = 'US West' | 'US East' | 'Europe' | 'Japan';

export interface WorldNewInstanceForm {
    selectedTab: string;
    accessType: WorldInstanceAccessType;
    region: WorldInstanceRegion;
    groupId: string;
    groupName?: string;
    groupAccessType: InstanceCreateGroupAccessType;
    queueEnabled: boolean;
    ageGate: boolean;
    displayName: string;
    displayNamePresets: string[];
    roleIds: string;
    instanceName: string;
    legacyUserId: string;
    strict: boolean;
}

export type CreatedWorldInstance = ReturnType<
    typeof buildCreatedInstanceDetails
>;

export type NewInstanceAfterCreateAction = '' | 'selfInvite' | 'openInGame';

export interface WorldNewInstanceRequest {
    selfInvite: boolean;
    afterCreateAction: NewInstanceAfterCreateAction;
    defaults: Partial<WorldNewInstanceForm>;
}

export interface WorldInstanceInviteRequest {
    location: string;
    launchToken: string;
    worldName: string;
}

export interface InstanceGroupOption {
    displayName?: unknown;
    groupId?: unknown;
    id?: unknown;
    name?: unknown;
}
