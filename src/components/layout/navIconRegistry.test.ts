import { DatabaseBackupIcon } from 'lucide-react';
import { describe, expect, it } from 'vitest';

import { toolDefinitionMap } from '@/shared/constants/tools';

import { getNavIconComponent } from './navIconRegistry';

describe('navigation icon registry', () => {
    it('resolves the profile backup tool icon', () => {
        const tool = toolDefinitionMap.get('profile-backup');

        expect(getNavIconComponent(tool?.navIcon)).toBe(DatabaseBackupIcon);
    });
});
