import { commands, type RequestInviteRequest } from '@/platform/tauri/bindings';
import notificationPersistenceRepository from '@/repositories/notificationPersistenceRepository';

interface SendInviteToLocationInput {
    receiverUserId?: string;
    instanceId?: string;
    worldId?: string;
    worldName?: string;
    messageSlot?: number | null;
    imageData?: string;
    rsvp?: boolean | null;
}

interface SendInvitesToLocationInput {
    receiverUserIds?: string[];
    location?: string;
    shortName?: string;
    worldName?: string;
}

interface SendRequestInviteToUserInput {
    receiverUserId?: string;
    requestSlot?: number | null;
    imageData?: string;
}

interface SendBoopToUserInput {
    userId?: string;
    emojiId?: string;
}

function normalizeText(value?: string | null): string {
    return value?.trim() ?? '';
}

export async function sendInvitesToLocation({
    receiverUserIds = [],
    location,
    shortName,
    worldName
}: SendInvitesToLocationInput = {}) {
    return commands.appInstanceInviteBatch({
        receiverUserIds: receiverUserIds
            .map((userId) => userId.trim())
            .filter(Boolean),
        location: normalizeText(location),
        shortName: normalizeText(shortName),
        worldName: normalizeText(worldName)
    });
}

export async function sendInviteToLocation({
    receiverUserId,
    instanceId,
    worldId,
    worldName,
    messageSlot = null,
    imageData = '',
    rsvp
}: SendInviteToLocationInput = {}) {
    const normalizedReceiverUserId = normalizeText(receiverUserId);
    const normalizedInstanceId = normalizeText(instanceId);
    const normalizedWorldId = normalizeText(worldId);
    if (
        !normalizedReceiverUserId ||
        !normalizedInstanceId ||
        !normalizedWorldId
    ) {
        return null;
    }

    const outcome = await commands.appNotificationInstanceInviteSend({
        receiverUserId: normalizedReceiverUserId,
        instanceId: normalizedInstanceId,
        worldId: normalizedWorldId,
        worldName: normalizeText(worldName),
        messageSlot,
        imageData: normalizeText(imageData),
        rsvp
    });
    if (outcome.status === 'remoteFailed') {
        throw new Error(
            outcome.remoteError || 'VRChat notification request failed'
        );
    }
    return outcome;
}

export async function sendRequestInviteToUser({
    receiverUserId,
    requestSlot = null,
    imageData = ''
}: SendRequestInviteToUserInput = {}) {
    const normalizedReceiverUserId = normalizeText(receiverUserId);
    if (!normalizedReceiverUserId) {
        return null;
    }

    const params: RequestInviteRequest = {};
    if (requestSlot !== null) {
        params.requestSlot = requestSlot;
    }

    const normalizedImageData = normalizeText(imageData);
    if (normalizedImageData) {
        return notificationPersistenceRepository.sendRequestInvitePhoto({
            receiverUserId: normalizedReceiverUserId,
            params,
            imageData: normalizedImageData
        });
    }

    return notificationPersistenceRepository.sendRequestInvite({
        receiverUserId: normalizedReceiverUserId,
        params
    });
}

export async function sendBoopToUser({
    userId,
    emojiId = ''
}: SendBoopToUserInput = {}) {
    const normalizedUserId = normalizeText(userId);
    if (!normalizedUserId) {
        return null;
    }

    return notificationPersistenceRepository.sendBoop({
        userId: normalizedUserId,
        emojiId
    });
}
