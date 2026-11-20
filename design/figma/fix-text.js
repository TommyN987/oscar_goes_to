/*
 * The Gold Ballot: correct text in the captured frames after three decisions made on 2026-11-20.
 *   1. American spelling throughout ("Color", "canceled").
 *   2. The colors daisyUI has no role for are --gb-* variables in the theme, not --color-*.
 *   3. The warning amber is #805300 (was #8A5A00); the swatch label on the sheet is corrected.
 * It replaces only the exact words listed in PAIRS and keeps each layer's styling.
 * Safe to run twice: the second run finds nothing to replace.
 * Paste the whole file into the browser console with the Figma file open.
 */
(async () => {
	const PAIRS = [
		['Colour', 'Color'],
		['colour', 'color'],
		['COLOUR', 'COLOR'],
		['Cancelled', 'Canceled'],
		['cancelled', 'canceled'],
		['CANCELLED', 'CANCELED'],
		['--color-line-control', '--gb-line-control'],
		['--color-neutral-muted', '--gb-neutral-muted'],
		['--color-neutral-line', '--gb-neutral-line'],
		['--color-muted', '--gb-muted'],
		['--color-selected', '--gb-selected'],
		['--color-line', '--gb-line'],
		// the warning amber was darkened so it also reaches 4.5:1 on a selected row
		['#8A5A00', '#805300'],
		['5.2 : 1', '5.9 : 1']
	];
	const report = {};
	for (const pageName of ['Screens', 'Design system']) {
		const page = figma.root.children.find((p) => p.name === pageName);
		if (!page) {
			report[pageName] = 'page not found';
			continue;
		}
		await page.loadAsync();
		const counts = {};
		const errors = [];
		let layers = 0;
		for (const t of page.findAllWithCriteria({ types: ['TEXT'] })) {
			if (!PAIRS.some(([from]) => t.characters.includes(from))) continue;
			try {
				for (const seg of t.getStyledTextSegments(['fontName'])) await figma.loadFontAsync(seg.fontName);
				for (const [from, to] of PAIRS) {
					let i = t.characters.indexOf(from);
					while (i !== -1) {
						// insert the new word in the style of the old one, then remove the old one
						t.insertCharacters(i, to, 'AFTER');
						t.deleteCharacters(i + to.length, i + to.length + from.length);
						counts[`${from} -> ${to}`] = (counts[`${from} -> ${to}`] || 0) + 1;
						i = t.characters.indexOf(from, i + to.length);
					}
				}
				layers++;
			} catch (e) {
				errors.push(`${t.id} "${t.characters.slice(0, 40)}": ${e.message}`);
			}
		}
		report[pageName] = { layersChanged: layers, replaced: counts, errorCount: errors.length, errors: errors.slice(0, 10) };
	}
	console.log('Gold Ballot fix-text report (copy everything below)\n' + JSON.stringify(report, null, 1));
	return report;
})();
