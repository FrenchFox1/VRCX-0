// @vitest-environment jsdom

import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

vi.mock('react-i18next', () => ({
    useTranslation: () => ({ t: (key: string) => key })
}));

import { AccentColorPicker } from './AccentColorPicker';

afterEach(cleanup);

describe('AccentColorPicker custom color', () => {
    it('applies colors selected by the native picker', () => {
        const updateThemeColor = vi.fn<() => Promise<void>>();
        render(
            <AccentColorPicker
                accentControlled={false}
                themeColor="default"
                updateThemeColor={updateThemeColor}
            />
        );

        fireEvent.change(screen.getByLabelText('view.themes.accent.custom'), {
            target: { value: '#ff00aa' }
        });

        expect(updateThemeColor).toHaveBeenCalledWith('#ff00aa');
    });

    it('normalizes a typed color when the input loses focus', async () => {
        const user = userEvent.setup();
        const updateThemeColor = vi.fn<() => Promise<void>>();
        render(
            <AccentColorPicker
                accentControlled={false}
                themeColor="default"
                updateThemeColor={updateThemeColor}
            />
        );
        const input = screen.getByLabelText('view.themes.accent.hex_label');

        await user.clear(input);
        await user.type(input, '#0af');
        await user.tab();

        expect(updateThemeColor).toHaveBeenCalledWith('#00aaff');
        expect((input as HTMLInputElement).value).toBe('#00aaff');
    });

    it('shows and activates the custom color through its button', async () => {
        const user = userEvent.setup();
        const updateThemeColor = vi.fn<() => Promise<void>>();
        const { rerender } = render(
            <AccentColorPicker
                accentControlled={false}
                themeColor="#123456"
                updateThemeColor={updateThemeColor}
            />
        );
        const customButton = screen.getByRole('button', {
            name: 'view.themes.accent.custom'
        });
        const neutralButton = screen.getByRole('button', {
            name: 'view.settings.appearance.theme_color.default'
        });
        const swatch = customButton.querySelector('[aria-hidden="true"]');

        expect(customButton.parentElement).toBe(neutralButton.parentElement);
        expect((swatch as HTMLSpanElement).style.backgroundColor).toBe(
            'rgb(18, 52, 86)'
        );
        expect(customButton.getAttribute('data-variant')).toBe('default');

        rerender(
            <AccentColorPicker
                accentControlled={false}
                themeColor="default"
                updateThemeColor={updateThemeColor}
            />
        );

        expect(customButton.getAttribute('data-variant')).toBe('outline');
        await user.click(customButton);
        expect(updateThemeColor).toHaveBeenCalledWith('#123456');
    });

    it('rejects invalid typed colors and disables both inputs when controlled', async () => {
        const user = userEvent.setup();
        const updateThemeColor = vi.fn<() => Promise<void>>();
        const { rerender } = render(
            <AccentColorPicker
                accentControlled={false}
                themeColor="#123456"
                updateThemeColor={updateThemeColor}
            />
        );
        const textInput = screen.getByLabelText('view.themes.accent.hex_label');
        const customButton = screen.getByRole('button', {
            name: 'view.themes.accent.custom'
        });

        expect(customButton.getAttribute('data-variant')).toBe('default');

        await user.clear(textInput);
        await user.type(textInput, '#nope');
        await user.tab();

        expect(updateThemeColor).not.toHaveBeenCalled();
        expect((textInput as HTMLInputElement).value).toBe('#123456');

        rerender(
            <AccentColorPicker
                accentControlled
                themeColor="#123456"
                updateThemeColor={updateThemeColor}
            />
        );

        expect(customButton.getAttribute('data-variant')).toBe('default');
        expect((customButton as HTMLButtonElement).disabled).toBe(true);
        expect(
            (
                screen.getByLabelText(
                    'view.themes.accent.custom'
                ) as HTMLInputElement
            ).disabled
        ).toBe(true);
        expect(
            (
                screen.getByLabelText(
                    'view.themes.accent.hex_label'
                ) as HTMLInputElement
            ).disabled
        ).toBe(true);
    });
});
