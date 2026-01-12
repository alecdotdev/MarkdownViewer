<script lang="ts">
	import { invoke, convertFileSrc } from '@tauri-apps/api/tauri';
	import { onMount } from 'svelte';
	import { open } from '@tauri-apps/api/shell';

	let editPath: string = '';
	let htmlContent: string = '';

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
			await open(editPath, 'code');
		} else {
			console.error('No file path available to open in editor.');
		}
	}

	onMount(() => {
		invoke('send_markdown_path')
			.then((path: string) => {
				editPath = path;
				loadMarkdown(path);
			})
			.catch((error) => {
				console.error('Error receiving Markdown file path:', error);
			});
	});

	onMount(async () => {
		setTimeout(() => {
			setupWindow();
			interceptLinks();
		}, 300);
	});

	// https://github.com/tauri-apps/tauri/issues/5170
	async function setupWindow() {
		const appWindow = (await import('@tauri-apps/api/window')).appWindow;
		appWindow.show();
	}

	function interceptLinks() {
		document.addEventListener('click', async (event) => {
			let target = event.target as HTMLElement;

			while (target && target.tagName !== 'A' && target !== document.body) {
				target = target.parentElement;
			}

			if (target && target.tagName === 'A' && (target as HTMLAnchorElement).href) {
				const anchor = target as HTMLAnchorElement;

				if (!anchor.href.startsWith('#')) {
					event.preventDefault();
					await open(anchor.href);
				}
			}
		});
	}
</script>

{#if !htmlContent}
	<div class="message">
		<p>Open a Markdown file by right-clicking and selecting 'Open with...' &rightarrow; 'Markdown Viewer'</p>
	</div>
{:else}
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
</style>
