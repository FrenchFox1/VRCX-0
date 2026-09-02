// @vitest-environment jsdom

import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { ContextMenu, ContextMenuTrigger } from '@/ui/shadcn/context-menu';
import { Table, TableBody, TableCell } from '@/ui/shadcn/table';

import { DataTableRow } from './DataTableView';

describe('DataTableRow', () => {
    afterEach(cleanup);

    it('keeps its table-row marker when used as a context-menu trigger', () => {
        render(
            <ContextMenu>
                <Table>
                    <TableBody>
                        <ContextMenuTrigger
                            render={
                                <DataTableRow>
                                    <TableCell>Avatar</TableCell>
                                </DataTableRow>
                            }
                        />
                    </TableBody>
                </Table>
            </ContextMenu>
        );

        const row = screen.getByRole('row');
        expect(row.getAttribute('data-slot')).toBe('context-menu-trigger');
        expect(row.hasAttribute('data-vrcx-0-table-row')).toBe(true);
    });
});
