/*
 * State boards and the design-system sheet. They reuse the patterns from app.js,
 * so a change to a pattern shows up here and on every screen.
 */
const cell = (label, html, cls = '') =>
	`<div class="board__cell"><div class="t-eyebrow muted">${label}</div><div class="frame ${cls}">${html}</div></div>`;
const boardHead = (eyebrow, h1, sub) =>
	`<div class="head__main"><div class="t-eyebrow accent">${eyebrow}</div><h1 class="t-h-l">${h1}</h1><p class="t-body-l muted">${sub}</p></div>`;
const section = (title, sub, body) =>
	`<section class="stack"><div class="stack stack--text"><h2 class="t-h-s">${title}</h2>${sub ? `<p class="muted">${sub}</p>` : ''}</div>${body}</section>`;

/* ---------- sign-in and name-step states (4.8.1.4, 4.8.2.3, 4.8.9.4, 4.8.10.2–4) ---------- */
const signinStates = () =>
	`<div class="board">${boardHead('Spec 4.8.1 · 4.8.2 · 4.8.10', 'Sign-in and name step: states', 'The form panel of the two-panel layout in each state. Messages sit above the button.')}<div class="board__grid">${[
		['Default · 4.8.1.2', signinActions()],
		['After a press · 4.8.1.4', signinActions({ pending: true })],
		[
			'Sign-in canceled · 4.8.10.2',
			signinActions({ notice: notice('info', 'Sign-in was canceled. Try again when you’re ready.') })
		],
		[
			'Google error · 4.8.10.2',
			signinActions({ notice: notice('error', 'Google couldn’t sign you in. Try again in a moment.') })
		],
		[
			'Unverified email · 4.8.10.2',
			signinActions({
				notice: notice('error', 'Your Google email isn’t verified. Verify it with Google, then try again.')
			})
		],
		[
			'Session expired · 4.8.10.4',
			signinActions({
				notice: notice('info', 'You’ve been signed out. Sign in to pick up where you left off.')
			})
		],
		['Account deleted · 4.8.9.4', signinActions({ notice: notice('success', 'Your account is deleted.') })],
		[
			'Name taken · 4.8.2.3',
			`<div class="stack">${nameForm('That name is taken. Try another.')}</div>`
		],
		['Blocked account · 4.8.10.3', `<div class="stack">${blockedForm().replace('t-h-l', 't-h-m')}</div>`]
	]
		.map(([l, h]) => cell(l, h))
		.join('')}</div></div>`;

/* ---------- confirmation dialogs (4.8.7.3, 4.8.7.4, 4.8.8.2, 4.8.9.4) ---------- */
const dialog = (eyebrow, title, body, confirm, danger) =>
	`<div class="scrim"><div class="dialog" role="dialog"><div class="t-eyebrow accent">${eyebrow}</div><h2 class="t-h-s">${title}</h2>${body}<div class="dialog__actions"><button class="btn">Back</button><button class="btn ${danger ? 'btn--danger' : 'btn--primary'}">${confirm}</button></div></div></div>`;
const winnerLine = (n, s) =>
	`<div class="card stack stack--text" style="gap:2px;padding:16px"><span class="t-body-l-strong">${n}</span><span class="t-caption muted">${s}</span></div>`;
