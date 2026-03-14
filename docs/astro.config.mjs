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
						{ label: 'Daily Usage', slug: 'getting-started/daily-usage' },
					],
				},
				{
					label: 'Commands',
					items: [
						{
							label: 'AI',
							items: [
								{ label: 'chat', slug: 'commands/chat' },
								{ label: 'auth', slug: 'commands/auth' },
							],
						},
						{
							label: 'Verification',
							items: [
								{ label: 'verify', slug: 'commands/verify' },
								{ label: 'coverage', slug: 'commands/coverage' },
								{ label: 'api', slug: 'commands/api' },
								{ label: 'doctor', slug: 'commands/doctor' },
							],
						},
						{
							label: 'Governance',
							items: [
								{ label: 'workspace', slug: 'commands/workspace' },
								{ label: 'ecosystem', slug: 'commands/ecosystem' },
							],
						},
						{
							label: 'Utilities',
							items: [
								{ label: 'init', slug: 'commands/init' },
								{ label: 'prompt', slug: 'commands/prompt' },
								{ label: 'sync', slug: 'commands/sync' },
								{ label: 'tui', slug: 'commands/tui' },
							],
						},
					],
				},
				{
					label: 'Concepts',
					items: [
						{ label: 'The Lexicon Model', slug: 'concepts/lexicon' },
						{ label: 'Progressive Adoption', slug: 'concepts/progressive-adoption' },
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
