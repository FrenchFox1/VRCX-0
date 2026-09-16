import type { UserProfileEntity } from '@/domain/entities/user';

export function mergeCurrentUserMediaFields<TUser extends UserProfileEntity>(
    user: TUser,
    profile: UserProfileEntity
) {
    return {
        ...user,
        userIcon: profile.userIcon || '',
        bannerCustomUrl: profile.bannerCustomUrl || '',
        iconUrl: profile.iconUrl ?? user.iconUrl,
        bannerUrl: profile.bannerUrl ?? user.bannerUrl,
        bannerType: profile.bannerType ?? user.bannerType,
        iconType: profile.iconType ?? user.iconType
    };
}
