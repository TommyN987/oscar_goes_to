/*
 * The Gold Ballot: static design prototype. One screen per ?s= value, see SCREENS at the bottom.
 * Sample data only: the 98th ceremony as in the original Figma drafts; winners are placeholders.
 * Copy follows docs/spec/4.8-screens.md.
 */
const q = new URLSearchParams(location.search);
const param = (k, d) => q.get(k) || d;

const ICONS = {
	check: '<path d="M20 6 9 17l-5-5"/>',
	x: '<path d="M18 6 6 18M6 6l12 12"/>',
	minus: '<path d="M5 12h14"/>',
	arrowRight: '<path d="M5 12h14M12 5l7 7-7 7"/>',
	arrowLeft: '<path d="M19 12H5M12 19l-7-7 7-7"/>',
	arrowDown: '<path d="M12 5v14M19 12l-7 7-7-7"/>',
	chevronDown: '<path d="m6 9 6 6 6-6"/>',
	refresh:
		'<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/>',
	alert: '<circle cx="12" cy="12" r="9"/><path d="M12 8v5M12 16.5v.01"/>',
	info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v5M12 7.5v.01"/>',
	radio: '<circle cx="12" cy="12" r="8"/>',
	radioOn:
		'<circle cx="12" cy="12" r="8"/><circle cx="12" cy="12" r="4" fill="currentColor" stroke="none"/>',
	search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>'
};
const ic = (n, s = 16) =>
	`<svg class="ic" width="${s}" height="${s}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${ICONS[n]}</svg>`;
const GOOGLE_G =
	'<svg class="ic" width="20" height="20" viewBox="0 0 48 48" aria-hidden="true"><path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z"/><path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z"/><path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z"/><path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z"/></svg>';

/* ---------- sample data ---------- */
const WHEN = 'Sunday, March 15, 2026 at 7:00 PM ET';
const CATS = [
	'Best picture',
	'Directing',
	'Actor in a leading role',
	'Actress in a leading role',
	'Actor in a supporting role',
	'Actress in a supporting role',
	'Original screenplay',
	'Adapted screenplay',
	'Casting',
	'Cinematography',
	'Costume design',
	'Film editing',
	'Makeup & hairstyling',
	'Production design',
	'Original score',
	'Original song',
	'Sound',
	'Visual effects',
	'Animated feature',
	'International feature',
	'Documentary feature',
	'Animated short',
	'Live action short',
	'Documentary short'
];
const pad = (n) => String(n).padStart(2, '0');

/* ---------- patterns (spec 4.8, "Patterns reused from the Figma file") ---------- */
const LINKS = {
	upcoming: [],
	open: [
		['picks', 'Make your picks'],
		['ballot', 'Your ballot']
	],
	locked: [
		['ballot', 'Your ballot'],
		['leaderboard', 'Leaderboard'],
		['statistics', 'Statistics']
	]
};
const nav = (phase, active, o = {}) =>
	`<header class="nav"><div class="nav__brand t-wordmark brand">The Gold Ballot</div><div class="nav__season t-eyebrow">${o.ord || '98th'} Academy Awards</div><nav class="nav__links">${LINKS[
		phase
	]
		.map(([k, l]) => `<a class="nav__link${k === active ? ' is-active' : ''}">${l}</a>`)
		.join('')}</nav><div class="nav__user">${o.user || 'Alex Morgan'} ${ic('chevronDown')}</div></header>`;
const footer = () =>
	`<footer class="footer"><span class="t-eyebrow">The Gold Ballot</span><span class="t-caption">An independent celebration of cinema. Not affiliated with the Academy.</span><a class="link t-caption">Privacy note</a></footer>`;
const shell = (navHtml, body) => `<div class="shell">${navHtml}${body}${footer()}</div>`;

const head = (eyebrow, h1, sub, extra = '') =>
	`<div class="head"><div class="head__main"><div class="t-eyebrow muted">${eyebrow}</div><h1 class="t-h-l">${h1}</h1>${sub ? `<p class="t-body-l muted">${sub}</p>` : ''}</div>${extra}</div>`;
const numeral = (fig, unit, pct, caption) =>
	`<div class="numeral"><div class="numeral__row"><span class="t-numeral-l">${fig}</span><span class="t-body-l muted">${unit}</span></div>${pct == null ? '<hr class="rule">' : `<div class="bar"><i style="width:${pct}%"></i></div>`}<p class="t-caption muted">${caption}</p></div>`;
const banner = (eyebrow, text, figure) =>
	`<div class="banner"><div class="banner__text"><div class="t-eyebrow brand">${eyebrow}</div><p class="t-body-l">${text}</p></div><div class="banner__figure ${/^[A-Z ]+$/.test(figure) ? 't-eyebrow' : 't-h-s'}">${figure}</div></div>`;
const panel = (eyebrow, heading, body) =>
	`<aside class="panel"><div class="t-eyebrow accent">${eyebrow}</div>${heading ? `<h2 class="t-h-s">${heading}</h2>` : ''}${body}</aside>`;
