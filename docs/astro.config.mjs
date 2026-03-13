// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Lexicon',
			description: 'Contract-driven verification for Rust libraries and workspaces',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/anthropics/lexicon' }],
			sidebar: [
				{
					label: 'Getting Started',
					items: [
						{ label: 'Installation', slug: 'getting-started/installation' },
						{ label: 'Quickstart', slug: 'getting-started/quickstart' },
					],
				},
				{
					label: 'Commands',
					autogenerate: { directory: 'commands' },
				},
				{
					label: 'Concepts',
					items: [
						{ label: 'Contracts', slug: 'concepts/contracts' },
						{ label: 'Conformance', slug: 'concepts/conformance' },
						{ label: 'Scoring', slug: 'concepts/scoring' },
						{ label: 'Gates', slug: 'concepts/gates' },
						{ label: 'Conversations', slug: 'concepts/conversations' },
						{ label: 'AI Integration', slug: 'concepts/ai-integration' },
					],
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
				},
				{
					label: 'Guides',
					autogenerate: { directory: 'guides' },
				},
			],
		}),
	],
});
