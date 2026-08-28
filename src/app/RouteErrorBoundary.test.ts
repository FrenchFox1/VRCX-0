import { describe, expect, it } from 'vitest';

import { classifyRouteError, RouteErrorBoundary } from './RouteErrorBoundary';

describe('classifyRouteError', () => {
    it('classifies chunk loading failures as load_fail', () => {
        expect(classifyRouteError(new Error('Loading chunk 123 failed'))).toBe(
            'load_fail'
        );
        expect(
            classifyRouteError(
                new Error('Failed to fetch dynamically imported module')
            )
        ).toBe('load_fail');
    });

    it('classifies other render exceptions as render_crash', () => {
        expect(classifyRouteError(new TypeError('bad props'))).toBe(
            'render_crash'
        );
        expect(classifyRouteError('unexpected')).toBe('render_crash');
    });
});

describe('RouteErrorBoundary', () => {
    it('enters the fallback state after a render error', () => {
        expect(RouteErrorBoundary.getDerivedStateFromError()).toEqual({
            hasError: true
        });
    });

    it('renders its fallback while handling an error', () => {
        const boundary = new RouteErrorBoundary({
            resetKey: 'route-a',
            fallback: 'fallback',
            children: 'content'
        });

        expect(boundary.render()).toBe('content');
        boundary.state = {
            hasError: true,
            renderedKey: 'route-a'
        };
        expect(boundary.render()).toBe('fallback');
    });
});