const tabs = (items, active) =>
	`<div class="tabs">${items.map((t, i) => `<a class="tab${i === active ? ' is-active' : ''}">${t}</a>`).join('')}</div>`;
/* rows: [{ cells: [html | { h, cls }], sel }] — each cell repeats its column label for the phone layout */
const table = (cols, heads, rows, o = {}) =>
	`<div class="table ${o.cls || ''}" style="--cols:${cols};--cols-m:${o.colsM || '1fr'}"><div class="tr tr--head">${heads.map((h) => `<div class="td t-eyebrow${h.startsWith('>') ? ' td--end' : ''}">${h.replace('>', '')}</div>`).join('')}</div>${rows
		.map(
			(r) =>
				`<div class="tr${r.sel ? ' is-selected' : ''}">${r.cells
					.map((c, i) => {
						const cell = typeof c === 'string' ? { h: c } : c;
						const label = heads[i].replace('>', '');
						return `<div class="td ${cell.cls || ''}${heads[i].startsWith('>') ? ' td--end' : ''}">${label && !cell.nolabel ? `<span class="td__label t-eyebrow">${label}</span>` : ''}${cell.h}</div>`;
					})
					.join('')}</div>`
		)
		.join('')}</div>`;
const two = (primary, secondary) =>
	`<span class="t-body-l-strong">${primary}</span>${secondary ? `<span class="t-caption muted">${secondary}</span>` : ''}`;
const dash = '<span class="muted">—</span>';
const status = (k) =>
	({
		correct: `<span class="status status--correct">${ic('check')} Correct</span>`,
		missed: `<span class="status status--missed">${ic('x')} Missed</span>`,
		pending: `<span class="status status--pending">${ic('minus')} Pending</span>`
	})[k];
const notice = (kind, text) =>
	`<div class="notice notice--${kind}">${ic(kind === 'error' ? 'alert' : kind === 'success' ? 'check' : 'info')}<span class="t-strong">${text}</span></div>`;
const field = (label, value, helper, error) =>
	`<div class="field${error ? ' is-error' : ''}"><label>${label}</label><div class="input">${value}</div>${error ? `<div class="field__error">${ic('alert')} ${error}</div>` : ''}${helper ? `<p class="t-caption muted">${helper}</p>` : ''}</div>`;
const sidebar = (top, groups, current) =>
	`<aside class="side">${top}<nav class="catlist">${groups
		.map(
			(g) =>
				(g.label ? `<div class="catlist__group t-eyebrow">${g.label}</div>` : '') +
				g.items
					.map(
						(i) =>
							`<a class="catlist__item${i === current ? ' is-current' : ''}"><span>${CATS[i]}</span>${g.tick && g.tick(i) ? ic('check') : ''}</a>`
					)
					.join('')
		)
		.join('')}</nav></aside>`;
const allCats = (label = 'All categories (24)') =>
	`<button class="btn btn--block">${label} ${ic('arrowDown')}</button>`;
const nomRow = (i, name, film, toggles, selected) =>
	`<div class="nom${selected ? ' is-selected' : ''}"><span class="nom__num t-caption muted">${pad(i + 1)}</span><div class="stack stack--text" style="gap:4px"><span class="t-title">${name}</span><span class="muted">${film}</span></div><div class="nom__toggles">${toggles}</div></div>`;
const toggle = (label, pressed, kind = 'will') =>
	`<button class="toggle toggle--${kind}${pressed ? ' is-pressed' : ''}" aria-pressed="${!!pressed}">${kind === 'should' ? (pressed ? ic('check') : '') : ic(pressed ? 'radioOn' : 'radio')} ${label}</button>`;

/* ---------- two-panel: sign-in, name step, blocked (4.8.1, 4.8.2, 4.8.10.3) ---------- */
const darkPanel = () =>
	`<section class="tp-dark"><div class="tp-dark__inner"><div class="t-wordmark brand">The Gold Ballot</div><div class="tp-dark__lead"><div class="t-eyebrow brand">The 98th Academy Awards</div><h2 class="t-h-xl">Great films.<br>Bold predictions.</h2><p class="t-body-l tp-dark__sub">For the films you believe in. And the ones you just know will win.</p></div><div class="tp-dark__rule"></div><div class="tp-dark__numeral t-numeral-xl">98</div><div class="tp-dark__foot"><div class="t-eyebrow brand">Guess the Oscars with Tommy</div><div class="t-eyebrow inv-muted">Sunday, March 15, 2026 / 7 PM ET</div></div><div class="tp-numeral-m"><span class="t-numeral-l">98</span><div class="stack stack--text"><div class="t-eyebrow brand">Guess the Oscars with Tommy</div><div class="t-eyebrow inv-muted">Sunday, March 15, 2026 / 7 PM ET</div></div></div></div></section>`;
const twoPanel = (form) =>
	`<main class="twopanel">${darkPanel()}<section class="tp-form"><div class="tp-form__inner">${form}</div></section></main>`;
const formHead = (eyebrow, h1, sub) =>
	`<div class="stack stack--item"><div class="t-eyebrow muted">${eyebrow}</div><h1 class="t-h-l">${h1}</h1><p class="t-body-l muted">${sub}</p></div>`;
