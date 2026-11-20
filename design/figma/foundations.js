/*
 * The Gold Ballot: Figma foundations.
 * Creates the variable collections (Primitives, Theme, Layout, Typography) and the text styles
 * that design/prototype/styles.css mirrors. Safe to run twice: it updates what already exists.
 *
 * Run on 2026-11-20 from the browser console (the Figma MCP quota was used up): it created
 * 4 collections and 16 text styles with no errors. To run it again, open the file in Figma,
 * open the browser console, and paste the whole file.
 */
(async () => {
	const hex = (h, a) => {
		const n = parseInt(h.slice(1), 16);
		const c = { r: ((n >> 16) & 255) / 255, g: ((n >> 8) & 255) / 255, b: (n & 255) / 255 };
		if (a !== undefined) c.a = a;
		return c;
	};
	const existingCols = await figma.variables.getLocalVariableCollectionsAsync();
	const existingVars = await figma.variables.getLocalVariablesAsync();
	const col = (name, mode) => {
		let c = existingCols.find((x) => x.name === name);
		if (!c) {
			c = figma.variables.createVariableCollection(name);
			c.renameMode(c.modes[0].modeId, mode);
		}
		return c;
	};
	const made = {};
	const mk = (c, name, type, value, scopes, syntax, desc) => {
		let v = existingVars.find((x) => x.name === name && x.variableCollectionId === c.id);
		if (!v) v = figma.variables.createVariable(name, c, type);
		v.setValueForMode(c.modes[0].modeId, value);
		v.scopes = scopes;
		if (syntax) v.setVariableCodeSyntax('WEB', syntax);
		if (desc) v.description = desc;
		made[name] = v;
		return v;
	};
	const alias = (n) => figma.variables.createVariableAlias(made[n]);

	// ---------- Primitives: raw values, hidden from the pickers ----------
	const P = col('Primitives', 'Value');
	const prim = {
		'ink/900': '#151613',
		'ink/700': '#3A3A33',
		'ink/500': '#5E5D54',
		'ink/300': '#B8B6A9',
		'ivory/50': '#FBF8F1',
		'ivory/100': '#F5F0E6',
		'ivory/200': '#EAE4D7',
		'ivory/300': '#D8D1C3',
		'ivory/500': '#8F8873',
		'gold/100': '#E8DDC4',
		'gold/300': '#D6BA7F',
		'gold/700': '#6F5A33',
		'gold/800': '#57513E',
		'green/600': '#3F5A3A',
		'red/600': '#8C2F26',
		'amber/600': '#805300',
		'blue/600': '#3E5C76'
	};
	for (const [n, h] of Object.entries(prim)) mk(P, n, 'COLOR', hex(h), [], null, h);
	mk(P, 'ink/900-60', 'COLOR', hex('#151613', 0.6), [], null, '#151613 at 60%');

	// ---------- Theme: semantic color, one mode (spec rule 4.7.6) ----------
	const T = col('Theme', 'Gold Ballot');
	const BG = ['FRAME_FILL', 'SHAPE_FILL'];
	const TX = ['TEXT_FILL', 'SHAPE_FILL', 'STROKE_COLOR'];
	const BD = ['STROKE_COLOR', 'FRAME_FILL', 'SHAPE_FILL'];
	const sem = [
		['color/bg/page', 'ivory/100', BG, 'var(--color-base-200)', 'Page background.'],
		['color/bg/raised', 'ivory/50', BG, 'var(--color-base-100)', 'Cards, table rows, inputs: the lightest surface.'],
		['color/bg/tint', 'ivory/200', BG, 'var(--color-base-300)', 'Tinted panels: sidebar, guidance panel, table header.'],
		['color/bg/selected', 'gold/100', BG, 'var(--gb-selected)', 'Selected row or pressed "Should win". Custom theme color.'],
		['color/bg/inverse', 'ink/900', BG, 'var(--color-neutral)', 'Dark surfaces: navigation, dark banner, primary button.'],
		['color/bg/brand', 'gold/300', BG, 'var(--color-primary)', 'Champagne gold fill. With dark text only.'],
		['color/bg/accent', 'gold/700', BG, 'var(--color-accent)', 'Bronze fill for progress and statistic bars on light surfaces.'],
		['color/bg/scrim', 'ink/900-60', BG, 'rgb(21 22 19 / 0.6)', 'Overlay behind a dialog.'],
		['color/text/default', 'ink/900', TX, 'var(--color-base-content)', 'Body text and icons on light surfaces. 13:1 or better.'],
		['color/text/muted', 'ink/500', TX, 'var(--gb-muted)', 'Secondary text on light surfaces. 4.9:1 or better on every light surface. Custom theme color.'],
		['color/text/accent', 'gold/700', TX, 'var(--color-accent)', 'Gold text on light surfaces: eyebrows, favorites. 4.9:1 or better.'],
		['color/text/inverse', 'ivory/100', TX, 'var(--color-neutral-content)', 'Text on dark surfaces.'],
		['color/text/inverse-muted', 'ink/300', TX, 'var(--gb-neutral-muted)', 'Secondary text on dark surfaces. 8.9:1. Custom theme color.'],
		['color/text/brand', 'gold/300', TX, 'var(--color-primary)', 'Gold text on dark surfaces only. Fails contrast on ivory.'],
		['color/text/success', 'green/600', TX, 'var(--color-success)', 'Correct, saved. Always with text or an icon (spec rule 6.3.3).'],
		['color/text/error', 'red/600', TX, 'var(--color-error)', 'Missed, failed, destructive. Always with text or an icon.'],
		['color/text/warning', 'amber/600', TX, 'var(--color-warning)', 'Caution notices.'],
		['color/text/info', 'blue/600', TX, 'var(--color-info)', 'Neutral notices.'],
		['color/border/default', 'ivory/300', BD, 'var(--gb-line)', 'Thin rules and card outlines. Decorative. Custom theme color.'],
		['color/border/control', 'ivory/500', BD, 'var(--gb-line-control)', 'Outline of inputs, outlined buttons and toggles. 3:1 on light surfaces. Custom theme color.'],
		['color/border/strong', 'ink/900', BD, 'var(--color-base-content)', 'Focus ring, active tab underline.'],
		['color/border/accent', 'gold/700', BD, 'var(--color-accent)', 'Outline of a selected row or pressed toggle.'],
		['color/border/inverse', 'gold/800', BD, 'var(--gb-neutral-line)', 'Rules on dark surfaces. Custom theme color.'],
		['color/border/brand', 'gold/300', BD, 'var(--color-primary)', 'Gold rule on dark surfaces: active navigation link.']
	];
	for (const [n, p, sc, syn, d] of sem) mk(T, n, 'COLOR', alias(p), sc, syn, d);

	// ---------- Layout: spacing scale, canonical paddings, sizes, radius, border width ----------
	const L = col('Layout', 'Value');
	for (const s of [0, 1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24])
		mk(L, 'space/' + s, 'FLOAT', s * 4, ['GAP'], `calc(var(--spacing) * ${s})`, `${s * 4}px. Tailwind step ${s} (p-${s}, gap-${s}).`);
	const lay = [
		['gutter/desktop', 'space/12', 'Left and right page padding at 1440.'],
		['gutter/tablet', 'space/8', 'Left and right page padding at 768.'],
		['gutter/mobile', 'space/6', 'Left and right page padding at 390.'],
		['stack/page', 'space/8', 'Gap between the major regions of a page (space/6 on a phone).'],
		['stack/section', 'space/6', 'Gap between blocks inside a region.'],
		['stack/item', 'space/3', 'Gap between repeated items: nominee rows, fields.'],
		['stack/text', 'space/2', 'Gap between lines of related text.'],
		['inset/panel', 'space/6', 'Padding of cards, panels and the dark banner.'],
		['inset/cell-x', 'space/4', 'Left and right padding of a table cell or list row.'],
		['inset/cell-y', 'space/4', 'Top and bottom padding of a table row.'],
		['inset/control-x', 'space/4', 'Left and right padding inside an input.'],
		['inset/button-x', 'space/6', 'Left and right padding inside a button.']
	];
	for (const [n, p, d] of lay) mk(L, n, 'FLOAT', alias(p), ['GAP'], null, d);
	const sizes = [
		['size/control', 48, 'Height of buttons, inputs and toggles. Above the 44px touch minimum.'],
		['size/control-lg', 56, 'Height of the single main action on a page.'],
		['size/nav', 80, 'Height of the navigation bar at desktop width.'],
		['size/nav-mobile', 56, 'Height of the first navigation row on a phone; the link row is 48.'],
		['size/sidebar', 288, 'Width of the category list at desktop width (w-72).'],
		['size/panel', 344, 'Width of the guidance panel and numeral block at desktop width (w-86).'],
		['size/form', 416, 'Width of the form column in the two-panel layout (w-104).'],
		['size/icon', 16, 'Icon box.']
	];
	for (const [n, v, d] of sizes) mk(L, n, 'FLOAT', v, ['WIDTH_HEIGHT'], null, d);
	mk(L, 'radius/none', 'FLOAT', 0, ['CORNER_RADIUS'], 'var(--radius-box)', 'Square corners everywhere (spec rule 4.7.1).');
	mk(L, 'radius/full', 'FLOAT', 999, ['CORNER_RADIUS'], '9999px', 'Radio dots only.');
	mk(L, 'border-width/hairline', 'FLOAT', 1, ['STROKE_FLOAT'], 'var(--border)', 'Every rule and outline.');
	mk(L, 'border-width/emphasis', 'FLOAT', 2, ['STROKE_FLOAT'], '2px', 'Active tab underline, progress bar, focus ring.');

	// ---------- Typography ----------
	const Y = col('Typography', 'Value');
	const fDisplay = mk(Y, 'font/family/display', 'STRING', 'Bodoni Moda', ['FONT_FAMILY'], 'var(--font-display)', 'Display serif. SIL Open Font License; self-host (spec rule 4.7.2).');
	const fUi = mk(Y, 'font/family/ui', 'STRING', 'Inter', ['FONT_FAMILY'], 'var(--font-sans)', 'UI sans. SIL Open Font License; self-host (spec rule 4.7.2).');

	await Promise.all(
		[
			['Bodoni Moda', 'Regular'],
			['Inter', 'Regular'],
			['Inter', 'Medium']
		].map(([family, style]) => figma.loadFontAsync({ family, style }))
	);
	const existingStyles = await figma.getLocalTextStylesAsync();
	const bindErrors = [];
	let styleCount = 0;
	const ts = (name, family, style, size, lh, opts = {}) => {
		const s = existingStyles.find((x) => x.name === name) || figma.createTextStyle();
		s.name = name;
		s.fontName = { family, style };
		s.fontSize = size;
		s.lineHeight = { unit: 'PIXELS', value: lh };
		s.letterSpacing = { unit: 'PERCENT', value: opts.ls || 0 };
		s.textCase = opts.upper ? 'UPPER' : 'ORIGINAL';
		s.description = opts.desc || '';
		try {
			s.setBoundVariable('fontFamily', family === 'Inter' ? fUi : fDisplay);
		} catch (e) {
			bindErrors.push(name + ': ' + e.message);
		}
		styleCount++;
	};
	const B = 'Bodoni Moda';
	const I = 'Inter';
	ts('Display/Numeral XL', B, 'Regular', 240, 240, { desc: 'The ceremony numeral in the two-panel layout. text-[15rem] leading-none' });
	ts('Display/Numeral L', B, 'Regular', 72, 72, { desc: 'Numeral block figure. text-7xl leading-none' });
	ts('Display/Headline XL', B, 'Regular', 60, 66, { desc: 'Two-panel headline at desktop width. text-6xl' });
	ts('Display/Headline L', B, 'Regular', 48, 54, { desc: 'Page headline at desktop width. text-5xl' });
	ts('Display/Headline M', B, 'Regular', 36, 42, { desc: 'Page headline on a phone. text-4xl' });
	ts('Display/Headline S', B, 'Regular', 30, 36, { desc: 'Guidance panel, section and dialog heading. text-3xl' });
	ts('Display/Title', B, 'Regular', 24, 30, { desc: 'Nominee names, ranks, section titles. text-2xl' });
	ts('Display/Title S', B, 'Regular', 20, 26, { desc: 'Small serif heading. text-xl' });
	ts('UI/Body L', I, 'Regular', 16, 24, { desc: 'Subtitles and lead text. text-base' });
	ts('UI/Body L Strong', I, 'Medium', 16, 24, { desc: 'Primary value in a table row. text-base font-medium' });
	ts('UI/Body', I, 'Regular', 14, 20, { desc: 'Default text. text-sm' });
	ts('UI/Body Strong', I, 'Medium', 14, 20, { desc: 'Emphasis inside body text. text-sm font-medium' });
	ts('UI/Label', I, 'Medium', 14, 20, { desc: 'Buttons, navigation, tabs, field labels. text-sm font-medium' });
	ts('UI/Caption', I, 'Regular', 12, 16, { desc: 'Helper text and footnotes. The smallest size allowed. text-xs' });
	ts('UI/Eyebrow', I, 'Medium', 12, 16, { ls: 10, upper: true, desc: 'Eyebrows and table headers. text-xs font-medium uppercase tracking-widest' });
	ts('UI/Wordmark', I, 'Medium', 16, 20, { ls: 14, upper: true, desc: 'THE GOLD BALLOT wordmark. text-base font-medium uppercase tracking-[0.14em]' });

	const after = await figma.variables.getLocalVariableCollectionsAsync();
	const result = {
		collections: after.map((c) => `${c.name}: ${c.variableIds.length} variables`),
		textStyles: styleCount,
		bindErrors
	};
	console.log('Gold Ballot foundations', result);
	return result;
})();
