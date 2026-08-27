import { isAvatarId, isWorldId } from './vrchatIds';

export const VRCX_OPEN_RELAY_ORIGIN = 'https://open.vrcx-0.dev';

function entityRelayLink(entity: 'avatar' | 'world', entityId: string): string {
    return `${VRCX_OPEN_RELAY_ORIGIN}/${entity}/${entityId.trim()}`;
}

export function vrcxWorldDeepLink(worldId: string): string {
    return isWorldId(worldId) ? entityRelayLink('world', worldId) : '';
}

export function vrcxAvatarDeepLink(avatarId: string): string {
    return isAvatarId(avatarId) ? entityRelayLink('avatar', avatarId) : '';
}
