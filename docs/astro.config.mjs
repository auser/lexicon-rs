// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	integrations: [
		starlight({
			title: 'Lexicon',
			description: 'Contract-driven verification and governance for software systems',
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
						{ label: 'The Lexicon Model', slug: 'concepts/lexicon' },
						{ label: 'Contracts', slug: 'concepts/contracts' },
						{ label: 'Conformance', slug: 'concepts/conformance' },
						{ label: 'API Extraction', slug: 'concepts/api-extraction' },
						{ label: 'Contract Coverage', slug: 'concepts/coverage' },
						{ label: 'Gates', slug: 'concepts/gates' },
						{ label: 'Scoring', slug: 'concepts/scoring' },
						{ label: 'Architecture', slug: 'concepts/architecture' },
						{ label: 'Ecosystem Governance', slug: 'concepts/ecosystem' },
						{ label: 'Conversations', slug: 'concepts/conversations' },
						{ label: 'AI Agents', slug: 'concepts/ai-agents' },
					],
				},
				{
					label: 'Guides',
					autogenerate: { directory: 'guides' },
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
				},
			],
		}),
	],
});