const DIALOGS = [
	[
		'Record a result · 4.8.7.3',
		dialog(
			'Cinematography',
			'Record this winner?',
			`${winnerLine('Sinners', 'Autumn Durald Arkapaw')}<p class="muted">Scores update for every player as soon as you confirm.</p>`,
			'Confirm'
		)
	],
	[
		'Record a tie · 4.8.7.3',
		dialog(
			'Sound',
			'Record these 2 winners?',
			`${winnerLine('F1', 'Sound team')}${winnerLine('Sinners', 'Sound team')}<p class="muted">A tie: a prediction for either one scores.</p>`,
			'Confirm'
		)
	],
	[
		'Correct a result · 4.8.7.4',
		dialog(
			'Actor in a supporting role',
			'Correct this result?',
			`${winnerLine('Delroy Lindo', 'Sinners')}<p class="muted">This replaces Sean Penn as the winner. Scores will change.</p>`,
			'Confirm'
		)
	],
	[
		'Clear a result · 4.8.7.4',
		dialog(
			'Actor in a supporting role',
			'Clear this result?',
			'<p class="t-body-l">The category goes back to “To announce”. Scores will change.</p>',
			'Clear result',
			true
		)
	],
	[
		'Remove a player · 4.8.8.2',
		dialog(
			'Admin / Players',
			'Remove Reel Talk?',
			'<p class="t-body-l">Their predictions are deleted and this Google account is blocked.</p>',
			'Remove player',
			true
		)
	],
	[
		'Delete my account · 4.8.9.4',
		dialog(
			'Leaving the game',
			'Delete your account?',
			'<p class="t-body-l">Your account, your predictions and your favorites are removed. This cannot be undone.</p>',
			'Yes, delete everything',
			true
		)
	]
];
const dialogs = () =>
	`<div class="board">${boardHead('Spec 4.8.7 · 4.8.8 · 4.8.9', 'Confirmation dialogs', 'One dialog pattern: eyebrow naming the subject, serif question, what will happen, Back and one confirming action. Shown over the scrim.')}<div class="board__grid board__grid--2">${DIALOGS.map(([l, h]) => cell(l, h, 'frame--flush')).join('')}</div></div>`;

