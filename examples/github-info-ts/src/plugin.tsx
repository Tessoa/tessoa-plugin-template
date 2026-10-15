// Example plugin: a repository's numbers from GitHub's API.
//
// The user writes a repository as owner/name, or pastes its address on
// github.com, and the page asks GitHub's API about it: stars, forks, open
// issues, language, licence, the last push. The plugin may connect only to
// the websites its manifest lists, here https://api.github.com; tessoa shows
// that list to the user when the plugin is installed, and the runner makes
// the requests. No other permission is needed.
//
// The page's handler awaits the answer and the page keeps working meanwhile.
// A second look-up abandons the first with `tessoa.net.cancel`, whose answer
// then never arrives. A status such as 404 is an answer, not an error; what
// throws is a request that got no answer (`timeout`, `failed`) or that the
// runner refused (`not-allowed`).
//
// The link to the repository is rich text: tessoa shows the user the whole
// address and opens it only when they agree. The page saves the repository it
// shows, so it shows it again after a restart; Refresh asks GitHub again.

interface Repo {
  /** owner/name, as GitHub writes it. */
  name: string;
  description: string;
  stars: number;
  forks: number;
  issues: number;
  language: string;
  licence: string;
  /** The day of the last push, YYYY-MM-DD. */
  pushed: string;
  archived: boolean;
}

interface Info {
  /** What the field holds. */
  query: string;
  repo: Repo | undefined;
  status: string;
  /** The request under way: a newer one abandons it. */
  request: Promise<tessoa.net.Response> | undefined;
}

/** What `save` keeps. */
interface Saved {
  query: string;
  repo?: Repo;
}

const API = "https://api.github.com/repos/";

/**
 * owner/name from what the user wrote: owner/name, or an address of the
 * repository on github.com, whatever follows the name in it.
 */
function repoName(text: string): string | undefined {
  let t = text.trim();
  const site = /^(https?:\/\/)?(www\.)?github\.com\//i;
  const address = site.test(t);
  t = t.replace(site, "").replace(/\/+$/, "");
  const parts = t.split("/");
  if (parts.length < 2 || (!address && parts.length > 2)) return undefined;
  const owner = parts[0];
  const name = parts[1].replace(/\.git$/i, "");
  if (!/^[A-Za-z0-9][A-Za-z0-9-]{0,38}$/.test(owner)) return undefined;
  if (!/^[A-Za-z0-9._-]{1,100}$/.test(name) || name === "." || name === "..") return undefined;
  return `${owner}/${name}`;
}

/** A repository from the JSON GitHub answers, whatever is missing from it. */
function readRepo(name: string, data: any): Repo {
  const text = (v: unknown) => (typeof v === "string" ? v : "");
  const count = (v: unknown) => (typeof v === "number" && Number.isFinite(v) ? v : 0);
  const spdx = text(data?.license?.spdx_id);
  return {
    name: text(data?.full_name) || name,
    description: text(data?.description),
    stars: count(data?.stargazers_count),
    forks: count(data?.forks_count),
    issues: count(data?.open_issues_count),
    language: text(data?.language),
    // GitHub writes NOASSERTION for a licence it does not recognise.
    licence: spdx === "NOASSERTION" ? text(data?.license?.name) : spdx,
    pushed: text(data?.pushed_at).slice(0, 10),
    archived: data?.archived === true,
  };
}

/** 12345 as "12,345". */
function grouped(n: number): string {
  return String(Math.round(n)).replace(/\B(?=(\d{3})+(?!\d))/g, ",");
}

/** What to tell the user about a request that got no answer. */
function failure(e: unknown): string {
  switch ((e as Partial<tessoa.net.NetError>).kind) {
    case "timeout":
      return "GitHub did not answer in time. Try again.";
    case "not-allowed":
      return `The plugin may not connect there: ${(e as Error).message}`;
    default:
      return `Could not reach GitHub: ${(e as Error).message}`;
  }
}

