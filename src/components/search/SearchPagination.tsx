import { ArrowLeftIcon, ArrowRightIcon } from 'lucide-react';
import { useTranslation } from 'react-i18next';

import { KeyboardShortcut } from '@/components/keyboard/KeyboardShortcut';
import { usePaginationKeyboardShortcuts } from '@/components/keyboard/usePaginationKeyboardShortcuts';
import { Button } from '@/ui/shadcn/button';
import {
    Pagination,
    PaginationContent,
    PaginationItem
} from '@/ui/shadcn/pagination';

export function SearchPagination({
    show = false,
    prevDisabled = true,
    nextDisabled = true,
    onPrev,
    onNext
}: {
    show?: boolean;
    prevDisabled?: boolean;
    nextDisabled?: boolean;
    onPrev: () => void;
    onNext: () => void;
}) {
    const { t } = useTranslation();
    const paginationRef = usePaginationKeyboardShortcuts<HTMLElement>({
        enabled: show,
        canPrevious: !prevDisabled,
        canNext: !nextDisabled,
        onPrevious: onPrev,
        onNext
    });

    if (!show) {
        return null;
    }

    return (
        <Pagination ref={paginationRef} className="h-16 shrink-0">
            <PaginationContent>
                <PaginationItem>
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        aria-label={'Previous search page'}
                        disabled={prevDisabled}
                        onClick={onPrev}
                    >
                        <ArrowLeftIcon data-icon="inline-start" />
                        {t('table.pagination.previous')}
                        <KeyboardShortcut keys="ArrowLeft" />
                    </Button>
                </PaginationItem>
                <PaginationItem>
                    <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        aria-label={'Next search page'}
                        disabled={nextDisabled}
                        onClick={onNext}
                    >
                        {t('table.pagination.next')}
                        <KeyboardShortcut keys="ArrowRight" />
                        <ArrowRightIcon data-icon="inline-end" />
                    </Button>
                </PaginationItem>
            </PaginationContent>
        </Pagination>
    );
}
