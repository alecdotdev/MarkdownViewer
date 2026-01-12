<script lang="ts">
	import { invoke, convertFileSrc } from '@tauri-apps/api/core';
	import { onMount } from 'svelte';
	import { openUrl, openPath } from '@tauri-apps/plugin-opener';
	import { open } from '@tauri-apps/plugin-dialog';

	let editPath = $state('');
	let htmlContent = $state('');

	async function loadMarkdown(filePath: string) {
		try {
			const html = (await invoke('open_markdown', { path: filePath })) as string;

			const parser = new DOMParser();
			const doc = parser.parseFromString(html, 'text/html');
			const imgs = doc.querySelectorAll('img');

			for (const img of imgs) {
				const src = img.getAttribute('src');
				if (src && !src.startsWith('http') && !src.startsWith('data:')) {
					const absolutePath = resolvePath(filePath, src);
					img.setAttribute('src', convertFileSrc(absolutePath));
				} else if (src && isYoutubeLink(src)) {
					const videoId = getYoutubeId(src);
					if (videoId) {
						replaceWithYoutubeEmbed(img, videoId);
					}
				}
			}

			const anchors = doc.querySelectorAll('a');
			for (const a of anchors) {
				const href = a.getAttribute('href');
				if (href && isYoutubeLink(href)) {
					// only embed if it's the only child of a paragraph or div
					const parent = a.parentElement;
					if (parent && (parent.tagName === 'P' || parent.tagName === 'DIV') && parent.childNodes.length === 1) {
						const videoId = getYoutubeId(href);
						if (videoId) {
							replaceWithYoutubeEmbed(a, videoId);
						}
					}
				}
			}

			// Parse GFM Alerts
			const blockquotes = doc.querySelectorAll('blockquote');
			for (const bq of blockquotes) {
				const firstP = bq.querySelector('p');
				if (firstP) {
					const text = firstP.textContent || '';
					const match = text.match(/^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]/i);
					if (match) {
						const type = match[1].toLowerCase();
						const alertDiv = doc.createElement('div');
						alertDiv.className = `markdown-alert markdown-alert-${type}`;

						const titleP = doc.createElement('p');
						titleP.className = 'markdown-alert-title';

						// Add icon based on type
						const icon = doc.createElement('span');
						icon.className = 'alert-icon';
						titleP.appendChild(icon);

						const titleText = doc.createTextNode(type.charAt(0).toUpperCase() + type.slice(1));
						titleP.appendChild(titleText);

						alertDiv.appendChild(titleP);

						// Remove the [!TYPE] text from the first paragraph
						firstP.textContent = text.replace(/^\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]/i, '').trim() || '';
						if (firstP.textContent === '' && firstP.nextSibling) {
							firstP.remove();
						}

						// Move contents of blockquote to alert div
						while (bq.firstChild) {
							alertDiv.appendChild(bq.firstChild);
						}

						bq.replaceWith(alertDiv);
					}
				}
			}

			htmlContent = doc.body.innerHTML;
		} catch (error) {
			console.error('Error loading Markdown file:', error);
		}
	}

	function resolvePath(basePath: string, relativePath: string) {
		if (relativePath.match(/^[a-zA-Z]:/) || relativePath.startsWith('/')) {
			return relativePath;
		}

		const parts = basePath.split(/[/\\]/);
		parts.pop();

		const relParts = relativePath.split(/[/\\]/);
		for (const p of relParts) {
			if (p === '.') continue;
			if (p === '..') {
				parts.pop();
			} else {
				parts.push(p);
			}
		}
		return parts.join('/');
	}

	function isYoutubeLink(url: string) {
		return url.includes('youtube.com/watch') || url.includes('youtu.be/');
	}

	function getYoutubeId(url: string) {
		const regExp = /^.*(youtu.be\/|v\/|u\/\w\/|embed\/|watch\?v=|\&v=)([^#\&\?]*).*/;
		const match = url.match(regExp);
		return match && match[2].length === 11 ? match[2] : null;
	}

	function replaceWithYoutubeEmbed(element: Element, videoId: string) {
		const container = element.ownerDocument.createElement('div');
		container.className = 'video-container';
		const iframe = element.ownerDocument.createElement('iframe');
		iframe.src = `https://www.youtube.com/embed/${videoId}`;
		iframe.title = 'YouTube video player';
		iframe.frameBorder = '0';
		iframe.allow = 'accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share';
		iframe.allowFullscreen = true;
		container.appendChild(iframe);
		element.replaceWith(container);
	}

	async function openInEditor() {
		if (editPath) {
			await openPath(editPath);
		} else {
			console.error('No file path available to open in editor.');
		}
	}

	async function selectFile() {
		const selected = await open({
			multiple: false,
			filters: [
				{
					name: 'Markdown',
					extensions: ['md'],
				},
			],
		});
		if (selected && typeof selected === 'string') {
			editPath = selected;
			loadMarkdown(selected);
		}
	}

	onMount(() => {
		invoke('send_markdown_path')
			.then((path: any) => {
				editPath = path as string;
				loadMarkdown(editPath);
			})
			.catch((error) => {
				console.error('Error receiving Markdown file path:', error);
			});
	});

	onMount(async () => {
		interceptLinks();
	});

	function interceptLinks() {
		document.addEventListener('click', async (event) => {
			let target = event.target as HTMLElement;

			while (target && target.tagName !== 'A' && target !== document.body) {
				target = target.parentElement as HTMLElement;
			}

			if (target && target.tagName === 'A' && (target as HTMLAnchorElement).href) {
				const anchor = target as HTMLAnchorElement;

				if (!anchor.href.startsWith('#')) {
					event.preventDefault();
					await openUrl(anchor.href);
				}
			}
		});
	}
</script>

{#if !htmlContent}
	<div class="message">
		<p>Open a Markdown file by right-clicking and selecting 'Open with...' &rightarrow; 'Markdown Viewer'</p>
		<button class="open-btn" onclick={selectFile}> Open file </button>
	</div>
{:else}
	<div class="fab">
		<button class="open-btn fab-btn" onclick={selectFile}> Open file </button>
	</div>
	<article contenteditable="false" class="markdown-body" bind:innerHTML={htmlContent}></article>
{/if}

<style>
	:root {
		--animation: cubic-bezier(0.05, 0.95, 0.05, 0.95);
		scroll-behavior: smooth;
		background-color: transparent !important;
	}

	:global(body) {
		background-color: transparent !important;
		margin: 0;
		padding: 0;
	}

	.markdown-body {
		box-sizing: border-box;
		min-width: 200px;
		max-width: 980px;
		margin: 0 auto;
		padding: 45px;
	}

	:global(.video-container) {
		position: relative;
		padding-bottom: 56.25%; /* 16:9 */
		height: 0;
		overflow: hidden;
		max-width: 100%;
		margin: 1em 0;
	}

	:global(.video-container iframe) {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
		height: 100%;
		border-radius: 8px;
	}

	.message {
		display: flex;
		flex-direction: column;
		justify-content: center;
		align-items: center;
		user-select: none;
		font-family:
			Segoe UI,
			Helvetica Neue,
			Helvetica,
			Arial,
			sans-serif;
		height: 90vh;
	}

	@media (prefers-color-scheme: dark) {
		.message {
			color: #ffffffaa;
		}
	}

	@media (prefers-color-scheme: light) {
		.message {
			color: #000000aa;
		}
	}

	.open-btn {
		background: #0078d4;
		color: white;
		border: 1px solid rgba(0, 0, 0, 0.1);
		padding: 6px 24px;
		border-radius: 4px;
		cursor: pointer;
		font-weight: 400;
		font-family: 'Segoe UI Variable Text', 'Segoe UI', sans-serif;
		font-size: 14px;
		transition: all 0.1s cubic-bezier(0, 0, 0, 1);
		margin-top: 20px;
		box-shadow: 0 2px 4px rgba(0, 0, 0, 0.05);
	}

	.open-btn:hover {
		background: #005a9e;
		box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
	}

	.open-btn:active {
		background: #004578;
		transform: scale(0.98);
	}

	.fab {
		position: fixed;
		top: 20px;
		right: 20px;
		z-index: 100;
	}

	.fab-btn {
		margin-top: 0;
		background: rgba(255, 255, 255, 0.2);
		backdrop-filter: blur(10px);
		-webkit-backdrop-filter: blur(10px);
		color: var(--color-fg-default);
		border: 1px solid rgba(255, 255, 255, 0.1);
		opacity: 0.8;
	}

	@media (prefers-color-scheme: dark) {
		.fab-btn {
			background: rgba(32, 32, 32, 0.4);
			border: 1px solid rgba(255, 255, 255, 0.05);
		}
	}

	.fab-btn:hover {
		opacity: 1;
		background: rgba(255, 255, 255, 0.3);
	}

	@media (prefers-color-scheme: dark) {
		.fab-btn:hover {
			background: rgba(48, 48, 48, 0.6);
		}
	}
</style>