const independence =
	'<hr class="rule"><p class="t-caption muted">An independent celebration of cinema. Not affiliated with the Academy.</p>';
const googleButton = (pending) =>
	pending
		? '<button class="btn btn--lg btn--block" disabled>Opening Google…</button>'
		: `<button class="btn btn--google btn--lg btn--block">${GOOGLE_G} Continue with Google</button>`;
const signinActions = (o = {}) =>
	`<div class="stack stack--item">${o.notice || ''}${googleButton(o.pending)}<p class="t-caption muted">We use your Google account only to confirm your email.<br><a class="link">Read the privacy note</a></p></div>`;
const signin = () =>
	twoPanel(
		formHead(
			'Your seat at awards night',
			'The ballot is yours.',
			'Make your predictions. Follow your favorites.'
		) +
			signinActions() +
			independence
	);
const nameForm = (error) =>
	`<div class="stack stack--item">${field('Display name', 'Alex Morgan', '1–40 characters, must be unique', error)}</div><button class="btn btn--primary btn--lg btn--block">Take my seat</button><p class="muted">Signed in as alex.morgan@example.com. Not you? <a class="link">Sign out</a></p>`;
const nameStep = () =>
	twoPanel(
		formHead(
			'One last thing',
			'What should we call you?',
			'This is the name other players see.'
		) +
			nameForm() +
			independence
	);
const blockedForm = () =>
	formHead(
		'Sign-in',
		'This account can’t join.',
		'This Google account was removed from the game by the admin.'
	) + '<button class="btn btn--lg btn--block">Use a different Google account</button>';

/* ---------- 4.8.3 upcoming ---------- */
const upcoming = () => {
	const half = (from) =>
		table(
			'48px 1fr auto',
			['', 'Category', '>Status'],
			CATS.slice(from, from + 12).map((c, i) => ({
				cells: [
					{ h: `<span class="t-caption muted">${pad(from + i + 1)}</span>`, nolabel: 1 },
					{ h: `<span class="t-strong">${c}</span>`, nolabel: 1 },
					{ h: '<span class="muted">Awaiting nominees</span>', nolabel: 1 }
				]
			}))
		);
	const steps = `<div class="steps">${howSteps()}</div>`;
	return shell(
		nav('upcoming', null, { ord: '99th' }),
		`<main class="page">${head('The 99th Academy Awards / Coming soon', 'The nominees aren’t in yet.', 'The ballot opens as soon as they are announced.')}${banner('Mark your calendar', 'Nominations are announced Thursday, January 21, 2027. Picks lock Sunday, March 14, 2027 at 7:00 PM ET.', 'Jan 21')}<div class="cols"><div class="stack"><div class="t-eyebrow muted">24 categories</div><div class="upcoming-grid">${half(0)}${half(12)}</div></div>${panel('How it works', '', steps)}</div></main>`
	);
};
const STEPS = [
	['Call the winner', 'Choose who you think will win in each category.'],
	['Follow your heart', 'Add an optional “should win” favorite. It can be a different nominee.'],
	['Change your mind', 'Picks save automatically. Edit them until the ceremony begins.'],
	[
		'Keep score',
		'Every correct prediction scores one point. Other players see your score, never your picks.'
	]
];
const howSteps = () =>
	STEPS.map(
		([t, d], i) =>
			`<div class="step"><span class="t-title accent">${pad(i + 1)}</span><span class="t-body-l-strong">${t}</span><span class="muted">${d}</span></div>`
	).join('');

/* ---------- make your picks (rebuilt from the original draft; 4.2.1–4.2.9) ---------- */
const NOMS = [
	['Benicio Del Toro', 'One Battle after Another'],
	['Jacob Elordi', 'Frankenstein'],
	['Delroy Lindo', 'Sinners'],
	['Sean Penn', 'One Battle after Another'],
	['Stellan Skarsgård', 'Sentimental Value']
];
const openBanner = (eyebrow, text) => banner(eyebrow, text, '02d 04h 18m');
const picks = () => {
	const top = `<div class="side__block"><div class="t-eyebrow muted">Your ballot</div><div class="t-title">8 of 24 picked</div><div class="bar"><i style="width:33.3%"></i></div></div>`;
	const side = sidebar(top, [{ items: CATS.map((_, i) => i), tick: (i) => i < 8 }], 4);
	const sideM = `<div class="side-m"><div class="row row--between"><span class="t-strong">8 of 24 picked</span><span class="status status--correct">${ic('check')} Saved</span></div><div class="bar"><i style="width:33.3%"></i></div>${allCats()}</div>`;
	const rows = NOMS.map(([n, f], i) =>
		nomRow(i, n, f, toggle('Will win', i === 3) + toggle('Should win', i === 2, 'should'), i === 3)
	).join('');
	return shell(
		nav('open', 'picks'),
		`<div class="with-side">${side}<main class="page">${sideM}<div class="t-eyebrow muted">The 98th Academy Awards / March 15, 2026</div>${openBanner('The red carpet is almost ready', `Picks lock ${WHEN}, when the ceremony begins.`)}<div class="head__main"><div class="t-eyebrow muted">Category 05 / 24</div><h1 class="t-h-l">Actor in a supporting role</h1><p class="t-body-l muted">Pick who will take home the gold. Add a personal favorite if you like.</p></div><div class="stack stack--item"><div class="nomhead t-eyebrow"><span>One prediction</span><span>Optional favorite</span></div>${rows}</div><div class="row row--between row--wrap"><span class="status status--correct">${ic('check')} All changes saved</span><span class="t-caption muted">One prediction and one optional favorite per category.</span></div><hr class="rule"><div class="row row--between actions"><button class="btn">${ic('arrowLeft')} Previous category</button><div class="row actions"><button class="btn">Review my ballot</button><button class="btn btn--primary">Next category ${ic('arrowRight')}</button></div></div></main></div>`
	);
};