/* ---------- design system ---------- */
const SWATCHES = {
	Surfaces: [
		['color/bg/page', '--color-base-200', '#F5F0E6', 'Page background'],
		['color/bg/raised', '--color-base-100', '#FBF8F1', 'Cards, rows, inputs'],
		['color/bg/tint', '--color-base-300', '#EAE4D7', 'Sidebar, guidance panel, table header'],
		['color/bg/selected', '--gb-selected', '#E8DDC4', 'Selected row, pressed “Should win”'],
		['color/bg/inverse', '--color-neutral', '#151613', 'Navigation, dark banner, primary button'],
		['color/bg/brand', '--color-primary', '#D6BA7F', 'Gold fill. With dark text only'],
		['color/bg/accent', '--color-accent', '#6F5A33', 'Progress and statistic bars']
	],
	'Borders and rules': [
		['color/border/default', '--gb-line', '#D8D1C3', 'Thin rules, card outlines'],
		['color/border/control', '--gb-line-control', '#8F8873', 'Inputs, outlined buttons, toggles · 3:1'],
		['color/border/strong', '--color-base-content', '#151613', 'Focus ring, active tab'],
		['color/border/accent', '--color-accent', '#6F5A33', 'Selected row, pressed toggle'],
		['color/border/inverse', '--gb-neutral-line', '#57513E', 'Rules on dark surfaces']
	]
};
const TEXTS = [
	['color/text/default', '--color-base-content', '#151613', 'base-200', '16.0 : 1'],
	['color/text/muted', '--gb-muted', '#5E5D54', 'base-200', '5.8 : 1 · 4.9 on selected'],
	['color/text/accent', '--color-accent', '#6F5A33', 'base-200', '5.8 : 1 · 4.9 on selected'],
	['color/text/success', '--color-success', '#3F5A3A', 'base-200', '6.8 : 1'],
	['color/text/error', '--color-error', '#8C2F26', 'base-200', '7.3 : 1'],
	['color/text/warning', '--color-warning', '#805300', 'base-200', '5.9 : 1'],
	['color/text/info', '--color-info', '#3E5C76', 'base-200', '6.2 : 1'],
	['color/text/inverse', '--color-neutral-content', '#F5F0E6', 'neutral', '16.0 : 1'],
	['color/text/inverse-muted', '--gb-neutral-muted', '#B8B6A9', 'neutral', '8.9 : 1'],
	['color/text/brand', '--color-primary', '#D6BA7F', 'neutral', '9.7 : 1 · dark surfaces only']
];
const TYPE = [
	['Display/Numeral XL', 't-numeral-xl', 'Bodoni Moda 240 / 240', '98'],
	['Display/Numeral L', 't-numeral-l', 'Bodoni Moda 72 / 72 · text-7xl', '08'],
	['Display/Headline XL', 't-h-xl', 'Bodoni Moda 60 / 66 · text-6xl', 'Great films.'],
	['Display/Headline L', 't-h-l', 'Bodoni Moda 48 / 54 · text-5xl', 'The ballot is yours.'],
	['Display/Headline M', 't-h-m', 'Bodoni Moda 36 / 42 · text-4xl', 'Your ballot is waiting.'],
	['Display/Headline S', 't-h-s', 'Bodoni Moda 30 / 36 · text-3xl', 'Still time to change your mind.'],
	['Display/Title', 't-title', 'Bodoni Moda 24 / 30 · text-2xl', 'Benicio Del Toro'],
	['Display/Title S', 't-title-s', 'Bodoni Moda 20 / 26 · text-xl', '16 categories to go'],
	['UI/Body L', 't-body-l', 'Inter Regular 16 / 24 · text-base', 'Make your predictions. Follow your favorites.'],
	['UI/Body L Strong', 't-body-l-strong', 'Inter Medium 16 / 24', 'One Battle after Another'],
	['UI/Body', '', 'Inter Regular 14 / 20 · text-sm', 'Choose who you think will win in each category.'],
	['UI/Body Strong · UI/Label', 't-strong', 'Inter Medium 14 / 20', 'Review my ballot'],
	['UI/Caption', 't-caption', 'Inter Regular 12 / 16 · text-xs · smallest size', '8 predictions saved · 3 favorites'],
	['UI/Eyebrow', 't-eyebrow', 'Inter Medium 12 / 16 · 10% tracking · caps', 'Your seat at awards night'],
	['UI/Wordmark', 't-wordmark', 'Inter Medium 16 / 20 · 14% tracking · caps', 'The Gold Ballot']
];
const SPACE = [1, 2, 3, 4, 5, 6, 8, 10, 12, 16, 20, 24];
const RULES = [
	['gutter/desktop · tablet · mobile', '48 · 32 · 24', 'Left and right page padding at 1440, 768 and 390'],
	['stack/page', '32', 'Between the major regions of a page (24 on a phone)'],
	['stack/section', '24', 'Between blocks inside a region'],
	['stack/item', '12', 'Between repeated items: nominee rows, fields'],
	['stack/text', '8', 'Between lines of related text'],
	['inset/panel', '24', 'Padding of cards, panels and the dark banner'],
	['inset/cell-x · inset/cell-y', '16 · 16', 'Padding of table cells and list rows'],
	['inset/button-x · inset/control-x', '24 · 16', 'Inside buttons · inside inputs'],
	['size/control · size/control-lg', '48 · 56', 'Height of controls · of the one main action'],
	['size/nav · size/nav-mobile', '80 · 56', 'Navigation bar height'],
	['size/sidebar · size/panel · size/form', '288 · 344 · 416', 'Category list · guidance panel · form column'],
	['radius/none', '0', 'Square corners everywhere; radio dots are the only circles'],
	['border-width/hairline · emphasis', '1 · 2', 'Rules and outlines · active tab, progress, focus']
];
const swatch = ([name, v, hex, note]) =>
	`<div class="swatch"><div class="swatch__chip" style="background:${hex}"></div><div class="t-strong">${name}</div><div class="t-caption muted">${hex} · ${v}</div><div class="t-caption muted">${note}</div></div>`;
