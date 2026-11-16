# Research: Oscar data sources

Researched on 2026-11-16. Part of [the research record](README.md).
Labels: **[C]** confirmed (primary source read or API queried directly), **[P]** probable, **[U]** uncertain.
All times are UTC unless stated.

## Part 1: 99th Academy Awards facts

- **Date:** Sunday 14 March 2027, Dolby Theatre. [C] https://abc.com/news/0346fd99-0415-425f-a09e-4bd26cb324d4/category/3590738
- **Start time:** "The shows will air live at 7 p.m. ET / 4 p.m. PT".
  This is officially stated in the joint Academy/ABC release of 7 April 2026, not inferred from prior years.
  The same release says all dates are subject to change. [C] same URL
  - Inference: US daylight saving starts that same day, so 7 p.m. EDT is 23:00 UTC, which is 2027-03-15 00:00 CET.
  - The 2026 show also started at 23:00:00Z, per a Wikipedia edit stamped "The ceremony officially begins". [P] https://en.wikipedia.org/w/index.php?title=98th_Academy_Awards&action=history
- **Broadcaster:** ABC, plus 200+ territories. [C] ABC release above.
  Hulu streaming is stated only on Wikipedia. [P] https://en.wikipedia.org/wiki/99th_Academy_Awards
- **Future broadcaster:** 2027 and 2028 are the last ABC years; YouTube takes over from 2029. [P] https://www.cbsnews.com/news/oscars-youtube-2029
- **Other published dates:** [C] ABC release above
  - Shortlists: 15 December 2026
  - Nominations voting: 11–15 January 2027
  - Nominations announced: Thursday 21 January 2027
  - Nominees luncheon: 16 February 2027
  - Final voting: 25 February – 4 March 2027
- **Nominations clock time:** not published for 2027.
  In 2026 it ran in two blocks, 5:30 and 5:41 a.m. PT (13:30 and 13:41 UTC). [C for 2026] https://press.oscars.org/news/98th-oscarsr-nominations-announced
- **Categories:** 24, the same set as the 98th. [P] https://en.wikipedia.org/wiki/99th_Academy_Awards
  - The 24 names are listed on the official 2026 ceremony page. [C] https://www.oscars.org/oscars/ceremonies/2026
  - Casting was new at the 98th (23 to 24). [P] https://awardswatch.com/academy-adds-new-best-casting-category-to-start-at-98th-oscars
  - No category is added or removed for the 99th.
    Rule changes: Casting statuettes rise from a maximum of two to three; an actor can be nominated twice in the same category; International Feature credits the film rather than the country. [C] https://press.oscars.org/news/awards-rules-and-campaign-promotional-regulations-approved-99th-oscarsr
  - Stunt Design starts at the 100th (2028), not the 99th. [P] https://awardswatch.com/?p=110913

## Part 2: data sources

### Official (oscars.org, press site, Awards Database)

- The oscars.org ceremony page lists nominees and winners per category, including the 2026 tie.
  There is no API or feed. [C] https://www.oscars.org/oscars/ceremonies/2026
- A plain scripted request (curl, generic User-Agent) got HTTP 403 "Access Denied" from www.oscars.org, press.oscars.org and awardsdatabase.oscars.org, including robots.txt.
  A browser-like fetcher worked. [C] research agent's own test, 2026-11-16
- The nominations press release does not contain the nominee list; it points to oscars.org. [C] https://press.oscars.org/news/98th-oscarsr-nominations-announced
- A search snippet described the Awards Database as complete only through the 97th ceremony; the date of that snippet is unknown. [U] https://awardsdatabase.oscars.org/

### Wikipedia (MediaWiki API; free; text CC BY-SA)

- No hard read limit; a descriptive User-Agent is mandatory; serial requests are advised. [C] https://www.mediawiki.org/wiki/API:Etiquette
- **Winners 2026:** 249 revisions during the show.
  Each winner marker appeared within about a minute of the Polymarket price collapse for the same category (table below). [C] https://en.wikipedia.org/w/index.php?title=98th_Academy_Awards&action=history
- **Winners 2025:** same pattern, 254 revisions.
  Wrong winners were also inserted that night: The Wild Robot at 03:31:18Z and Demi Moore at 03:54:01Z (the actual winners were Flow and Mikey Madison). [C] https://en.wikipedia.org/w/index.php?title=97th_Academy_Awards&action=history
- The 98th article has no protection log entries, and temporary (anonymous) accounts edited it during the show; 7 reverts in 2026, 8 in 2025. [C] same histories
- **Nominees 2026:** an empty template was in place at 13:31.
  Of 250 empty placeholders, 134 remained at 14:04, 39 at 14:45, 18 at 15:25 and 14 at 16:30.
  Filling took over two hours. [C] same history
- Winners are marked in free wikitext (bold plus a double-dagger template), not structured data. [C]

### Wikidata (SPARQL and API; CC0)

- The ceremony item Q85315079 carries no nominee or winner statements. [C] https://www.wikidata.org/wiki/Q85315079
- Award statements on the Best Picture winner (Q117085614) were hand-added by one editor at 04:33–04:39, about two hours after the announcement.
  Some nomination statements were created only on 4 March 2026, six weeks after nominations. [C] https://www.wikidata.org/w/index.php?title=Q117085614&action=history

### Prediction markets

- Polymarket ran per-category winner and nomination markets in 2026.
  2027 markets are already open (for example Best Picture winner with 46 outcomes).
  The Gamma and CLOB APIs returned data with no key. [C] https://gamma-api.polymarket.com/events?slug=oscars-2026-best-picture-winner