/* ---------- your ballot, open (4.2.10–4.2.13) ---------- */
const BALLOT = [
	['Best picture', 'One Battle after Another', 'Motion picture', 'Sinners'],
	['Directing', 'Paul Thomas Anderson', 'One Battle after Another'],
	['Actor in a leading role', 'Timothée Chalamet', 'Marty Supreme'],
	['Actress in a leading role', 'Jessie Buckley', 'Hamnet'],
	['Actor in a supporting role', 'Sean Penn', 'One Battle after Another', 'Delroy Lindo'],
	['Actress in a supporting role', 'Amy Madigan', 'Weapons'],
	['Original screenplay', 'Sinners', 'Ryan Coogler', 'Sinners'],
	['Adapted screenplay', 'One Battle after Another', 'Paul Thomas Anderson']
];
const ballot = () => {
	const rows = BALLOT.map(([c, w, s, f]) => ({
		cells: [
			`<span class="t-strong">${c}</span>`,
			two(w, s),
			f ? `<span class="accent t-strong">${f}</span>` : dash,
			{ h: '<a class="link">Edit</a>', nolabel: 1 }
		]
	}));
	const side = panel(
		'Before the big night',
		'Still time to change your mind.',
		`<p class="t-body-l muted">Your prediction is who you think will win. Your “should win” is who you’re rooting for. They don’t have to match.</p><hr class="rule"><h3 class="t-title">16 categories to go</h3><p class="muted">There’s more gold to call. Fill the remaining categories before picks lock.</p><button class="btn btn--primary btn--lg btn--block">Finish my ballot ${ic('arrowRight')}</button><p class="t-caption muted">No final submission needed. Every pick saves automatically.</p>`
	);
	return shell(
		nav('open', 'ballot'),
		`<main class="page">${head('The 98th Academy Awards / Your personal ballot', 'Your night. Your predictions.', 'Head or heart? Here’s where you landed. Every pick is saved automatically.', numeral('08', '/ 24 categories', 33.3, '8 predictions saved · 3 favorites'))}${openBanner('Your picks are still open', `Edit any prediction until ${WHEN}, when the ceremony begins.`)}<div class="cols"><div class="stack">${tabs(['Your picks (8)', 'Unpicked (16)'], 0)}${table('1.1fr 1.6fr 1.2fr 48px', ['Category', 'Will win', 'Should win · optional', ''], rows)}</div>${side}</div></main>`
	);
};
const ballotEmpty = () =>
	shell(
		nav('open', 'ballot'),
		`<main class="page">${head('The 98th Academy Awards / Your personal ballot', 'Your night. Your predictions.', 'Every great awards night starts with a hunch.', numeral('00', '/ 24 categories', null, 'No predictions yet · 0 favorites'))}${openBanner('Your picks are open', `Picks lock ${WHEN}, when the ceremony begins.`)}<div class="cols"><div class="card hero"><div class="t-eyebrow accent">The first pick is the fun part</div><div class="t-numeral-l accent">00</div><h2 class="t-h-m">Your ballot is waiting.</h2><p class="t-body-l muted">You haven’t made any predictions yet. Start with Best picture, then work your way through all 24 categories.</p><button class="btn btn--primary btn--lg">Make my first pick ${ic('arrowRight')}</button><p class="t-caption muted">Your picks appear here as you make them.</p></div>${panel('How it works', '', `<div class="steps">${howSteps()}</div>`)}</div></main>`
	);

