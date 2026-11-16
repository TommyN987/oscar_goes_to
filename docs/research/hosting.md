# Research: zero-cost hosting, database, domain, email, deployment

Researched on 2026-11-16. Part of [the research record](README.md).
Labels: **C** confirmed on the provider's own current page, **P** probable (secondary source or inference, marked "inf."), **U** uncertain.
Bracketed numbers refer to the source list at the end.

## 1. Compute for a Rust backend

| Option | Free allowance | Idle / cold start | Card | SSE / WebSocket | Reclamation, EU | Shape of the Rust code |
|---|---|---|---|---|---|---|
| Cloud Run | 2M requests, 180k vCPU-s, 360k GB-s per month; no end date; Google can change it with 30 days' notice (C)[1] | Scales to zero | Billing account required (C)[1] | WebSockets supported; streams limited by request timeout, max 60 min; an open connection keeps the instance billed (C)[2][3] | No inactivity policy found; page states no region limit for Cloud Run (C)[1] | Container |
| AWS Lambda | 1M requests + 400k GB-s per month (C)[4] | Scales to zero | P (inf.) | Response streaming via Function URLs; non-Node languages need a custom runtime or Lambda Web Adapter (C)[5] | Free-plan accounts close after 6 months or when credits run out unless upgraded to paid; "30+ services always free" on both plans (C)[6] | Lambda runtime |
| Azure Container Apps | 180k vCPU-s, 360k GiB-s, 2M requests per subscription per month; no charge at zero replicas (C)[7] | Scales to zero | Card needed at sign-up (P)[8] | Not checked | Not checked | Container |
| Azure Functions (Flex) | 250k executions + 100k GB-s per month (P)[8] | "Improved cold start" (C)[9] | As above | HTTP response limit 230 s (C)[9] | Linux Consumption retires 2028-09-30 (C)[9] | Rust support not verified |
| Oracle Always Free | 2 AMD micro VMs (1/8 OCPU, 1 GB each); Arm A1 1,500 OCPU-h + 9,000 GB-h = 2 OCPU / 12 GB; 200 GB block storage; 10 TB egress (C)[10] | Always on | P [11] | Unrestricted (inf.) | Idle instances "may be reclaimed" if over 7 days CPU p95 < 20%, network < 20% and (A1) memory < 20%; home region only (C)[10] | Any binary |
| Fly.io | No free tier; USD 1 / 7-day trial; personal-org bills under USD 5 are waived; smallest machine USD 2.19/month (C)[12] | n/a | Required after trial (C)[12] | n/a | n/a | Container |
| Render | 750 instance-hours/month; no persistent disk (C)[13] | Spins down after 15 min idle; about 1 min to wake (C)[13] | Not required (C)[13] | No max WebSocket duration; deploys close connections (C)[14] | Suspended if hours or bandwidth run out (C)[13] | Container |
| Railway | Free plan: USD 1 credit/month, 0.5 GB RAM, 0.5 GB volume; trial is USD 5 one-time (C)[15] | Not checked | Not checked | Not checked | Free/trial volume data deleted 30 days after expiry (C)[15] | Container |
| Koyeb | One free web service: 0.1 vCPU, 512 MB, 2 GB disk, no volumes; Frankfurt or Washington (C)[16][17] | Sleeps after 1 h idle, cannot be disabled; HTTP/2 requests cannot wake it (C)[18] | Required, USD 29 pre-authorisation hold (C)[16] | An open WebSocket prevents scale-down (C)[18] | Described as not for production (C)[17] | Container |
| Shuttle | Ceased operations; community-tier projects deleted from early 2026 (C)[19] | n/a | n/a | n/a | n/a | n/a |
| Cloudflare Workers | 100k requests/day, 10 ms CPU per request, 128 MB; SQLite Durable Objects on Free (C)[20][21] | No container cold start | Not checked | Streaming and WebSockets work; hibernated Durable Objects accrue no duration charge (C)[21][22] | Error 1027 over the daily limit (C)[21] | WASM via workers-rs (C)[23] |
| Cloudflare Containers | Free plan shows no allowance; included usage is on Workers Paid, USD 5/month (C)[24] | n/a | n/a | n/a | n/a | n/a |
| Vercel | Rust runtime in beta on all plans; Hobby max function duration 300 s; exceeding limits pauses the feature for 30 days (C)[25][26] | Not checked | Not checked | Streams capped at 300 s (inf.) | Not checked | One binary per `api/` file, `vercel_runtime` |
| Netlify | Rust was experimental in 2021; no current docs found (U)[27] | n/a | n/a | n/a | n/a | n/a |
| Northflank | Sandbox: 2 services, 2 jobs, 1 database; "always on, no sleeping" (C)[28] | None | Not needed (C)[28] | Not checked | Regions not stated | Container |
| Zeabur | Free plan: services auto-sleep, no SLA (C)[29] | Cold start on wake | Not needed (C)[29] | Not checked | Not checked | Container |
| Home + Cloudflare Tunnel | Named tunnel requires a site added to Cloudflare (C)[30]; quick tunnels have a random hostname, no SSE, 200 concurrent requests, testing only (C)[31] | Always on | n/a | See allowance | n/a | Any binary |
| Home + Tailscale Funnel | All plans; `*.ts.net` hostname only; ports 443/8443/10000; unspecified bandwidth limits (C)[32] | Always on | n/a | Not checked | n/a | Any binary |

