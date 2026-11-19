/*
 * The Gold Ballot: tidy the captured frames and connect them to the variables and text styles.
 * Run foundations.js first. Paste this whole file into the browser console with the file open.
 *
 * Run with APPLY = true on 2026-11-20 against the 47 captured frames: no errors. It is kept
 * here as a dry run: with APPLY = false it changes nothing and only prints a report of what it
 * would do. Run it again after new captures; layers that are already bound or styled are skipped.
 *
 * With APPLY = true it does four things, each of which leaves the look of a frame unchanged:
 *   1. deletes the superseded "Upcoming notice · 390" capture (the one with the misplaced name menu)
 *   2. lays the frames on the Screens page out in rows, one row per screen
 *   3. binds every solid fill and stroke whose colour equals a token to that variable
 *   4. applies the text style whose font, size and weight equal a text layer's
 */
(async () => {
	const APPLY = false;

	const vars = await figma.variables.getLocalVariablesAsync();
	const V = Object.fromEntries(vars.map((v) => [v.name, v]));
	const styles = Object.fromEntries((await figma.getLocalTextStylesAsync()).map((s) => [s.name, s]));
	if (!V['color/bg/page'] || !styles['UI/Body']) {
		console.log('Gold Ballot: variables or text styles are missing. Run foundations.js first.');
		return;
	}

	const hex = (c) =>
		'#' +
		[c.r, c.g, c.b]
			.map((x) => Math.round(x * 255).toString(16).padStart(2, '0'))
			.join('')
			.toUpperCase();
	// colour -> variable, by what the paint is used for
	const TEXT = {
		// pure black is the unpressed toggles: the prototype's .toggle rule set no colour, so the
		// browser default was captured. Binding them corrects them to the ink colour.
		'#000000': 'color/text/default',
		'#151613': 'color/text/default',
		'#5E5D54': 'color/text/muted',
		'#6F5A33': 'color/text/accent',
		'#F5F0E6': 'color/text/inverse',
		'#B8B6A9': 'color/text/inverse-muted',
		'#D6BA7F': 'color/text/brand',
		'#3F5A3A': 'color/text/success',
		'#8C2F26': 'color/text/error',
		'#8A5A00': 'color/text/warning',
		'#3E5C76': 'color/text/info'
	};
	const FILL = {
		'#F5F0E6': 'color/bg/page',
		'#FBF8F1': 'color/bg/raised',
		'#EAE4D7': 'color/bg/tint',
		'#E8DDC4': 'color/bg/selected',
		'#151613': 'color/bg/inverse',
		'#D6BA7F': 'color/bg/brand',
		'#6F5A33': 'color/bg/accent',
		'#D8D1C3': 'color/border/default',
		'#8F8873': 'color/border/control',
		'#57513E': 'color/border/inverse'
	};
	const STROKE = {
		'#D8D1C3': 'color/border/default',
		'#8F8873': 'color/border/control',
		'#151613': 'color/border/strong',
		'#6F5A33': 'color/border/accent',
		'#57513E': 'color/border/inverse',
		'#D6BA7F': 'color/border/brand',
		'#8C2F26': 'color/text/error',
		'#3F5A3A': 'color/text/success',
		'#3E5C76': 'color/text/info'
	};
	const SHAPES = new Set(['VECTOR', 'BOOLEAN_OPERATION', 'ELLIPSE', 'LINE', 'STAR', 'POLYGON']);

	const bump = (o, k) => (o[k] = (o[k] || 0) + 1);
	const top = (o, n = 12) =>
		Object.entries(o)
			.sort((a, b) => b[1] - a[1])
			.slice(0, n);

	const bindPaints = (node, prop, map, kind, st) => {
		const paints = node[prop];
		if (!Array.isArray(paints) || !paints.length) return;
		let changed = false;
		const next = paints.map((p) => {
			if (p.type !== 'SOLID' || p.visible === false) return p;
			if (p.boundVariables && p.boundVariables.color) {
				st.alreadyBound++;
				return p;
			}
			const h = hex(p.color);
			const name = map[h];
			if (!name || (p.opacity !== undefined && p.opacity < 0.999)) {
				bump(st.unmatchedColours, `${kind} ${h}${p.opacity < 0.999 ? ' @' + p.opacity.toFixed(2) : ''}`);
				return p;
			}
			bump(st.bound, name);
			changed = true;
			return APPLY ? figma.variables.setBoundVariableForPaint(p, 'color', V[name]) : p;
		});
		if (APPLY && changed) node[prop] = next;
	};

	const styleFor = (n) => {
		if (n.fontName === figma.mixed || n.fontSize === figma.mixed || n.letterSpacing === figma.mixed) return null;
		const size = n.fontSize;
		const ls = n.letterSpacing.unit === 'PERCENT' ? n.letterSpacing.value : (n.letterSpacing.value / size) * 100;
		if (n.fontName.family === 'Bodoni Moda')
			return {
				240: 'Display/Numeral XL',
				72: 'Display/Numeral L',
				60: 'Display/Headline XL',
				48: 'Display/Headline L',
				36: 'Display/Headline M',
				30: 'Display/Headline S',
				24: 'Display/Title',
				20: 'Display/Title S'
			}[size];
		if (n.fontName.family === 'Inter') {
			const medium = /Medium/.test(n.fontName.style);
			if (size === 16) return medium ? (ls > 12 ? 'UI/Wordmark' : 'UI/Body L Strong') : 'UI/Body L';
			if (size === 14) return medium ? 'UI/Label' : 'UI/Body';
			if (size === 12) return medium ? (ls >= 8 ? 'UI/Eyebrow' : null) : 'UI/Caption';
		}
		return null;
	};

	const report = { apply: APPLY, pages: {} };

	for (const pageName of ['Screens', 'Design system']) {
		const page = figma.root.children.find((p) => p.name === pageName);
		if (!page) {
			report.pages[pageName] = 'page not found';
			continue;
		}
		await page.loadAsync();
		const st = { bound: {}, alreadyBound: 0, unmatchedColours: {}, textStyled: {}, textAlreadyStyled: 0, textUnmatched: {}, textMixed: 0, errors: [] };
		const r = (report.pages[pageName] = {
			topLevel: page.children.map((n) => `${n.id} ${n.type} "${n.name}" ${Math.round(n.width)}x${Math.round(n.height)} @${Math.round(n.x)},${Math.round(n.y)}`)
		});

		if (pageName === 'Screens') {
			// 1. the superseded capture
			const dup = page.children.filter((n) => /Upcoming notice · 390$/.test(n.name));
			if (dup.length === 2 && dup.some((n) => n.id === '28:2')) {
				r.deleted = '28:2 ' + dup.find((n) => n.id === '28:2').name;
				if (APPLY) dup.find((n) => n.id === '28:2').remove();
			} else r.deleted = `nothing (found ${dup.length} frames named "Upcoming notice · 390")`;

			// 2. rows: one per screen; in a row the default 1440, 768 and 390 come first, then variants
			const frames = page.children.filter((n) => !n.removed && / · \d+$/.test(n.name));
			const seq = (n) => Number(n.id.split(':')[0]);
			const groups = new Map();
			for (const f of frames) {
				const key = f.name.replace(/ \([^)]*\)/, '').replace(/ · \d+$/, '');
				if (!groups.has(key)) groups.set(key, []);
				groups.get(key).push(f);
			}
			const rows = [...groups.entries()].sort((a, b) => Math.min(...a[1].map(seq)) - Math.min(...b[1].map(seq)));
			let y = 0;
			r.rows = [];
			for (const [key, fs] of rows) {
				fs.sort((a, b) => /\(/.test(a.name) - /\(/.test(b.name) || b.width - a.width || seq(a) - seq(b));
				let x = 0;
				for (const f of fs) {
					if (APPLY) {
						f.x = x;
						f.y = y;
					}
					x += f.width + 160;
				}
				r.rows.push(`${key}: ${fs.length} frame(s)`);
				y += Math.max(...fs.map((f) => f.height)) + 320;
			}
			r.notArranged = page.children.filter((n) => !n.removed && !/ · \d+$/.test(n.name)).map((n) => n.name);
		}

		// 3 and 4. colours and text styles
		const nodes = page.findAll(() => true).filter((n) => !n.removed);
		const texts = nodes.filter((n) => n.type === 'TEXT');
		if (APPLY) {
			const fonts = new Map();
			for (const t of texts)
				for (const seg of t.getStyledTextSegments(['fontName'])) fonts.set(seg.fontName.family + '|' + seg.fontName.style, seg.fontName);
			for (const f of fonts.values()) await figma.loadFontAsync(f);
		}
		for (const n of nodes) {
			try {
				await bindNode(n, st);
			} catch (e) {
				st.errors.push(`${n.id} ${n.type} "${n.name}": ${e.message}`);
			}
		}
		Object.assign(r, {
			errorCount: st.errors.length,
			errors: st.errors.slice(0, 10),
			nodes: nodes.length,
			textLayers: texts.length,
			coloursBound: top(st.bound, 30),
			coloursAlreadyBound: st.alreadyBound,
			coloursUnmatched: top(st.unmatchedColours, 20),
			textStylesApplied: top(st.textStyled, 20),
			textAlreadyStyled: st.textAlreadyStyled,
			textUnmatched: top(st.textUnmatched, 20),
			textMixed: st.textMixed
		});
	}

	console.log('Gold Ballot tidy-and-bind report (copy everything below)\n' + JSON.stringify(report, null, 1));
	return report;

	async function bindNode(n, st) {
		{
			if (n.type === 'TEXT') {
				if (n.fills === figma.mixed) st.textMixed++;
				else bindPaints(n, 'fills', TEXT, 'text', st);
				if (n.textStyleId && n.textStyleId !== figma.mixed) {
					st.textAlreadyStyled++;
					return;
				}
				const name = styleFor(n);
				if (name && styles[name]) {
					bump(st.textStyled, name);
					if (APPLY) await n.setTextStyleIdAsync(styles[name].id);
				} else if (n.fontName !== figma.mixed && n.fontSize !== figma.mixed) {
					const ls = n.letterSpacing === figma.mixed ? 'mixed' : Math.round(n.letterSpacing.value * 10) / 10 + (n.letterSpacing.unit === 'PERCENT' ? '%' : 'px');
					bump(st.textUnmatched, `${n.fontName.family} ${n.fontName.style} ${n.fontSize} ls ${ls}`);
				} else st.textMixed++;
			} else if (SHAPES.has(n.type)) {
				if ('fills' in n && n.fills !== figma.mixed) bindPaints(n, 'fills', TEXT, 'icon-fill', st);
				if ('strokes' in n) bindPaints(n, 'strokes', TEXT, 'icon-stroke', st);
			} else {
				if ('fills' in n && n.fills !== figma.mixed) bindPaints(n, 'fills', FILL, 'fill', st);
				if ('strokes' in n) bindPaints(n, 'strokes', STROKE, 'stroke', st);
			}
		}
	}
})();