- Rate limits: `/events` 500 requests per 10 s, `/prices-history` 1,000 per 10 s. [C] https://docs.polymarket.com/quickstart/introduction/rate-limits
- Kalshi has per-category series, and 2027 events are active.
  Public reads need no auth; unauthenticated limits are "separate" and unpublished. [C] https://api.elections.kalshi.com/trade-api/v2/historical/markets?event_ticker=KXOSCARPIC-26 and https://docs.kalshi.com/getting_started/rate_limits
- 2026 timing (Polymarket price history is minute-sampled): [C]

| Category | Wikipedia marker | Polymarket price ≥0.985 | Kalshi trading closed |
|---|---|---|---|
| Animated Feature | 23:32:13 | 23:32:20 | 23:36:14 |
| Costume Design | 23:51:44 | 23:51:26 | 23:54:22 |
| Casting | 00:04:50 | 00:05:30 | not checked |
| Sound | 01:36:05 | 01:36:22 | 01:43:57 |
| Actor | 02:21:38 | 02:21:22 | 02:23:58 |
| Picture | 02:36:52 | 02:36:25 | 02:57:49 |

- Formal Polymarket resolution (UMA oracle) came hours later, between 04:31 and 07:17.
  Kalshi settled one hour after its close. [C] same APIs
- Caveats:
  - Favourites already traded at about 0.98 before the announcement (Makeup, Live Action Short), so a price threshold is ambiguous. [C]
  - For Original Score the price crossed about 3.5 minutes after the Wikipedia edit. [C]
  - Polymarket's tie rule resolves to the alphabetically first film, so only The Singers resolved "Yes" in the 2026 tie. [C]
  - Winner options are market-defined strings, not IDs. [C]
  - The search found 17 of the 24 Polymarket winner events. [C]
  - On-stage times were not independently established; "within a minute or two of the announcement" is an inference from two independent signals agreeing. [P]

### TMDB

- Free for non-commercial use with attribution: "This product uses the TMDB API but is not endorsed or certified by TMDB", plus the logo. [C] https://developer.themoviedb.org/docs/faq
- Terms forbid caching longer than six months and forbid use "in connection with… a machine learning (ML) or artificial intelligence (AI) based Application". [C] https://www.themoviedb.org/api-terms-of-use
- No awards endpoint was found; a user feature request says TMDB has no structured awards. [P] https://www.themoviedb.org/talk/67db473213d4b1aed4e9d8aa
- Posters and metadata come through the same API and terms.
  TMDB says "We do not claim ownership of any of the images or data in the API". [C] FAQ above

### OMDb

- Free key with a 1,000 requests/day limit. [C] https://www.omdbapi.com/apikey.aspx
- Content is CC BY-NC 4.0; the Poster API is patrons-only. [C] https://www.omdbapi.com/
- The Awards field is a free-text summary, not per-category data. [P] from memory, not verified on the page

### IMDb

- The free daily TSV dumps (titles, names, crew, ratings) contain no awards and are for personal, non-commercial use. [C] https://data.imdb.com/non-commercial-datasets/
- The paid API is on AWS Data Exchange with contract pricing and no public price. [P] https://aws.amazon.com/marketplace/pp/prodview-bdmgbi3sikr6i

### Trakt, JustWatch, Letterboxd

- JustWatch has no open API; access is by partner contract. [P] https://apis.justwatch.com/docs/api
- The Letterboxd API is by request only. [C] https://letterboxd.com/api-beta/
- Trakt awards coverage was not established. [U]

### Open datasets

- DLu/oscar_data (BSD-2, includes IMDb IDs): 2026 nominations were committed at 15:06 on 22 January, about 1.5 hours after the announcement.
  Wins were committed on 22 April 2026, five weeks after the ceremony. [C] https://github.com/DLu/oscar_data/commits
- delventhalz/json-nominations: last push April 2024. [C] https://github.com/delventhalz/json-nominations
- The Kaggle "the-oscar-award" set is updated yearly. [P] https://baselight.app/u/kaggle/dataset/unanimad_the_oscar_award

### News

- ABC, CBS and SlashFilm ran "updated live" winner pages as HTML.
  No free structured feed was found. [P] https://www.cbsnews.com/live-updates/oscar-winners-2026-list/ and https://www.slashfilm.com/2123919/oscars-2026-winners-list
- No AP or Reuters awards feed or pricing was found. [U]

### Pools and ballot apps

- The open-source cilim/oscars has the admin click winners, pushed to browsers over WebSocket. [P] https://github.com/cilim/oscars
- A Google-Sheets approach uses a hand-filled winners tab. [P] https://leaderboarded.com/blog/posts/oscar-pool-leaderboard/
- OfficePools and the ESPN/ABC Pick'em advertise live scoring but do not disclose the mechanism and expose no data. [P] https://www2.officepools.com/fantasy-entertainment/ and https://abc7news.com/post/can-predict-winners-2026-oscars-compete-prizes-pickem-game/18621578/

## Unknowns

- oscars.org terms of use: guessed URLs returned 404, and robots.txt was blocked.
  Whether automated access is permitted is not established.
- How fast oscars.org, ABC and the Academy's social accounts posted winners in 2025 and 2026: no timestamps found.
- Polymarket and Kalshi terms on reuse of market data, and any jurisdiction limits on read-only API access: primary terms not read.
  One third-party page says Kalshi restricts commercial redistribution. [U]
- Kalshi's unauthenticated rate limit, and when its `result` field is populated relative to the close time.
- Gold Derby data access or terms: nothing found.
- Whether any project uses closed captions or an LLM reading a liveblog for winners: nothing found.
- Whether Wikipedia poster images are reusable: believed to be non-free fair-use files, not verified.
- TMDB numeric rate limits: not in the FAQ or terms.
- 2025 prediction-market timing, and on-stage timestamps for either year.