- GCP's free e2-micro VM exists only in three US regions (C)[1].
- Older guides give Oracle A1 as 4 OCPU / 24 GB; Oracle's page now says 2 / 12 (C)[10].
  A1 "out of capacity" errors are widely reported (P)[11].
- Peak-night arithmetic for Cloud Run: one instance held open for 3.5 hours uses about 12,600 of 180,000 free vCPU-seconds (inf., assuming 1 vCPU and one instance).

## 2. Database

| Option | Limits | Inactivity | Backups | Connections |
|---|---|---|---|---|
| Neon | 100 CU-hours per project per month, no card; the storage figure was unclear in the report (0.5 or 1 GB per project) (C)[33] | Suspends after 5 min; resumes in a few hundred ms; no inactivity pause (C)[33] | 6-hour history (C)[33] | 112 direct at 0.25 CU; 10,000 pooled (C)[34] |
| Supabase | 500 MB, 2 active projects (C)[35] | Paused after 1 week of inactivity (C)[35]; restore window length unclear (U)[36] | None on Free (C)[35] | 60 direct, 200 pooled (C)[37] |
| Turso | 5 GB, 500M row reads, 10M row writes per month, 100 databases, no card (C)[38] | Archived after 10 days idle, per Turso docs via search (P)[39] | 1-day restore (C)[38] | HTTP-based |
| Cloudflare D1 | 500 MB per database, 5 GB per account, 5M rows read and 100k written per day (C)[20][40] | None stated | Time Travel 7 days (C)[40] | Worker binding |
| CockroachDB | Pricing page shows only a USD 400 / 30-day trial (C)[41] | Paused after trial, deleted after 30 days (C)[41] | n/a | n/a |
| Aiven Postgres | 1 CPU, 1 GB RAM, up to 8 GB disk, no region choice, no card (C)[42] | Powered off when inactive, after a notice (C)[42] | Included (C)[42] | 20, no pooling (C)[42] |
| Xata | No free cloud tier, only a trial (C)[43] | n/a | n/a | n/a |
| Oracle Autonomous | 2 databases, 20 GB each (C)[10] | Stopped after 7 days idle; may be deleted after 90 days stopped (C)[44] | No manual or point-in-time restore (C)[44] | 30 sessions (C)[44] |
| Render Postgres | 1 GB; expires 30 days after creation (C)[13] | n/a | None | n/a |
| Koyeb Postgres | 1 GB, 5 hours active time (C)[16] | n/a | n/a | n/a |

- SQLite on a persistent disk: Oracle VMs (C)[10], Railway's 0.5 GB volume (C)[15], Durable Object SQLite storage, 5 GB (C)[20].
  Not available on Render free (C)[13] or Koyeb free (C)[17].
- Litestream replicates to S3-compatible stores, GCS, Azure Blob, B2, SFTP and others (C)[45].
- Free object storage: R2 10 GB (C)[46], B2 10 GB (C)[47], OCI 20 GB (C)[10], GCS 5 GB in US regions only (C)[1].

## 3. Static frontend

| Host | Limits |
|---|---|
| Cloudflare Pages | 500 builds/month, 20,000 files, 25 MiB per file, 100 custom domains per project (C)[48] |
| Workers static assets | Requests "free and unlimited" (C)[20] |
| GitHub Pages | 1 GB site, 100 GB/month soft limit; not for running an online business (C)[49] |
| Netlify | 300 credits; a production deploy costs 15, bandwidth 20 per GB (C)[50] |
| Vercel Hobby | 100 GB transfer, 1M requests (C)[26] |

Serving the files from the Rust binary works on any container or VM option (P, inf.).
Each asset then counts as a request against that platform's quota, and a sleeping backend delays the first page load (P, inf.).