/* ---------- 4.8.4 results view of "Your ballot" ---------- */
const RESULTS = {
	sealed: {
		eyebrow: 'Picks locked',
		h1: 'The ballot is sealed.',
		sub: 'The guessing is over. Now let the movies do the talking.',
		known: []
	},
	during: {
		eyebrow: 'Ceremony underway',
		h1: 'The envelopes are opening.',
		sub: 'Winners land here as they are announced. Refresh to see the latest.',
		known: [2, 3, 4, 5, 6, 7]
	},
	complete: {
		eyebrow: 'Final results',
		h1: 'That’s a wrap.',
		sub: 'Every envelope is open. Here’s how your ballot held up.',
		known: [0, 1, 2, 3, 4, 5, 6, 7]
	}
};
const WINNERS = [
	['One Battle after Another', 'Motion picture'],
	['Paul Thomas Anderson', 'One Battle after Another'],
	['Michael B. Jordan', 'Sinners'],
	['Jessie Buckley', 'Hamnet'],
	['Sean Penn', 'One Battle after Another'],
	['Amy Madigan', 'Weapons'],
	['Sinners', 'Ryan Coogler'],
	['One Battle after Another', 'Paul Thomas Anderson']
];
const OTHER_WINNERS = { 10: ['Frankenstein', 'Kate Hawley'], 12: ['Frankenstein', 'Makeup team'], 18: ['KPop Demon Hunters', 'Animated feature'] };
const results = () => {
	const phase = param('phase', 'during');
	const r = RESULTS[phase];
	const none = param('tab') === 'none';
	let correct = 0,
		missed = 0;
	const rows = BALLOT.map(([c, w, s, f], i) => {
		const known = r.known.includes(i);
		const hit = known && WINNERS[i][0] === w;
		if (known) hit ? correct++ : missed++;
		return {
			cells: [
				`<span class="t-strong">${c}</span>`,
				two(w, s) + (f ? `<span class="t-caption accent">Should win: ${f}</span>` : ''),
				known ? two(...WINNERS[i]) : '<span class="muted">Awaiting result</span>',
				status(known ? (hit ? 'correct' : 'missed') : 'pending')
			]
		};
	});
	const blank = CATS.slice(8).map((c, i) => {
		const w = phase === 'sealed' ? null : OTHER_WINNERS[i + 8];
		return {
			cells: [
				`<span class="t-strong">${c}</span>`,
				i === 1 ? `${dash}<span class="t-caption accent">Should win: Sinners</span>` : dash,
				w ? two(...w) : '<span class="muted">Awaiting result</span>',
				dash
			]
		};
	});
	const pending = 8 - correct - missed;
	const side = panel(
		'The envelope, please',
		'See how the room is doing.',
		`<p class="t-body-l muted">One point for every correct prediction. The leaderboard updates as winners are recorded.</p><a class="btn btn--primary btn--lg btn--block">Open the leaderboard ${ic('arrowRight')}</a><p class="t-caption muted">Your favorites are just for you. They never score.</p>`
	);
	return shell(
		nav('locked', 'ballot'),
		`<main class="page">${head(`The 98th Academy Awards / ${r.eyebrow}`, r.h1, r.sub, numeral(pad(correct), correct === 1 ? 'point' : 'points', null, `${correct} correct · ${missed} missed · ${pending} pending`))}${banner('Picks locked', `${WHEN}. Your predictions can no longer be changed.`, 'READ ONLY')}<div class="cols"><div class="stack">${tabs(['Saved picks (8)', 'No prediction (16)'], none ? 1 : 0)}${table('1.1fr 1.5fr 1.4fr 104px', ['Category', 'Your prediction', 'Winner', 'Result'], none ? blank : rows)}</div>${side}</div></main>`
	);
};

/* ---------- 4.8.5 leaderboard ---------- */
const PLAYERS = [
	['Popcorn Prophet', 6],
	['Marta K.', 6],
	['Alex Morgan', 5],
	['Dušan', 5],
	['Ines V.', 4],
	['Reel Talk', 4],
	['Tommy', 4],
	['Best Boy Grip', 3],
	['Klára', 3],
	['The Understudy', 3],
	['Jonas', 2],
	['Nora B.', 2],
	['Cutting Room', 1],
	['Peter', 0]
];
const leaderboard = () => {
	const state = param('state', 'default');
	if (state === 'prelock')
		return shell(
			nav('open', null),
			`<main class="page">${head('The 98th Academy Awards / Leaderboard', 'The scores open at showtime.', `The leaderboard opens when picks lock: ${WHEN}.`)}<div><a class="btn btn--primary">Back to my ballot ${ic('arrowRight')}</a></div></main>`
		);
	const unranked = state === 'unranked';
	let rank = 0;
	const rows = PLAYERS.filter((p) => !(unranked && p[0] === 'Alex Morgan')).map(([n, pts], i, a) => {
		if (i === 0 || a[i - 1][1] !== pts) rank = i + 1;
		const you = n === 'Alex Morgan';
		return {
			sel: you,
			cells: [
				`<span class="t-title">${rank}</span>`,
				{
					h: `<span class="t-body-l-strong">${n}</span>${you ? '<span class="tag tag--solid">You</span>' : ''}`,
					cls: 'td--row'
				},
				`<span class="t-title">${pts}</span>`
			]
		};
	});
	const block = unranked
		? numeral('—', 'not ranked', null, '13 players · 0 points')
		: numeral('03', 'of 14 players', null, 'Your rank · 5 points');
	const side = panel(
		'How scoring works',
		'One point per envelope.',
		`<p class="t-body-l muted">Every correct prediction scores one point. Equal scores share a rank. Other players see your score, never your picks.</p><a class="btn btn--primary btn--lg btn--block">See your ballot ${ic('arrowRight')}</a>`
	);
	return shell(
		nav('locked', 'leaderboard'),
		`<main class="page">${head('The 98th Academy Awards / Leaderboard', 'Who called it?', 'One point for every correct prediction. Refresh to see the latest.', block)}<div class="cols"><div class="stack">${unranked ? notice('info', 'You’re not ranked because you saved no predictions before picks locked.') : ''}<div class="row row--between"><span class="t-eyebrow muted">${unranked ? 13 : 14} players</span><button class="btn">${ic('refresh')} Refresh</button></div>${table('88px 1fr 88px', ['Rank', 'Player', '>Points'], rows, { cls: 'table--keep', colsM: '40px 1fr 48px' })}</div>${side}</div></main>`
	);
};

