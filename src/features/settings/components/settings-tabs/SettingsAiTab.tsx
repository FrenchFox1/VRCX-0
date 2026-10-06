import { SettingsTabContent } from '../SettingsViewParts';
import { AssistantSettingsGroup } from './AssistantSettingsGroup';
import { McpServerSettingsGroup } from './McpServerSettingsGroup';

type SettingsAiTabProps = {
    active: boolean;
};

export function SettingsAiTab({ active }: SettingsAiTabProps) {
    return (
        <SettingsTabContent value="ai">
            <AssistantSettingsGroup active={active} />
            <McpServerSettingsGroup />
        </SettingsTabContent>
    );
}
