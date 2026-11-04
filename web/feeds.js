// Real activity => food. Each feed returns a weight (roughly 0.4 to 2.0) or null.
import { GITHUB_USER, WORK_ORG, SIDE_REPOS, NOTES_FEED } from './config.js';

const clamp = (v, a, b) => Math.max(a, Math.min(b, v));

let recentPushes = null;

async function pushesByKind() {
  if (!GITHUB_USER) return null;
  recentPushes ??= fetch(`https://api.github.com/users/${GITHUB_USER}/events/public?per_page=100`).then(async (res) => {
    if (!res.ok) return null;
    const since = Date.now() - 14 * 864e5;
    const pushes = (await res.json()).filter((e) => e.type === 'PushEvent' && Date.parse(e.created_at) > since);
    const named = (e) => e.repo.name.toLowerCase();
    const work = pushes.filter((e) => named(e).startsWith(`${WORK_ORG.toLowerCase()}/`)).length;
    const side = pushes.filter((e) => SIDE_REPOS.some((r) => r.toLowerCase() === named(e))).length;
    return { work, side };
  });
  return recentPushes;
}

export const feeds = {
  async github() {
    const counts = await pushesByKind();
    return counts && clamp(0.5 + counts.work / 15, 0.4, 2.0);
  },

  async sideRepos() {
    const counts = await pushesByKind();
    return counts && clamp(0.5 + counts.side / 15, 0.4, 2.0);
  },

  // Fresher notes are more nutritious: full food on the day you post, fading over ~2 months.
  async notes() {
    if (!NOTES_FEED) return null;
    const res = await fetch(NOTES_FEED);
    if (!res.ok) return null;
    const posts = await res.json();
    if (!posts.length) return 0.4; // nothing released yet: a hungry, thin node
    const newest = Math.max(...posts.map((p) => Date.parse(p.date)));
    const days = (Date.now() - newest) / 864e5;
    return clamp(2.0 * Math.exp(-days / 60), 0.4, 2.0);
  },
};

export async function readFeeds(pages, onWeight) {
  for (const [i, page] of pages.entries()) {
    if (!page.feed || !feeds[page.feed]) continue;
    try {
      const w = await feeds[page.feed]();
      if (w != null) onWeight(i, w);
    } catch {
      // Offline or rate limited: the organism keeps its default appetite.
    }
  }
}
