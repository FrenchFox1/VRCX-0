import { Toggle } from '@/ui/shadcn/toggle';
import { ToggleGroup, ToggleGroupItem } from '@/ui/shadcn/toggle-group';

export function OptionToggle({
    label,
    active,
    onToggle
}: {
    label: string;
    active: boolean;
    onToggle: (next: boolean) => void;
}) {
    return (
        <Toggle
            variant="outline"
            size="sm"
            pressed={active}
            onPressedChange={onToggle}
            aria-label={label}
            className="shrink-0 text-xs"
        >
            {label}
        </Toggle>
    );
}

export function OptionSegmented<T extends string>({
    value,
    options,
    onValueChange
}: {
    value: T;
    options: readonly { value: T; label: string }[];
    onValueChange: (next: T) => void;
}) {
    return (
        <ToggleGroup
            variant="outline"
            size="sm"
            value={[value]}
            onValueChange={(next) => {
                const selected = options.find(
                    (option) => option.value === next[0]
                );
                if (selected) {
                    onValueChange(selected.value);
                }
            }}
            className="shrink-0"
        >
            {options.map((option) => (
                <ToggleGroupItem
                    key={option.value}
                    value={option.value}
                    aria-label={option.label}
                    className="text-xs"
                >
                    {option.label}
                </ToggleGroupItem>
            ))}
        </ToggleGroup>
    );
}
