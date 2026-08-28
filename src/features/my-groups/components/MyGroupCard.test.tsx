// @vitest-environment jsdom

import { DndContext } from '@dnd-kit/core';
import { SortableContext } from '@dnd-kit/sortable';
import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key })
}));

vi.mock('@/services/dialogService', () => ({
    openGroupDialog: vi.fn()
}));

import type { MyGroupRow } from '../useMyGroupsPageState';
import { MyGroupCard } from './MyGroupCard';

const group: MyGroupRow = {
    bannerUrl: '',
    description: '',
    discriminator: '1234',
    displayName: 'Example Group',
    iconUrl: '',
    id: 'grp_example',
    languages: [],
    links: [],
    memberCount: 42,
    memberVisibility: 'hidden',
    membershipStatus: 'member',
    name: 'Example Group',
    onlineMemberCount: 0,
    ownerDisplayName: 'Owner',
    ownerId: 'usr_owner',
    privacy: 'default',
    roles: [],
    rules: '',
    shortCode: 'TEST',
    tags: [],
    url: ''
};

function renderCard({
    editMode,
    selected = false,
    actionsDisabled = false,
    onToggleSelected = vi.fn()
}: {
    editMode: boolean;
    selected?: boolean;
    actionsDisabled?: boolean;
    onToggleSelected?: (groupId: string) => void;
}) {
    return render(
        <DndContext>
            <SortableContext items={[group.id]}>
                <MyGroupCard
                    group={group}
                    editMode={editMode}
                    orderEditable={editMode}
                    orderIndex={2}
                    orderBusy={false}
                    selected={selected}
                    actionsDisabled={actionsDisabled}
                    isOwner
                    onToggleSelected={onToggleSelected}
                    onSetVisibility={vi.fn()}
                    onLeave={vi.fn()}
                />
            </SortableContext>
        </DndContext>
    );
}

describe('MyGroupCard', () => {
    afterEach(cleanup);

    it('uses the shared gallery card shell and edit overlays', () => {
        const onToggleSelected = vi.fn();
        const { container } = renderCard({
            editMode: true,
            selected: true,
            onToggleSelected
        });

        expect(
            screen.queryByLabelText('view.my_groups.row_actions')
        ).toBeNull();

        const checkbox = screen.getByRole('checkbox', {
            name: 'common.actions.select Example Group'
        });
        fireEvent.click(checkbox);
        expect(onToggleSelected).toHaveBeenCalledWith('grp_example');

        const card = container.querySelector('button[aria-pressed="true"]');
        expect(card?.className).toContain('h-full');
        expect(card?.className).toContain('rounded-lg');
        expect(card?.className).toContain('ring-primary');

        const order = screen.getByText('3');
        expect(order.className).toContain('top-1');
        expect(order.className).toContain('right-1');
    });

    it('shows ownership and visibility beside the name outside edit mode', () => {
        renderCard({ editMode: false });

        expect(
            screen.getByLabelText('dialog.group.label.owner_2')
        ).toBeTruthy();
        expect(
            screen.getByLabelText('dialog.group.actions.visibility_hidden')
        ).toBeTruthy();
        expect(
            screen.getByLabelText('view.my_groups.row_actions')
        ).toBeTruthy();
    });

    it('keeps every row action trigger hidden while a batch action is busy', () => {
        renderCard({ editMode: false, actionsDisabled: true });

        const trigger = screen.getByLabelText('view.my_groups.row_actions');
        expect(trigger.hasAttribute('disabled')).toBe(true);
        expect(trigger.className).toContain('disabled:invisible');
    });
});