/* ---------- 4.8.6 statistics ---------- */
const STATS = [
	[
		['Sinners', 'Motion picture', 5, 'fav'],
		['One Battle after Another', 'Motion picture', 4],
		['Hamnet', 'Motion picture', 2],
		['Marty Supreme', 'Motion picture', 1],
		['Sentimental Value', 'Motion picture', 1],
		['Bugonia', 'Motion picture', 0],
		['F1', 'Motion picture', 0],
		['Frankenstein', 'Motion picture', 0],
		['The Secret Agent', 'Motion picture', 0],
		['Train Dreams', 'Motion picture', 0]
	],
	null,
	[
		['Timothée Chalamet', 'Marty Supreme', 4],
		['Michael B. Jordan', 'Sinners', 3, 'win'],
		['Leonardo DiCaprio', 'One Battle after Another', 2],
		['Wagner Moura', 'The Secret Agent', 1],
		['Ethan Hawke', 'Blue Moon', 0]
	],
	[
		['Jessie Buckley', 'Hamnet', 6, 'win'],
		['Renate Reinsve', 'Sentimental Value', 2],
		['Emma Stone', 'Bugonia', 1],
		['Rose Byrne', 'If I Had Legs I’d Kick You', 0],
		['Kate Hudson', 'Song Sung Blue', 0]
	],
	[
		['Delroy Lindo', 'Sinners', 6, 'fav'],
		['Sean Penn', 'One Battle after Another', 3, 'win'],
		['Benicio Del Toro', 'One Battle after Another', 2],
		['Stellan Skarsgård', 'Sentimental Value', 1],
		['Jacob Elordi', 'Frankenstein', 0]
	]
];
const statSection = (i) => {
	const rows = STATS[i];
	const total = rows ? rows.reduce((s, r) => s + r[2], 0) : 0;
	const max = rows ? Math.max(...rows.map((r) => r[2])) : 0;
	const tag = { win: '<span class="tag tag--gold">Winner</span>', fav: '<span class="tag">Your favorite</span>' };
	return `<section class="stack stack--item"><div class="row row--between" style="align-items:flex-end"><div class="stack stack--text"><div class="t-eyebrow muted">Category ${pad(i + 1)} / 24</div><h2 class="t-h-s">${CATS[i]}</h2></div><span class="t-caption muted">${total} ${total === 1 ? 'favorite' : 'favorites'}</span></div>${
		rows
			? `<div class="table" style="border-top:1px solid var(--gb-line)">${rows
					.map(
						([n, s, c, t]) =>
							`<div class="statrow"><div class="td"><div class="row row--wrap" style="gap:8px"><span class="t-body-l-strong">${n}</span>${t ? tag[t] : ''}</div><span class="t-caption muted">${s}</span></div><div class="statbar"><i style="width:${max ? (c / max) * 100 : 0}%"></i></div><span class="t-title" style="text-align:right">${c}</span></div>`
					)
					.join('')}</div>`
			: '<div class="card muted">No favorites in this category.</div>'
	}</section>`;
};
const statistics = () => {
	const top = `<div class="side__block"><div class="t-eyebrow muted">Jump to</div><div class="t-title">24 categories</div></div>`;
	return shell(
		nav('locked', 'statistics'),
		`<div class="with-side">${sidebar(top, [{ items: CATS.map((_, i) => i) }], 0)}<main class="page">${head('The 98th Academy Awards / Statistics', 'Who the room was rooting for.', 'Favorites, counted. No names attached.')}<div class="side-m">${allCats()}</div><div class="stack stack--page" style="gap:48px">${STATS.map((_, i) => statSection(i)).join('')}</div></main></div>`
	);
};

/* ---------- 4.8.7 admin results ---------- */
const RECORDED = [2, 3, 4, 5, 6, 7, 10, 12, 18];
const adminTabs = (active) => tabs(['Results', 'Players'], active);
const adminSide = (current) => {
	const todo = CATS.map((_, i) => i).filter((i) => !RECORDED.includes(i));
	const top = `<div class="side__block"><div class="t-eyebrow muted">Results</div><div class="t-title">9 / 24 recorded</div><div class="bar"><i style="width:37.5%"></i></div><div class="input is-placeholder" style="gap:8px;font-size:14px">${ic('search')} Filter categories</div></div>`;
	return sidebar(
		top,
		[
			{ label: 'To announce (15)', items: todo },
			{ label: 'Recorded (9)', items: RECORDED, tick: () => true }
		],
		current
	);
};
const adminSideM = () =>
	`<div class="side-m"><div class="row row--between"><span class="t-strong">9 / 24 recorded</span><span class="muted">15 to announce</span></div><div class="bar"><i style="width:37.5%"></i></div>${allCats()}</div>`;
