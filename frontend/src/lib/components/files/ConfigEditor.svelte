<script>
	import { onMount } from 'svelte';
	import { EditorView, lineNumbers, keymap, highlightActiveLine, highlightActiveLineGutter } from '@codemirror/view';
	import { EditorState, Compartment } from '@codemirror/state';
	import { yaml } from '@codemirror/lang-yaml';
	import {
		Save,
		FileCode,
		CheckCircle2,
		AlertCircle,
		FileText,
		Settings,
		ChevronRight,
		RotateCw,
		SlidersHorizontal,
		Check,
		GitCompare
	} from 'lucide-svelte';
	import ConfigDiffModal from './ConfigDiffModal.svelte';

	let showDiffModal = $state(false);

	/**
	 * @typedef {Object} FileData
	 * @property {string} path
	 * @property {string} content
	 * @property {string} syntax_mode
	 * @property {number} size_bytes
	 */

	let {
		fileData = null,
		isSaving = false,
		error = '',
		isLoading = false,
		onSave = (content, triggerReload) => {},
		onRetry = () => {},
		onDirtyChange = (dirty) => {}
	} = $props();

	let editorContainer = $state(null);
	let editorView = $state(null);
	let currentContent = $state('');
	let selectedSyntax = $state('YAML');
	let triggerReload = $state(false);

	const syntaxOptions = ['YAML', 'Properties', 'JSON', 'TOML', 'TXT', 'LOG'];
	const languageCompartment = new Compartment();

	// Derived dirty unsaved state
	let isDirty = $derived(fileData ? currentContent !== (fileData.content ?? '') : false);

	// Report dirty state upward so the parent's "unsaved changes" guard actually fires.
	$effect(() => {
		onDirtyChange(isDirty);
	});

	// CodeMirror Theme definition matching ChiPanel tokens
	const chiPanelTheme = EditorView.theme(
		{
			'&': {
				color: 'var(--text-primary)',
				backgroundColor: 'var(--bg-surface)',
				fontFamily: 'var(--font-mono)',
				fontSize: 'var(--font-size-sm)',
				height: '100%',
				outline: 'none'
			},
			'.cm-content': {
				caretColor: 'var(--accent-blue)',
				fontFamily: 'var(--font-mono)',
				padding: '12px 0'
			},
			'.cm-cursor, .cm-dropCursor': {
				borderLeftColor: 'var(--accent-blue)',
				borderLeftWidth: '2px'
			},
			'&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection': {
				backgroundColor: 'rgba(99, 102, 241, 0.3)'
			},
			'.cm-activeLine': {
				backgroundColor: 'rgba(255, 255, 255, 0.03)'
			},
			'.cm-gutters': {
				backgroundColor: 'var(--bg-base)',
				color: 'var(--text-muted)',
				borderRight: '1px solid var(--border-subtle)',
				fontFamily: 'var(--font-mono)',
				minWidth: '42px'
			},
			'.cm-activeLineGutter': {
				backgroundColor: 'var(--bg-elevated)',
				color: 'var(--text-primary)'
			},
			'.cm-lineNumbers .cm-gutterElement': {
				padding: '0 12px 0 8px'
			},
			'.cm-scroller': {
				overflow: 'auto',
				fontFamily: 'var(--font-mono)'
			}
		},
		{ dark: true }
	);

	// CodeMirror Initialization
	$effect(() => {
		if (!editorContainer) return;

		const initialDoc = fileData?.content ?? '';
		currentContent = initialDoc;

		if (fileData?.syntax_mode) {
			selectedSyntax = fileData.syntax_mode.toUpperCase();
		}

		const state = EditorState.create({
			doc: initialDoc,
			extensions: [
				lineNumbers(),
				highlightActiveLineGutter(),
				highlightActiveLine(),
				languageCompartment.of(yaml()),
				EditorView.updateListener.of((update) => {
					if (update.docChanged) {
						currentContent = update.state.doc.toString();
					}
				}),
				keymap.of([
					{
						key: 'Mod-s',
						run: () => {
							handleSave();
							return true;
						}
					}
				]),
				chiPanelTheme
			]
		});

		const view = new EditorView({
			state,
			parent: editorContainer
		});

		editorView = view;

		return () => {
			view.destroy();
			editorView = null;
		};
	});

	// Synchronize editor content when fileData prop updates
	$effect(() => {
		if (fileData && editorView) {
			const incoming = fileData.content ?? '';
			const currentDoc = editorView.state.doc.toString();
			if (incoming !== currentDoc) {
				editorView.dispatch({
					changes: { from: 0, to: currentDoc.length, insert: incoming }
				});
				currentContent = incoming;
			}
			if (fileData.syntax_mode) {
				selectedSyntax = fileData.syntax_mode.toUpperCase();
			}
		}
	});

	// Dynamic language reconfiguration
	$effect(() => {
		if (!editorView) return;
		let langExt = [];
		if (selectedSyntax === 'YAML') {
			langExt = yaml();
		}
		editorView.dispatch({
			effects: languageCompartment.reconfigure(langExt)
		});
	});

	function handleSave() {
		// `fileData` is only ever set from a successful read, so this is the second of two
		// gates (the first being that the editor is not rendered at all without it).
		if (fileData && !error && !isSaving) {
			onSave(currentContent, triggerReload);
		}
	}

	function getBreadcrumbs(path) {
		if (!path) return ['server-root'];
		return path.split('/').filter(Boolean);
	}

	function formatBytes(bytes) {
		if (!bytes || bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
	}

	let lineCount = $derived.by(() => {
		if (!currentContent) return 0;
		let count = 1;
		for (let i = 0; i < currentContent.length; i++) {
			if (currentContent.charCodeAt(i) === 10) count++;
		}
		return count;
	});
	let charCount = $derived(currentContent ? currentContent.length : 0);
	let breadcrumbs = $derived(fileData ? getBreadcrumbs(fileData.path) : []);
</script>

<div class="config-editor-shell">
	{#if error}
		<!-- Read failed: no editable buffer is opened, so nothing can be written back. -->
		<div class="editor-placeholder-state editor-error-state" role="alert">
			<AlertCircle size={44} class="error-state-icon" />
			<h3>File Could Not Be Read</h3>
			<p class="editor-error-detail">{error}</p>
			<p class="editor-error-note">
				The editor stays closed: no content is shown for a file that could not be read, so
				nothing can be saved over it.
			</p>
			<button type="button" class="btn btn-secondary btn-sm" onclick={onRetry}>
				<RotateCw size={14} />
				<span>Retry</span>
			</button>
		</div>
	{:else if isLoading && !fileData}
		<div class="editor-placeholder-state">
			<RotateCw size={40} class="placeholder-icon spin" />
			<h3>Reading file…</h3>
		</div>
	{:else if !fileData}
		<div class="editor-placeholder-state">
			<FileCode size={48} class="placeholder-icon" />
			<h3>No File Selected</h3>
			<p>Select a configuration file from the directory tree on the left to edit content.</p>
		</div>
	{:else}
		<!-- Header Toolbar -->
		<header class="editor-toolbar">
			<div class="toolbar-left">
				<!-- Path Breadcrumbs -->
				<div class="breadcrumb-trail" aria-label="File location">
					{#each breadcrumbs as crumb, i}
						{#if i > 0}
							<ChevronRight size={14} class="crumb-separator" />
						{/if}
						<span class="crumb-badge {i === breadcrumbs.length - 1 ? 'crumb-active' : ''}">
							{crumb}
						</span>
					{/each}
				</div>

				<!-- File Meta Badges -->
				<span class="badge badge-secondary file-size-badge" title="File Size">
					{formatBytes(fileData.size_bytes || currentContent.length)}
				</span>

				<!-- Dirty Indicator Badge -->
				{#if isDirty}
					<span class="badge badge-warning dirty-badge">
						<span class="status-dot status-dot-warning status-dot-pulse"></span>
						Unsaved Changes
					</span>
				{:else}
					<span class="badge badge-success clean-badge">
						<Check size={12} />
						Saved
					</span>
				{/if}
			</div>

			<div class="toolbar-right">
				<!-- Syntax Mode Selector Badge Dropdown -->
				<div class="syntax-selector-group">
					<SlidersHorizontal size={14} class="syntax-icon" />
					<select
						class="select syntax-select"
						bind:value={selectedSyntax}
						aria-label="Select Editor Syntax Mode"
					>
						{#each syntaxOptions as mode}
							<option value={mode}>{mode}</option>
						{/each}
					</select>
				</div>

				<!-- Reload RCON Checkbox Toggle -->
				<label class="checkbox-group rcon-toggle" title="Trigger RCON reload command on save">
					<input type="checkbox" class="checkbox" bind:checked={triggerReload} />
					<span class="rcon-toggle-label">
						<RotateCw size={13} class="rcon-icon" />
						Reload via RCON
					</span>
				</label>

				<!-- Diff Preview Button -->
				<button
					type="button"
					class="btn btn-secondary btn-sm diff-btn"
					disabled={!isDirty}
					onclick={() => (showDiffModal = true)}
					title="Visualiser le diff avant enregistrement"
				>
					<GitCompare size={14} />
					<span>Diff</span>
				</button>

				<!-- Save Button -->
				<button
					type="button"
					class="btn btn-primary btn-sm save-btn {isSaving ? 'btn-loading' : ''}"
					disabled={isSaving || !isDirty}
					onclick={handleSave}
					title="Enregistrer les modifications (Ctrl+S)"
					aria-label="Enregistrer les modifications"
				>
					<Save size={15} />
					<span>Enregistrer</span>
					<kbd class="shortcut-kbd tabular-nums">Ctrl+S</kbd>
				</button>
			</div>
		</header>

		<ConfigDiffModal
			open={showDiffModal}
			filePath={fileData.path}
			newContent={currentContent}
			onConfirm={() => {
				showDiffModal = false;
				handleSave();
			}}
			onCancel={() => (showDiffModal = false)}
		/>

		<!-- CodeMirror Container -->
		<div class="editor-container-wrapper">
			<div bind:this={editorContainer} class="codemirror-viewport" role="region" aria-label="Code editor"></div>
		</div>

		<!-- Status Footer Bar -->
		<footer class="editor-footer">
			<div class="footer-left">
				<span class="footer-stat">{lineCount} lines</span>
				<span class="footer-divider">•</span>
				<span class="footer-stat">{charCount} characters</span>
			</div>
			<div class="footer-right">
				<span class="footer-syntax">{selectedSyntax}</span>
				<span class="footer-divider">•</span>
				<span class="footer-encoding">UTF-8</span>
			</div>
		</footer>
	{/if}
</div>

<style>
	.config-editor-shell {
		display: flex;
		flex-direction: column;
		width: 100%;
		height: 100%;
		background-color: var(--bg-surface);
		overflow: hidden;
	}

	.editor-placeholder-state {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: var(--space-3);
		padding: var(--space-8);
		text-align: center;
		color: var(--text-muted);
	}

	:global(.placeholder-icon) {
		color: var(--border-focus);
	}

	.editor-placeholder-state h3 {
		font-size: var(--font-size-lg);
		color: var(--text-primary);
	}

	.editor-placeholder-state p {
		font-size: var(--font-size-sm);
		max-width: 400px;
	}

	.editor-error-state h3 {
		color: var(--danger-text);
	}

	:global(.error-state-icon) {
		color: var(--danger-text);
	}

	.editor-error-detail {
		font-family: var(--font-mono);
		font-size: var(--font-size-xs) !important;
		color: var(--text-primary);
		background-color: var(--danger-bg);
		border: 1px solid var(--danger-border);
		border-radius: var(--radius-btn);
		padding: var(--space-3) var(--space-4);
		word-break: break-word;
	}

	.editor-error-note {
		font-size: var(--font-size-xs) !important;
		color: var(--text-muted);
	}

	:global(.placeholder-icon.spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* Toolbar Header */
	.editor-toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-3) var(--space-4);
		background-color: var(--bg-base);
		border-bottom: 1px solid var(--border);
		gap: var(--space-4);
		flex-wrap: wrap;
	}

	.toolbar-left,
	.toolbar-right {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		flex-wrap: wrap;
	}

	.breadcrumb-trail {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
	}

	.crumb-badge {
		color: var(--text-muted);
	}

	.crumb-active {
		color: var(--text-primary);
		font-weight: var(--font-weight-semibold);
	}

	:global(.crumb-separator) {
		color: var(--border-focus);
	}

	.dirty-badge {
		font-family: var(--font-mono);
	}

	.clean-badge {
		font-family: var(--font-mono);
	}

	.syntax-selector-group {
		display: flex;
		align-items: center;
		gap: var(--space-1);
		background-color: var(--bg-surface);
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-input);
		padding-left: var(--space-2);
	}

	:global(.syntax-icon) {
		color: var(--text-muted);
	}

	.syntax-select {
		height: 30px;
		border: none;
		background-color: transparent;
		font-size: var(--font-size-xs);
		font-family: var(--font-mono);
		padding-right: var(--space-6);
	}

	.syntax-select:focus {
		box-shadow: none;
	}

	.rcon-toggle {
		font-size: var(--font-size-xs);
		color: var(--text-secondary);
	}

	.rcon-toggle-label {
		display: flex;
		align-items: center;
		gap: var(--space-1);
	}

	:global(.rcon-icon) {
		color: var(--accent-blue-text);
	}

	.shortcut-kbd {
		background-color: rgba(255, 255, 255, 0.15);
		border-radius: 3px;
		padding: 1px 5px;
		font-size: var(--font-size-xs);
		margin-left: 4px;
		border: none;
	}

	/* CodeMirror Viewport Wrapper */
	.editor-container-wrapper {
		flex: 1;
		position: relative;
		overflow: hidden;
	}

	.codemirror-viewport {
		height: 100%;
		width: 100%;
	}

	:global(.codemirror-viewport .cm-editor) {
		height: 100%;
	}

	/* Footer Bar */
	.editor-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-1) var(--space-4);
		background-color: var(--bg-base);
		border-top: 1px solid var(--border-subtle);
		font-family: var(--font-mono);
		font-size: var(--font-size-xs);
		color: var(--text-muted);
	}

	.footer-left,
	.footer-right {
		display: flex;
		align-items: center;
		gap: var(--space-2);
	}

	.footer-divider {
		color: var(--border-focus);
	}
</style>