## 4. Domains

- **Platform hostnames** (P, from general knowledge, not re-verified): `*.run.app`, `*.lambda-url.<region>.on.aws`, `*.azurecontainerapps.io`, `*.onrender.com`, `*.koyeb.app`, `*.workers.dev`, `*.pages.dev`, `*.vercel.app`, `*.netlify.app`, `*.github.io`.
  `*.ts.net` is confirmed (C)[32].
  None gives DNS control (P, inf.).
- **Freenom:** free registration has been closed since 2023; a July 2026 report says it returned as a paid service (P)[51].
- **eu.org:** the site still describes free registration (C)[52]; forum reports describe waits from months to years (U)[53].
- **is-a.dev:** registration by pull request; supports A, CNAME, MX and TXT; NS delegation is restricted; meant for developer personal sites (C)[54][55].
- **DigitalPlat:** offers .dpdns.org, .us.kg, .qzz.io, .xx.kg and .qd.je; custom nameservers allowed; 1 domain per account; policy tightened "due to increased domain abuse" (C)[56][57].
- **Paid, Porkbun renewals in USD:** .de 4.07, .top 4.63, .eu 5.88, .com 11.81 (C)[58].
- **Cloudflare Registrar:** sells at registry cost with no markup (C)[59]; .com is about USD 10.46 (P)[60].

## 5. Transactional email

| Provider | Free tier | Without your own domain |
|---|---|---|
| Resend | 3,000/month, 100/day (C)[61] | No: `resend.dev` sends only to your own address (C)[62] |
| Brevo | 300/day (P)[63] | Free-mailbox or unauthenticated senders are rewritten to `@brevosend.com`, described as a stopgap (P)[64] |
| Mailjet | 6,000/month, 200/day (C)[65] | Not established |
| Mailgun | 100/day, 1 custom domain (C)[66] | Not established |
| Postmark | 100/month, does not expire (C)[67] | Not established |
| MailerSend | 500/month; card required (C)[68] | Not established |
| SMTP2GO | 1,000/month (C)[69] | Not established |
| Mailtrap | 4,000/month, 150/day (C)[70] | Not established |
| Amazon SES | No standing free quota; covered by Free Tier credits; USD 0.10 per 1,000 (C)[71] | A single email-address identity can be verified (C)[72]; DMARC fails for a `gmail.com` From address sent this way (P)[73] |
| Cloudflare Email Service | Sending "not available" on Workers Free; 3,000/month on Paid (C)[74] | No |
| ZeptoMail | First 10,000-email credit free, then USD 2.50 per credit, valid 6 months (P)[75] | Not established |
| OCI Email Delivery | 3,000/month (C)[10] | Not established |
| Gmail SMTP | 500 emails/day; a breach pauses sending for 1–24 h (C)[76] | Yes |
| Outlook.com SMTP | Requires OAuth2; app passwords are reported not to work (P)[77] | Yes |

- **Does verified sending force a domain?**
  Every provider where the mechanism could be confirmed needs DNS records on a domain you control for arbitrary recipients.
  The exceptions found are Brevo's rewrite, an SES address identity, and a personal mailbox over SMTP.
- A NS-delegated free name (DigitalPlat, eu.org) or is-a.dev's TXT/MX records would technically allow SPF/DKIM/DMARC (P, inf.).
  Whether providers accept such names was not established.
- Gmail requires SPF or DKIM from all senders and warns against impersonating Gmail From headers (C)[78].
- Outlook's SPF+DKIM+DMARC rule applies to senders above 5,000/day to consumer mailboxes (C)[79].
- For corporate Microsoft 365, new domains have no reputation and passing authentication does not guarantee the inbox; this rests on vendor and forum sources only (P)[80].

## 6. Verified email without sending mail

- **Google:**
  - `email_verified` is "true if the user's email address has been verified"; `sub` should be the user key (C)[81].
  - With only `openid`, `email` and `profile`, there is no test-user list, no 7-day expiry, and the 100-user cap does not apply (C)[82][83].
  - Brand verification needs a homepage and privacy policy on a domain you own (C)[84].
- **Microsoft:**
  - The `email` claim "isn't guaranteed to be correct"; the optional `xms_edov` claim indicates domain-owner verification (C)[85].
  - Multitenant apps without publisher verification show as "unverified"; verification needs a Cloud Partner Program account (P)[86].
- **GitHub:** `GET /user/emails` returns a `verified` flag and needs the `user:email` scope (C)[87].
- **Passkeys:** the RP ID is bound to the hostname and cannot be a public suffix; changing hostname breaks existing passkeys (C)[88].
  They carry no email claim (P, inf.).