const CINE = [
	['Frankenstein', 'Dan Laustsen'],
	['Marty Supreme', 'Darius Khondji'],
	['One Battle after Another', 'Michael Bauman'],
	['Sinners', 'Autumn Durald Arkapaw'],
	['Train Dreams', 'Adolpho Veloso']
];
const adminResults = () => {
	const state = param('state', 'default');
	const adminNav = nav(state === 'prelock' ? 'open' : 'locked', null, { user: 'Tommy' });
	if (state === 'prelock')
		return shell(
			adminNav,
			`<main class="page">${adminTabs(0)}${head('Admin / Results', 'Results open at lock.', `You can record winners from ${WHEN}.`)}</main>`
		);
	if (state === 'recorded')
		return shell(
			adminNav,
			`<div class="with-side">${adminSide(4)}<main class="page">${adminTabs(0)}${adminSideM()}<div class="head__main"><div class="t-eyebrow muted">Category 05 / 24 · Recorded</div><h1 class="t-h-l">Actor in a supporting role</h1><p class="t-body-l muted">This result is live. Correcting or clearing it changes the scores.</p></div><div class="card stack"><div class="stack stack--text"><div class="t-eyebrow accent">Winner</div><div class="t-h-s">Sean Penn</div><div class="muted">One Battle after Another</div></div><hr class="rule"><div class="row row--between row--wrap"><span class="t-caption muted">Recorded Sunday, March 15, 2026 at 8:14 PM ET</span><div class="row actions"><button class="btn">Correct</button><button class="btn btn--danger">Clear</button></div></div></div><hr class="rule"><div class="row row--between actions"><button class="btn">${ic('arrowLeft')} Previous category</button><button class="btn">Next category ${ic('arrowRight')}</button></div></main></div>`
		);
	const rows = CINE.map(([n, f], i) => nomRow(i, n, f, toggle('Winner', i === 3), i === 3)).join('');
	return shell(
		adminNav,
		`<div class="with-side">${adminSide(9)}<main class="page">${adminTabs(0)}${adminSideM()}<div class="head__main"><div class="t-eyebrow muted">Category 10 / 24 · To announce</div><h1 class="t-h-l">Cinematography</h1><p class="t-body-l muted">Press the winner, then review. Press more than one only for a tie.</p></div><div class="stack stack--item">${rows}</div><hr class="rule"><div class="row row--between actions"><button class="btn">${ic('arrowLeft')} Previous category</button><div class="row actions"><button class="btn">Next category ${ic('arrowRight')}</button><button class="btn btn--primary">Review</button></div></div></main></div>`
	);
};

/* ---------- 4.8.8 admin players ---------- */
const JOINED = ['Mar 14, 2026', 'Mar 12, 2026', 'Mar 9, 2026', 'Mar 9, 2026', 'Mar 2, 2026', 'Feb 27, 2026', 'Feb 20, 2026', 'Feb 20, 2026', 'Feb 11, 2026', 'Feb 3, 2026', 'Jan 30, 2026', 'Jan 24, 2026', 'Jan 23, 2026', 'Jan 22, 2026'];
const adminPlayers = () => {
	const order = [13, 11, 5, 3, 9, 0, 12, 8, 2, 4, 7, 10, 1, 6];
	const rows = order.map((p, i) => {
		const name = PLAYERS[p][0];
		const me = name === 'Tommy';
		return {
			cells: [
				{
					h: `<span class="t-body-l-strong">${name}</span>${me ? '<span class="tag tag--solid">You</span>' : ''}`,
					cls: 'td--row'
				},
				`<span class="muted">${name.toLowerCase().replace(/[^a-zšá]+/g, '.').replace(/\.$/, '').replace('š', 's').replace('á', 'a')}@example.com</span>`,
				`<span class="muted"><span class="only-m">Joined </span>${JOINED[i]}</span>`,
				{ h: me ? '' : '<a class="link">Remove</a>', nolabel: 1 }
			]
		};
	});
	const blocked = [
		['spoiler.alert@example.com', 'Mar 10, 2026'],
		['second.account@example.com', 'Feb 26, 2026']
	].map(([e, d]) => ({
		cells: [`<span class="t-strong">${e}</span>`, `<span class="muted"><span class="only-m">Removed </span>${d}</span>`, { h: '<a class="link">Unblock</a>', nolabel: 1 }]
	}));
	return shell(
		nav('locked', null, { user: 'Tommy' }),
		`<main class="page">${adminTabs(1)}${head('Admin / Players', 'Who’s in the room.', '14 players, newest first. Removing a player deletes their predictions and blocks their Google account.')}${table('1.2fr 1.6fr 1fr 72px', ['Name', 'Email', 'Joined', ''], rows, { cls: 'table--tight' })}<div class="stack stack--item"><h2 class="t-h-s">Blocked accounts</h2><p class="muted">These Google accounts can’t sign in.</p></div>${table('1.6fr 1fr 72px', ['Email', 'Removed', ''], blocked, { cls: 'table--tight' })}</main>`
	);
};