async function lookUp(s: Info, text: string): Promise<void> {
  const name = repoName(text);
  if (name === undefined) {
    s.status = "Write the repository as owner/name, or paste its address on github.com.";
    return;
  }
  if (s.request !== undefined) tessoa.net.cancel(s.request);
  const request = tessoa.net.fetch(API + name, {
    headers: { Accept: "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28" },
    timeoutMs: 15_000,
  });
  s.request = request;
  s.status = `Looking up ${name}…`;
  // A failed look-up of another repository leaves nothing on the page; a
  // failed refresh leaves the numbers the page has.
  const forget = () => {
    if (s.repo?.name.toLowerCase() !== name.toLowerCase()) s.repo = undefined;
  };
  let answer: tessoa.net.Response;
  try {
    answer = await request;
  } catch (e) {
    s.request = undefined;
    s.status = failure(e);
    forget();
    return;
  }
  s.request = undefined;
  if (answer.ok) {
    s.repo = readRepo(name, await answer.json());
    s.status = "";
    return;
  }
  forget();
  if (answer.status === 404) {
    s.status = `GitHub has no public repository ${name}.`;
  } else if ((answer.status === 403 || answer.status === 429) && answer.header("x-ratelimit-remaining") === "0") {
    s.status = "GitHub's limit of requests without signing in is used up. Try again later.";
  } else {
    s.status = `GitHub answered ${answer.status}.`;
  }
}

function RepoCard({ repo, s }: { repo: Repo; s: Info }) {
  return (
    <card key="card" pad="m" gap="s">
      <row gap="s" align="center">
        <text key="name" size="title" grow>
          {repo.name}
        </text>
        {repo.archived && (
          <text key="archived" tone="dim">
            Archived
          </text>
        )}
        <button key="refresh" icon="reload" onClick={() => lookUp(s, repo.name)}>
          Refresh
        </button>
      </row>
      <text key="description" tone={repo.description ? "normal" : "dim"} wrap>
        {repo.description || "No description."}
      </text>
      <grid key="numbers" columns={2} gap="s">
        <text tone="dim">Stars</text>
        <text key="stars">{grouped(repo.stars)}</text>
        <text tone="dim">Forks</text>
        <text key="forks">{grouped(repo.forks)}</text>
        <text tone="dim">Open issues</text>
        <text key="issues">{grouped(repo.issues)}</text>
        <text tone="dim">Language</text>
        <text key="language">{repo.language || "-"}</text>
        <text tone="dim">Licence</text>
        <text key="licence">{repo.licence || "-"}</text>
        <text tone="dim">Last push</text>
        <text key="pushed">{repo.pushed || "-"}</text>
      </grid>
      <rich-text key="link">
        See it on <a href={`https://github.com/${repo.name}`}>GitHub</a>.
      </rich-text>
    </card>
  );
}

tessoa.pages.define<Info>("github", {
  open: ({ state }) => {
    const saved = state as Partial<Saved> | undefined;
    return {
      query: typeof saved?.query === "string" ? saved.query : "",
      repo: typeof saved?.repo?.name === "string" ? saved.repo : undefined,
      status: "",
      request: undefined,
    };
  },
  save: (s): Saved => ({ query: s.query, repo: s.repo }),
  title: (s) => s.repo?.name,
  render: (s) => (
    <column pad="m" gap="m">
      <row gap="s" align="center">
        <text-input
          key="repo"
          grow
          value={s.query}
          placeholder="owner/name, or the repository's address"
          onInput={(text) => {
            s.query = text;
          }}
          onSubmit={(text) => lookUp(s, text)}
        />
        <button key="look-up" kind="primary" icon="search" disabled={s.query.trim() === ""} onClick={() => lookUp(s, s.query)}>
          Look up
        </button>
      </row>
      <text key="status" tone="dim" wrap>
        {s.status}
      </text>
      {s.repo && <RepoCard repo={s.repo} s={s} />}
    </column>
  ),
});
