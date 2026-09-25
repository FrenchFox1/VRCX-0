// @vitest-environment jsdom

import { cleanup, render, screen, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import vrchatInstanceRepository from '@/repositories/vrchatInstanceRepository';

import { useInstancePopulation } from './useInstancePopulation';

vi.mock('@/repositories/vrchatInstanceRepository', () => ({
    default: { getInstance: vi.fn() }
}));

function Harness({
    enabled,
    refreshKey
}: {
    enabled: boolean;
    refreshKey: number;
}) {
    const { ref, population } = useInstancePopulation({
        worldId: 'wrld_a',
        instanceId: '12345~friends(usr_a)',
        enabled,
        refreshKey
    });
    return (
        <div ref={ref} data-testid="population">
            {population
                ? `${population.nUsers}/${population.capacity}`
                : 'none'}
        </div>
    );
}

beforeEach(() => {
    vi.resetAllMocks();
    vi.mocked(vrchatInstanceRepository.getInstance).mockResolvedValue({
        json: { n_users: 12, capacity: 32 }
    } as Awaited<ReturnType<typeof vrchatInstanceRepository.getInstance>>);
});

afterEach(() => {
    cleanup();
});

describe('useInstancePopulation', () => {
    it('loads the instance population once the row is shown', async () => {
        render(<Harness enabled refreshKey={1} />);

        await waitFor(() => {
            expect(screen.getByTestId('population').textContent).toBe('12/32');
        });
        expect(vrchatInstanceRepository.getInstance).toHaveBeenCalledWith({
            worldId: 'wrld_a',
            instanceId: '12345~friends(usr_a)'
        });
    });

    it('refetches when the refresh key changes', async () => {
        const { rerender } = render(<Harness enabled refreshKey={1} />);
        await waitFor(() => {
            expect(vrchatInstanceRepository.getInstance).toHaveBeenCalledTimes(
                1
            );
        });

        rerender(<Harness enabled refreshKey={2} />);

        await waitFor(() => {
            expect(vrchatInstanceRepository.getInstance).toHaveBeenCalledTimes(
                2
            );
        });
    });

    it('does not request locations that are not real instances', () => {
        render(<Harness enabled={false} refreshKey={1} />);

        expect(vrchatInstanceRepository.getInstance).not.toHaveBeenCalled();
        expect(screen.getByTestId('population').textContent).toBe('none');
    });
});