/* ---------- 4.8.9 settings ---------- */
const settings = () =>
	shell(
		nav('open', null),
		`<main class="page">${head('Account / Settings', 'Settings', 'Change your name, sign out, or leave the game.')}<div class="cols"><div class="stack stack--page"><div class="stack" style="max-width:var(--size-form)">${field('Display name', 'Alex Morgan', '1–40 characters, must be unique. This is the name other players see.')}<div><button class="btn btn--primary">Save</button></div></div><hr class="rule"><div class="row row--between row--wrap"><div class="stack stack--text" style="gap:4px"><span class="t-strong">Signed in with Google as alex.morgan@example.com</span><span class="t-caption muted">Your email comes from Google and can’t be changed here.</span></div><button class="btn">Sign out</button></div></div>${panel('Leaving the game', 'Delete your account', '<p class="t-body-l muted">Your account, your predictions and your favorites are removed. This cannot be undone.</p><button class="btn btn--danger btn--lg btn--block">Delete my account</button>')}</div></main>`
	);

/* ---------- 4.8.10 not-found and error; 4.8.11 privacy note ---------- */
const notFound = () =>
	shell(
		nav('open', null),
		`<main class="page"><div class="hero" style="margin:auto"><div class="t-eyebrow accent">Page not found</div><h1 class="t-h-l">This scene didn’t make the final cut.</h1><p class="t-body-l muted">The page you asked for doesn’t exist, or it has moved.</p><a class="btn btn--primary btn--lg">Back to the ballot ${ic('arrowRight')}</a></div></main>`
	);
const errorPage = () =>
	shell(
		nav('open', null),
		`<main class="page"><div class="hero" style="margin:auto"><div class="t-eyebrow accent">Something broke</div><h1 class="t-h-l">Something went wrong on our side.</h1><p class="t-body-l muted">Refresh to try again. Your saved picks are safe.</p><button class="btn btn--primary btn--lg">${ic('refresh')} Refresh</button></div></main>`
	);
const privacy = () =>
	shell(
		nav('open', null),
		`<main class="page">${head('Privacy note', 'What we keep, and who sees it.', 'The short version: as little as possible, and only what the game needs.')}<div class="prose"><section><h2 class="t-h-s">What is stored</h2><p class="t-body-l">For each player we store a Google account ID, the email address Google confirms, your display name, and timestamps such as when you joined. Your predictions and favorites are stored with your account.</p><p class="t-body-l">If the admin removes a player, we keep that account’s ID, email and removal time so it stays blocked.</p></section><section><h2 class="t-h-s">Who can see it</h2><p class="t-body-l">Only you can see your predictions and favorites, at every stage of the game. Other players see your display name and your score. Favorites appear only as anonymous counts. The admin can see your display name and email.</p></section><section><h2 class="t-h-s">How to delete it</h2><p class="t-body-l">Open Settings and choose “Delete my account”. Your account, predictions and favorites are erased, and it cannot be undone.</p></section></div></main>`
	);

/* ---------- router ---------- */
const SCREENS = {
	signin: ['4.8.1 Sign-in', signin],
	name: ['4.8.2 Name step', nameStep],
	upcoming: ['4.8.3 Upcoming notice', upcoming],
	picks: ['4.2 Make your picks', picks],
	ballot: ['4.2 Your ballot · open', ballot],
	'ballot-empty': ['4.2.13 Your ballot · empty', ballotEmpty],
	results: ['4.8.4 Your ballot · results', results],
	leaderboard: ['4.8.5 Leaderboard', leaderboard],
	statistics: ['4.8.6 Statistics', statistics],
	'admin-results': ['4.8.7 Admin results', adminResults],
	'admin-players': ['4.8.8 Admin players', adminPlayers],
	settings: ['4.8.9 Settings', settings],
	notfound: ['4.8.10.1 Not found', notFound],
	error: ['4.8.10.5 Page error', errorPage],
	privacy: ['4.8.11 Privacy note', privacy]
};
function render() {
	const s = param('s', 'index');
	const app = document.getElementById('app');
	if (!SCREENS[s]) {
		app.innerHTML = `<div class="board"><h1 class="t-h-l">The Gold Ballot · prototype</h1><ul class="stack stack--item">${Object.entries(SCREENS)
			.map(([k, [t]]) => `<li><a class="link" href="?s=${k}">${t}</a></li>`)
			.join('')}</ul></div>`;
		return;
	}
	const [title, fn] = SCREENS[s];
	const variant = [param('phase', ''), param('state', ''), param('tab', '')].filter(Boolean).join(' ');
	document.title = `${title}${variant ? ' (' + variant + ')' : ''} · ${window.innerWidth}`;
	app.innerHTML = fn();
	if (location.hash.includes('figmacapture')) {
		const el = document.createElement('script');
		el.src = 'https://mcp.figma.com/mcp/html-to-design/capture.js';
		document.head.appendChild(el);
	}
}