const system = () => {
	const color =
		Object.entries(SWATCHES)
			.map(
				([g, items]) =>
					`<div class="stack stack--item"><div class="t-eyebrow muted">${g}</div><div class="board__grid board__grid--4">${items.map(swatch).join('')}</div></div>`
			)
			.join('') +
		`<div class="stack stack--item"><div class="t-eyebrow muted">Text and icons, on the surface they are meant for, with contrast ratio</div><div class="board__grid board__grid--2" style="gap:0 32px">${TEXTS.map(
			([name, v, hex, bg, ratio]) =>
				`<div class="spec" style="grid-template-columns:minmax(0,1fr) 200px;align-items:center"><div class="stack stack--text" style="gap:2px"><span class="t-strong">${name}</span><span class="t-caption muted">${hex} · ${v} · ${ratio}</span></div><div style="background:var(--color-${bg});color:${hex};padding:12px 16px" class="t-body-l-strong">The ballot is yours.</div></div>`
		).join('')}</div></div>`;
	const type = TYPE.map(
		([name, cls, meta, sample]) =>
			`<div class="spec"><div class="stack stack--text" style="gap:2px"><span class="t-strong">${name}</span><span class="t-caption muted">${meta}</span></div><div class="${cls}">${sample}</div></div>`
	).join('');
	const space = `<div class="board__grid board__grid--2"><div class="stack stack--text">${SPACE.map(
		(s) =>
			`<div class="row"><span class="t-strong" style="width:88px">space/${s}</span><span class="t-caption muted" style="width:48px">${s * 4}px</span><div class="spacebar" style="width:${s * 4}px"></div></div>`
	).join(
		''
	)}</div><div>${RULES.map(([n, v, d]) => `<div class="spec" style="grid-template-columns:minmax(0,1fr) 200px;padding:10px 0"><div class="stack stack--text" style="gap:2px"><span class="t-strong">${n}</span><span class="t-caption muted">${d}</span></div><span class="t-title" style="text-align:right">${v}</span></div>`).join('')}</div></div>`;
	const menu = `<div class="menu"><div class="menu__item">Settings</div><div class="menu__item">Admin <span class="t-caption muted">admins only</span></div><div class="menu__item">Sign out</div></div>`;
	const sampleTable = table(
		'1.1fr 1.5fr 1.4fr 104px',
		['Category', 'Your prediction', 'Winner', 'Result'],
		[
			{ cells: ['<span class="t-strong">Actress in a leading role</span>', two('Jessie Buckley', 'Hamnet'), two('Jessie Buckley', 'Hamnet'), status('correct')] },
			{ sel: true, cells: ['<span class="t-strong">Actor in a leading role</span>', two('Timothée Chalamet', 'Marty Supreme'), two('Michael B. Jordan', 'Sinners'), status('missed')] },
			{ cells: ['<span class="t-strong">Best picture</span>', two('One Battle after Another', 'Motion picture'), '<span class="muted">Awaiting result</span>', status('pending')] }
		]
	);
	const comps = [
		cell(
			'Button',
			`<div class="row row--wrap"><button class="btn btn--primary">Primary ${ic('arrowRight')}</button><button class="btn">Secondary</button><button class="btn btn--danger">Destructive</button><button class="btn" disabled>Disabled</button><button class="btn is-focus">Focused</button></div><div class="row row--wrap" style="margin-top:12px"><button class="btn btn--primary btn--lg">Main action · 56 ${ic('arrowRight')}</button><button class="btn btn--google btn--lg">${GOOGLE_G} Continue with Google</button></div>`
		),
		cell(
			'Toggle · pressed again clears the pick (4.2.4)',
			`<div class="row row--wrap">${toggle('Will win', false)}${toggle('Will win', true)}</div><div class="row row--wrap" style="margin-top:12px">${toggle('Should win', false, 'should')}${toggle('Should win', true, 'should')}</div>`
		),
		cell(
			'Field',
			`<div class="stack">${field('Display name', 'Alex Morgan', '1–40 characters, must be unique')}${field('Display name · focused', 'Alex Morgan').replace('class="input"', 'class="input is-focus"')}${field('Display name · error', 'Alex Morgan', '', 'That name is taken. Try another.')}</div>`
		),
		cell(
			'Tag and result status · never color alone (6.3.3)',
			`<div class="row row--wrap"><span class="tag tag--solid">You</span><span class="tag tag--gold">Winner</span><span class="tag">Your favorite</span></div><div class="row row--wrap" style="margin-top:16px;gap:24px">${status('correct')}${status('missed')}${status('pending')}</div>`
		),
		cell(
			'Tabs and notices',
			`<div class="stack">${tabs(['Your picks (8)', 'Unpicked (16)'], 0)}${notice('info', 'You’ve been signed out. Sign in to pick up where you left off.')}${notice('error', 'Couldn’t save your pick. Try again.')}${notice('success', 'Your account is deleted.')}</div>`
		),
		cell(
			'Category list · default, picked, current',
			`<nav class="catlist" style="background:var(--color-base-300);padding:16px"><a class="catlist__item"><span>Casting</span></a><a class="catlist__item"><span>Best picture</span>${ic('check')}</a><a class="catlist__item is-current"><span>Actor in a supporting role</span>${ic('check')}</a></nav>`
		)
	].join('');
	const wide = [
		cell('Navigation · Upcoming (4.8.3.4)', nav('upcoming', null, { ord: '99th' }), 'frame--flush'),
		cell('Navigation · Open (4.5.3)', nav('open', 'picks'), 'frame--flush'),
		cell('Navigation · from lock (4.5.3)', nav('locked', 'leaderboard'), 'frame--flush'),
		cell('Name menu (4.5.4)', menu),
		cell(
			'Page head with numeral block',
			head('The 98th Academy Awards / Your personal ballot', 'Your night. Your predictions.', 'Head or heart? Here’s where you landed.', numeral('08', '/ 24 categories', 33.3, '8 predictions saved · 3 favorites'))
		),
		cell('Dark banner', openBanner('Your picks are still open', `Edit any prediction until ${WHEN}.`), 'frame--flush'),
		cell(
			'Toggle row · default and selected',
			`<div class="stack stack--item">${nomRow(0, 'Benicio Del Toro', 'One Battle after Another', toggle('Will win', false) + toggle('Should win', false, 'should'))}${nomRow(3, 'Sean Penn', 'One Battle after Another', toggle('Will win', true) + toggle('Should win', false, 'should'), true)}</div>`
		),
		cell('Table · tinted header, ivory rows, thin rules, selected row', sampleTable),
		cell('Footer (4.5.6)', footer(), 'frame--flush')
	].join('');
	const sidePair = `<div class="board__grid board__grid--2">${cell(
		'Guidance panel',
		panel('Before the big night', 'Still time to change your mind.', `<p class="t-body-l muted">Your prediction is who you think will win. Your “should win” is who you’re rooting for.</p><button class="btn btn--primary btn--lg btn--block">Finish my ballot ${ic('arrowRight')}</button>`),
		'frame--flush'
	)}${cell('Dialog', DIALOGS[4][1], 'frame--flush')}</div>`;
	return `<div class="board"><div class="stack"><div class="t-wordmark accent">The Gold Ballot</div><h1 class="t-h-xl">Design system</h1><p class="t-body-l muted" style="max-width:720px">One theme, square corners, thin rules, two type families. Token names match the Figma variables; the CSS custom property after each name is the one the daisyUI theme defines. No color is used outside this list (spec rule 4.7.1).</p></div>${section('Color', 'Every text pairing meets WCAG AA (spec rule 4.7.5). Gold #D6BA7F is for dark surfaces only; on ivory use the bronze accent.', color)}${section('Type', 'Bodoni Moda for display, Inter for everything else. Both are under the SIL Open Font License and are self-hosted (spec rule 4.7.2). Nothing is set below 12px.', `<div>${type}</div>`)}${section('Spacing and layout', 'A 4px grid on Tailwind’s scale: space/6 is p-6. The named rules on the right are the canonical paddings and sizes.', space)}${section('Components', 'The named patterns of spec section 4.8. Each screen is built only from these.', `<div class="board__grid board__grid--2">${comps}</div><div class="stack stack--page">${wide}</div>${sidePair}`)}</div>`;
};

SCREENS['signin-states'] = ['4.8.1 Sign-in and name step · states', signinStates];
SCREENS['dialogs'] = ['4.8.7–9 Confirmation dialogs', dialogs];
SCREENS['system'] = ['Design system', system];