## 7. Abuse protection

- **Turnstile:** free, 20 widgets, unlimited challenges (C)[89].
- **hCaptcha:** Basic plan is USD 0 with no stated request cap (C)[90].
- **Cloudflare rate limiting (Free):** 1 rule, per IP, 10-second window (C)[91].
  It needs a zone on Cloudflare (P, inf.).
- Without that, rate limiting has to live in the app (P, inf.).
  The Workers 10 ms CPU limit constrains password hashing (P, inf.).

## 8. Deployment pipeline

- **GitHub Actions:**
  - Public repos are free on standard runners (C)[92].
  - GitHub Free private repos include USD 12 of compute per month, about 2,000 Linux minutes at USD 0.006/min (inf.), plus 500 MB artifacts and 10 GB cache per repo (C)[92].
- **GHCR:** storage and bandwidth "currently free", with one month's notice of any change (C)[93].
- **Rust build times:** cold builds run from several minutes to 10–15 minutes; rust-cache, sccache or cargo-chef cut this substantially.
  Figures come from vendor blogs (P)[94].
- **Deploying:**
  - Cloud Run: `google-github-actions/deploy-cloudrun` with keyless Workload Identity Federation; `no_traffic`, `tag` and `revision_traffic` give previews and rollback (C)[95].
  - Workers: `wrangler rollback` over the last 100 versions; D1 and KV data are not rolled back (C)[96].
  - Other platforms' actions were not verified.
- **Database recovery:** Neon 10 branches per project and 6-hour history (C)[33]; D1 Time Travel 7 days (C)[40]; Turso 1 day (C)[38]; Supabase Free none (C)[35].

## Unknowns

- SendGrid's current free offering (pricing URLs redirect in a loop).
- Scaleway TEM pricing and several Brevo, Mailjet and SMTP2GO help pages (HTTP 403).
- Whether Mailjet, SMTP2GO, Postmark, Mailgun, MailerSend or Mailtrap allow single-sender sending without a domain.
- Whether email providers accept free subdomains as sender domains.
- Whether CockroachDB still has a perpetual free allowance.
- Turso's current EU regions and whether its archiving policy still applies.
- Supabase's paused-project restore window: the docs text says 1 year, a screenshot shows 90 days.
- Whether Oracle reclaims idle instances on upgraded pay-as-you-go accounts.
- Card requirements for AWS and Cloudflare.
- Azure Container Apps streaming timeouts and Azure Functions Rust support.
- Current eu.org processing time.
- Cloud Run and Lambda cold-start times for Rust.
- Official deploy actions for AWS, Azure, Fly, Render and Koyeb.
- Google's terms on automated Gmail SMTP use.
- Northflank Sandbox regions and volumes.
- Neon's free storage limit (the report was garbled between 0.5 GB and 1 GB).

## Sources

