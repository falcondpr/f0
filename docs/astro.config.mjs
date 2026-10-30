// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://f0.falcondpr.com',
	integrations: [
		starlight({
			title: 'f0',
			description: 'A worktree-first terminal workspace for AI-assisted development.',
			logo: {
				light: './src/assets/logo-light.svg',
				dark: './src/assets/logo-dark.svg',
			},
			customCss: ['./src/styles/custom.css'],
			components: {
				ThemeSelect: './src/components/ThemeSelect.astro',
			},
			expressiveCode: {
				themes: ['github-dark', 'github-light'],
				// Shell snippets render as plain code cards, not terminal windows.
				defaultProps: { frame: 'code' },
				styleOverrides: {
					borderRadius: '0.75rem',
					borderColor: 'var(--sl-color-hairline-light)',
					codeBackground: 'var(--f0-surface)',
					codeFontSize: '0.8125rem',
					uiFontFamily: 'var(--__sl-font)',
					frames: {
						frameBoxShadowCssValue: 'none',
						editorTabBarBackground: 'var(--f0-surface)',
						editorActiveTabBackground: 'var(--f0-surface)',
						editorActiveTabIndicatorTopColor: 'transparent',
						editorActiveTabIndicatorBottomColor: 'var(--sl-color-text-accent)',
						editorTabBarBorderBottomColor: 'var(--sl-color-hairline-light)',
						inlineButtonBackground: 'var(--sl-color-black)',
						inlineButtonBorder: 'var(--sl-color-hairline-light)',
					},
				},
			},
			defaultLocale: 'root',
			locales: {
				root: { label: 'English', lang: 'en' },
				es: { label: 'Español', lang: 'es' },
			},
			sidebar: [
				// Outside the groups and styled as a shortcut in custom.css, so visitors notice it.
				{ slug: 'cheatsheet', attrs: { class: 'sidebar-cheatsheet' } },
				{
					label: 'Getting started',
					translations: { es: 'Primeros pasos' },
					items: [{ autogenerate: { directory: 'getting-started' } }],
				},
				{
					label: 'Guides',
					translations: { es: 'Guías' },
					items: [{ autogenerate: { directory: 'guides' } }],
				},
				{
					label: 'Reference',
					translations: { es: 'Referencia' },
					items: [{ autogenerate: { directory: 'reference' } }],
				},
				{
					label: 'Concepts',
					translations: { es: 'Conceptos' },
					items: [{ autogenerate: { directory: 'concepts' } }],
				},
			],
		}),
	],
});