1. https://docs.cloud.google.com/free/docs/free-cloud-features
2. https://docs.cloud.google.com/run/docs/triggering/websockets
3. https://docs.cloud.google.com/run/docs/configuring/request-timeout
4. https://aws.amazon.com/lambda/pricing/
5. https://docs.aws.amazon.com/lambda/latest/dg/configuration-response-streaming.html
6. https://aws.amazon.com/free/
7. https://azure.microsoft.com/en-us/pricing/details/container-apps/
8. https://azure.microsoft.com/en-us/pricing/details/functions/ and the Azure free account FAQ (both via search)
9. https://learn.microsoft.com/en-us/azure/azure-functions/functions-scale
10. https://docs.oracle.com/en-us/iaas/Content/FreeTier/freetier_topic-Always_Free_Resources.htm
11. https://space-node.net/blog/oracle-cloud-always-free-limits-2026
12. https://docs.fly.io/about/pricing
13. https://render.com/docs/free
14. https://render.com/docs/websocket
15. https://docs.railway.com/reference/pricing/plans
16. https://www.koyeb.com/docs/faqs/pricing
17. https://www.koyeb.com/docs/reference/instances
18. https://www.koyeb.com/docs/run-and-scale/scale-to-zero
19. https://docs.shuttle.dev/docs/shuttle-shutdown.md
20. https://developers.cloudflare.com/workers/platform/pricing/
21. https://developers.cloudflare.com/workers/platform/limits/
22. https://developers.cloudflare.com/durable-objects/best-practices/websockets/
23. https://developers.cloudflare.com/workers/languages/rust/
24. https://developers.cloudflare.com/containers/pricing/
25. https://vercel.com/docs/functions/runtimes/rust
26. https://vercel.com/docs/plans/hobby
27. https://www.netlify.com/blog/2021/10/14/write-netlify-functions-in-rust/
28. https://northflank.com/pricing
29. https://zeabur.com/docs/en-US/pricing/free-plan
30. https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/get-started/create-remote-tunnel/
31. https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/do-more-with-tunnels/trycloudflare/
32. https://tailscale.com/kb/1223/funnel
33. https://neon.com/pricing
34. https://neon.com/docs/connect/connection-pooling
35. https://supabase.com/pricing
36. https://supabase.com/docs/guides/platform/upgrading
37. https://supabase.com/docs/guides/platform/compute-and-disk
38. https://turso.tech/pricing
39. https://docs.turso.tech/cli/group/unarchive
40. https://developers.cloudflare.com/d1/platform/limits/
41. https://www.cockroachlabs.com/pricing/
42. https://aiven.io/docs/platform/concepts/free-plan
43. https://xata.io/pricing
44. https://docs.oracle.com/en-us/iaas/autonomous-database-serverless/doc/autonomous-always-free.html
45. https://litestream.io/guides/
46. https://developers.cloudflare.com/r2/pricing/
47. https://www.backblaze.com/cloud-storage/pricing
48. https://developers.cloudflare.com/pages/platform/limits/
49. https://docs.github.com/en/pages/getting-started-with-github-pages/github-pages-limits
50. https://www.netlify.com/pricing/
51. https://domainincite.com/31821-freenom-is-back-but-no-longer-free
52. https://nic.eu.org/
53. https://forum.infinityfree.com/t/time-taken-to-get-eu-org-domain-approval/92362
54. https://github.com/is-a-dev/register
55. https://docs.is-a.dev/faq/
56. https://github.com/DigitalPlatDev/FreeDomain
57. https://github.com/DigitalPlatDev/FreeDomain/blob/main/documents/domains/faq.md
58. https://porkbun.com/products/domains
59. https://www.cloudflare.com/products/registrar/
60. https://www.whtop.com/plans/cloudflare.com/135912
61. https://resend.com/pricing
62. https://resend.com/docs/knowledge-base/403-error-resend-dev-domain
63. https://www.brevo.com/features/email-api/
64. https://help.brevo.com/hc/en-us/articles/16045394674066 (via search; direct fetch returned 403)
65. https://www.mailjet.com/pricing/
66. https://www.mailgun.com/pricing/
67. https://postmarkapp.com/pricing
68. https://www.mailersend.com/pricing
69. https://www.smtp2go.com/pricing/
70. https://mailtrap.io/pricing/
71. https://aws.amazon.com/ses/pricing/
72. https://docs.aws.amazon.com/ses/latest/dg/creating-identities.html
73. https://www.gmass.co/blog/send-from-gmail-through-external-smtp-server/
74. https://developers.cloudflare.com/email-service/platform/pricing/
75. https://www.zoho.com/zeptomail/pricing.html (via search)
76. https://support.google.com/mail/answer/22839
77. https://learn.microsoft.com/en-us/answers/questions/5695252/outlook-com-authentication
78. https://support.google.com/a/answer/81126
79. https://techcommunity.microsoft.com/blog/microsoftdefenderforoffice365blog/strengthening-email-ecosystem-outlook%E2%80%99s-new-requirements-for-high%E2%80%90volume-senders/4399730
80. https://learn.microsoft.com/en-us/answers/questions/5788143/deliverability-issue-valid-transactional-emails-be
81. https://developers.google.com/identity/openid-connect/openid-connect
82. https://support.google.com/cloud/answer/15549945
83. https://support.google.com/cloud/answer/7454865
84. https://support.google.com/cloud/answer/13464321
85. https://learn.microsoft.com/en-us/entra/identity-platform/optional-claims-reference
86. https://learn.microsoft.com/en-us/entra/identity-platform/howto-configure-publisher-domain (via search)
87. https://docs.github.com/en/rest/users/emails
88. https://web.dev/articles/webauthn-rp-id
89. https://developers.cloudflare.com/turnstile/plans/
90. https://www.hcaptcha.com/pricing
91. https://developers.cloudflare.com/waf/rate-limiting-rules/
92. https://docs.github.com/en/billing/concepts/product-billing/github-actions
93. https://docs.github.com/en/billing/concepts/product-billing/github-packages
94. https://depot.dev/blog/guide-to-faster-rust-builds-in-ci
95. https://github.com/google-github-actions/deploy-cloudrun
96. https://developers.cloudflare.com/workers/configuration/versions-and-deployments/rollbacks/
