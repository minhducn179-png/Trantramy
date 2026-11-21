<script lang="ts">
  import { RefreshCw, Pause, Play, Heart, Download, ChevronDown, Check } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { api, fmtTime, getTrackCover, handleImgError, type OnlineTrack, type Track } from "$lib/api";
  import { player } from "$lib/player.svelte";
  import WaveformBar from "./WaveformBar.svelte";

  let {
    downloads = {},
    savedUrls = new Set<string>(),
    downloadedUrls = new Set<string>(),
    onsave,
    ondownload,
    onopen,
    onclear,
  }: {
    downloads?: Record<string, number>;
    savedUrls?: Set<string>;
    downloadedUrls?: Set<string>;
    onsave: (t: OnlineTrack) => void;
    ondownload: (t: OnlineTrack) => void;
    onopen?: (t: Track) => void;
    onclear?: () => void;
  } = $props();

  const MOOD_CATEGORIES = [
    {
      id: "for-you",
      label: "Dành cho bạn",
      query: "nhạc trẻ việt nam hot hits official audio",
      queries: [
        "nhạc trẻ việt nam hot hits official audio",
        "top hits vietnam international music 2026",
        "bài hát được nghe nhiều nhất hiện nay",
      ],
    },
    {
      id: "trending",
      label: "Thịnh hành",
      query: "top trending music hits 2026",
      queries: [
        "top trending music hits 2026",
        "nhạc thịnh hành hot trend tiktok youtube 2026",
        "global top 50 hits official audio",
      ],
    },
    {
      id: "chill",
      label: "Chill & Lo-Fi",
      query: "lofi chill beats study relax",
      queries: [
        "lofi chill beats study relax",
        "nhạc chill nhẹ nhàng thư giãn đêm khuya",
        "chillhop instrumental lofi hip hop",
      ],
    },
    {
      id: "remix",
      label: "Remix & EDM",
      query: "edm house remix bass boosted",
      queries: [
        "edm house remix bass boosted",
        "nhạc trẻ remix sôi động vinahouse mới nhất",
        "festival edm club mix electro house",
      ],
    },
    {
      id: "vpop",
      label: "Nhạc Việt Hot",
      query: "nhạc trẻ việt nam hot trend acoustic",
      queries: [
        "nhạc trẻ việt nam hot trend official mv",
        "vpop bảng xếp hạng nhạc việt mới nhất",
        "nhạc tình ca việt nam hay nhất",
      ],
    },
    {
      id: "ballad",
      label: "Ballad & Acoustic",
      query: "nhạc ballad acoustic nhẹ nhàng sâu lắng",
      queries: [
        "nhạc ballad acoustic nhẹ nhàng sâu lắng",
        "acoustic guitar cover nhạc việt buồn chill",
        "love ballad piano vocal songs",
      ],
    },
    {
      id: "indie",
      label: "Indie & Underground",
      query: "indie chill underground Vietnamese international",
      queries: [
        "indie việt nam underground chill hay nhất",
        "indie pop bedroom pop alternative chill",
        "rap melodic underground việt nam",
      ],
    },
    {
      id: "usuk",
      label: "US-UK Pop/R&B",
      query: "us uk pop rnb hits",
      queries: [
        "us uk pop rnb hits official audio",
        "billboard hot 100 pop r&b hits",
        "best english pop songs chill vibes",
      ],
    },
  ];

  const STARTER_TRACKS: Record<string, OnlineTrack[]> = {
    "for-you": [
      {
        title: "Chúng Ta Của Tương Lai",
        artist: "Sơn Tùng M-TP",
        album: "M-TP Entertainment",
        duration: 251,
        thumbnail: "https://i.ytimg.com/vi/q_v2u6oMvK0/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=q_v2u6oMvK0",
        source: "youtube",
        reason: "Bản hit V-Pop nổi bật",
      },
      {
        title: "Từng Là",
        artist: "Vũ Cát Tường",
        album: "Single",
        duration: 228,
        thumbnail: "https://i.ytimg.com/vi/lJ_Zq6b9U64/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=lJ_Zq6b9U64",
        source: "youtube",
        reason: "Giai điệu cảm xúc",
      },
      {
        title: "Nơi Này Có Anh",
        artist: "Sơn Tùng M-TP",
        album: "M-TP",
        duration: 260,
        thumbnail: "https://i.ytimg.com/vi/FN7ALfpGxiI/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=FN7ALfpGxiI",
        source: "youtube",
        reason: "Bản tình ca được yêu thích",
      },
      {
        title: "Lạ Lùng",
        artist: "Vũ.",
        album: "Vũ Official",
        duration: 262,
        thumbnail: "https://i.ytimg.com/vi/F5tS5m86bOI/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=F5tS5m86bOI",
        source: "youtube",
        reason: "Acoustic nhẹ nhàng thư giãn",
      },
      {
        title: "Tháng Tư Là Lời Nói Dối Của Em",
        artist: "Hà Anh Tuấn",
        album: "Fragile",
        duration: 315,
        thumbnail: "https://i.ytimg.com/vi/UCXao7aTDtw/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=UCXao7aTDtw",
        source: "youtube",
        reason: "Giai điệu hoài niệm",
      },
      {
        title: "Save Your Tears",
        artist: "The Weeknd",
        album: "After Hours",
        duration: 215,
        thumbnail: "https://i.ytimg.com/vi/XXYlFuWEuKi/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=XXYlFuWEuKi",
        source: "youtube",
        reason: "Top Hits Toàn cầu",
      },
      {
        title: "Blinding Lights",
        artist: "The Weeknd",
        album: "After Hours",
        duration: 200,
        thumbnail: "https://i.ytimg.com/vi/4NRXx6U8ABQ/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=4NRXx6U8ABQ",
        source: "youtube",
        reason: "Synthwave sôi động",
      },
      {
        title: "Starboy",
        artist: "The Weeknd ft. Daft Punk",
        album: "Starboy",
        duration: 230,
        thumbnail: "https://i.ytimg.com/vi/34Na4j8AVgA/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=34Na4j8AVgA",
        source: "youtube",
        reason: "Electronic & R&B",
      },
    ],
    trending: [
      {
        title: "Die With A Smile",
        artist: "Lady Gaga, Bruno Mars",
        album: "Trending Hits",
        duration: 251,
        thumbnail: "https://i.ytimg.com/vi/kPa7bsKwL-c/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=kPa7bsKwL-c",
        source: "youtube",
        reason: "Thịnh hành",
      },
      {
        title: "APT.",
        artist: "ROSÉ & Bruno Mars",
        album: "Trending Hits",
        duration: 170,
        thumbnail: "https://i.ytimg.com/vi/ekr2nIex040/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=ekr2nIex040",
        source: "youtube",
        reason: "Thịnh hành",
      },
      {
        title: "Birds of a Feather",
        artist: "Billie Eilish",
        album: "Hit Me Hard and Soft",
        duration: 210,
        thumbnail: "https://i.ytimg.com/vi/V9PVRfjEBTI/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=V9PVRfjEBTI",
        source: "youtube",
        reason: "Thịnh hành",
      },
      {
        title: "Espresso",
        artist: "Sabrina Carpenter",
        album: "Short n' Sweet",
        duration: 175,
        thumbnail: "https://i.ytimg.com/vi/eVli-tstM5E/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=eVli-tstM5E",
        source: "youtube",
        reason: "Thịnh hành",
      },
      {
        title: "Đừng Làm Trái Tim Anh Đau",
        artist: "Sơn Tùng M-TP",
        album: "M-TP Entertainment",
        duration: 279,
        thumbnail: "https://i.ytimg.com/vi/abPmZCZZrFA/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=abPmZCZZrFA",
        source: "youtube",
        reason: "Thịnh hành",
      },
      {
        title: "Cruel Summer",
        artist: "Taylor Swift",
        album: "Lover",
        duration: 178,
        thumbnail: "https://i.ytimg.com/vi/ic8j13piAhQ/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=ic8j13piAhQ",
        source: "youtube",
        reason: "Thịnh hành",
      },
    ],
    chill: [
      {
        title: "Synthwave Radio — Chill Retro Beats",
        artist: "Lofi Girl",
        album: "Chill & Lo-Fi",
        duration: 240,
        thumbnail: "https://i.ytimg.com/vi/4xDzrJKXOOY/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=4xDzrJKXOOY",
        source: "youtube",
        reason: "Chill & Lo-Fi",
      },
      {
        title: "Coffee Shop Radio — Chill Lofi Hip Hop Beats",
        artist: "STEEZYASFUCK",
        album: "Chill & Lo-Fi",
        duration: 215,
        thumbnail: "https://i.ytimg.com/vi/-5KAN9_CzSA/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=-5KAN9_CzSA",
        source: "youtube",
        reason: "Chill & Lo-Fi",
      },
      {
        title: "Snowman",
        artist: "WYS",
        album: "1 Am. Study Session",
        duration: 195,
        thumbnail: "https://i.ytimg.com/vi/lTRiuFIWV54/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=lTRiuFIWV54",
        source: "youtube",
        reason: "Chill & Lo-Fi",
      },
      {
        title: "5:32pm",
        artist: "The Deli",
        album: "Vibes 2",
        duration: 132,
        thumbnail: "https://i.ytimg.com/vi/2DVpys50LVE/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=2DVpys50LVE",
        source: "youtube",
        reason: "Chill & Lo-Fi",
      },
    ],
    remix: [
      {
        title: "Faded",
        artist: "Alan Walker",
        album: "Different World",
        duration: 212,
        thumbnail: "https://i.ytimg.com/vi/60ItHLz5WEA/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=60ItHLz5WEA",
        source: "youtube",
        reason: "Remix & EDM",
      },
      {
        title: "Wake Me Up",
        artist: "Avicii",
        album: "True",
        duration: 247,
        thumbnail: "https://i.ytimg.com/vi/IcrbM1l_BoI/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=IcrbM1l_BoI",
        source: "youtube",
        reason: "Remix & EDM",
      },
      {
        title: "Animals",
        artist: "Martin Garrix",
        album: "Gold Skies",
        duration: 191,
        thumbnail: "https://i.ytimg.com/vi/gCYcHz2k5x0/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=gCYcHz2k5x0",
        source: "youtube",
        reason: "Remix & EDM",
      },
      {
        title: "Titanium (feat. Sia)",
        artist: "David Guetta",
        album: "Nothing but the Beat",
        duration: 245,
        thumbnail: "https://i.ytimg.com/vi/JRfuAukYTKg/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=JRfuAukYTKg",
        source: "youtube",
        reason: "Remix & EDM",
      },
    ],
    vpop: [
      {
        title: "Đừng Làm Trái Tim Anh Đau",
        artist: "Sơn Tùng M-TP",
        album: "V-Pop",
        duration: 279,
        thumbnail: "https://i.ytimg.com/vi/abPmZCZZrFA/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=abPmZCZZrFA",
        source: "youtube",
        reason: "Nhạc Việt Hot",
      },
      {
        title: "Chúng Ta Của Hiện Tại",
        artist: "Sơn Tùng M-TP",
        album: "V-Pop",
        duration: 301,
        thumbnail: "https://i.ytimg.com/vi/psZ1g9fMfeo/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=psZ1g9fMfeo",
        source: "youtube",
        reason: "Nhạc Việt Hot",
      },
      {
        title: "Có Chắc Yêu Là Đây",
        artist: "Sơn Tùng M-TP",
        album: "V-Pop",
        duration: 215,
        thumbnail: "https://i.ytimg.com/vi/6t-MjBazs3o/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=6t-MjBazs3o",
        source: "youtube",
        reason: "Nhạc Việt Hot",
      },
      {
        title: "Hãy Trao Cho Anh",
        artist: "Sơn Tùng M-TP ft. Snoop Dogg",
        album: "V-Pop",
        duration: 245,
        thumbnail: "https://i.ytimg.com/vi/knW7-x7Y7RE/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=knW7-x7Y7RE",
        source: "youtube",
        reason: "Nhạc Việt Hot",
      },
    ],
    ballad: [
      {
        title: "Tháng Tư Là Lời Nói Dối Của Em",
        artist: "Hà Anh Tuấn",
        album: "Fragile",
        duration: 315,
        thumbnail: "https://i.ytimg.com/vi/UCXao7aTDtw/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=UCXao7aTDtw",
        source: "youtube",
        reason: "Ballad & Acoustic",
      },
      {
        title: "Từng Là",
        artist: "Vũ Cát Tường",
        album: "Single",
        duration: 228,
        thumbnail: "https://i.ytimg.com/vi/lJ_Zq6b9U64/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=lJ_Zq6b9U64",
        source: "youtube",
        reason: "Ballad & Acoustic",
      },
      {
        title: "Nàng Thơ",
        artist: "Hoàng Dũng",
        album: "25",
        duration: 254,
        thumbnail: "https://i.ytimg.com/vi/Zzn9-ATB9aU/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=Zzn9-ATB9aU",
        source: "youtube",
        reason: "Ballad & Acoustic",
      },
    ],
    indie: [
      {
        title: "Lạ Lùng",
        artist: "Vũ.",
        album: "Indie Việt",
        duration: 262,
        thumbnail: "https://i.ytimg.com/vi/F5tS5m86bOI/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=F5tS5m86bOI",
        source: "youtube",
        reason: "Indie & Underground",
      },
      {
        title: "Bước Qua Nhau",
        artist: "Vũ.",
        album: "Một Vạn Năm",
        duration: 257,
        thumbnail: "https://i.ytimg.com/vi/ixdSsW6n_rI/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=ixdSsW6n_rI",
        source: "youtube",
        reason: "Indie & Underground",
      },
      {
        title: "Một Đêm Say",
        artist: "Thịnh Suy",
        album: "Indie Việt",
        duration: 215,
        thumbnail: "https://i.ytimg.com/vi/dQw4w9WgXcQ/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=X0t1I8T98mE",
        source: "youtube",
        reason: "Indie & Underground",
      },
    ],
    usuk: [
      {
        title: "Blinding Lights",
        artist: "The Weeknd",
        album: "After Hours",
        duration: 200,
        thumbnail: "https://i.ytimg.com/vi/4NRXx6U8ABQ/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=4NRXx6U8ABQ",
        source: "youtube",
        reason: "US-UK Pop/R&B",
      },
      {
        title: "Save Your Tears",
        artist: "The Weeknd",
        album: "After Hours",
        duration: 215,
        thumbnail: "https://i.ytimg.com/vi/XXYlFuWEuKi/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=XXYlFuWEuKi",
        source: "youtube",
        reason: "US-UK Pop/R&B",
      },
      {
        title: "Starboy",
        artist: "The Weeknd ft. Daft Punk",
        album: "Starboy",
        duration: 230,
        thumbnail: "https://i.ytimg.com/vi/34Na4j8AVgA/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=34Na4j8AVgA",
        source: "youtube",
        reason: "US-UK Pop/R&B",
      },
      {
        title: "Levitating",
        artist: "Dua Lipa",
        album: "Future Nostalgia",
        duration: 203,
        thumbnail: "https://i.ytimg.com/vi/TUVcZfQe-Kw/hqdefault.jpg",
        url: "https://www.youtube.com/watch?v=TUVcZfQe-Kw",
        source: "youtube",
        reason: "US-UK Pop/R&B",
      },
    ],
  };

  function getCachedMood(moodId: string): OnlineTrack[] | null {
    if (typeof localStorage === "undefined") return null;
    try {
      const raw = localStorage.getItem("dlc.discovery.mood.v2." + moodId);
      if (raw) {
        const parsed = JSON.parse(raw);
        if (Array.isArray(parsed) && parsed.length > 0) return parsed;
      }
    } catch {
      // ignore
    }
    return null;
  }

  function setCachedMood(moodId: string, data: OnlineTrack[]) {
    if (typeof localStorage === "undefined") return;
    try {
      localStorage.setItem("dlc.discovery.mood.v2." + moodId, JSON.stringify(data));
    } catch {
      // ignore
    }
  }

  let activeMood = $state("for-you");
  let customStyleLabel = $state<string | null>(null);
  let customStyleQuery = $state<string | null>(null);
  let moodVariantIndex = $state<Record<string, number>>({});
  let viewLayout = $state<"both" | "spotify" | "soundcloud">("both");
  let items = $state<OnlineTrack[]>(getCachedMood("for-you") || STARTER_TRACKS["for-you"] || []);
  let loading = $state(false);
  let err = $state("");
  let moodReqSeq = 0;

  const activeCategoryLabel = $derived(
    customStyleLabel ||
      MOOD_CATEGORIES.find((m) => m.id === activeMood)?.label ||
      "Dành cho bạn",
  );

  const quickPicks = $derived(items.slice(0, 6));
  const shelfItems = $derived(items.slice(0, 10));
  const streamItems = $derived(items);

  async function selectMoodButton(moodId: string) {
    onclear?.();
    const isRepeatClick = activeMood === moodId && !customStyleLabel;
    await loadMood(moodId, isRepeatClick);
  }

  async function searchByStyleTag(rawReason: string) {
    const cleaned = rawReason
      .replace(/^Chủ đề:\s*/i, "")
      .replace(/^Khám phá thêm:\s*/i, "")
      .trim();
    if (!cleaned) return;
    onclear?.();
    const matchedCat = MOOD_CATEGORIES.find(
      (m) => m.label.toLowerCase() === cleaned.toLowerCase(),
    );
    if (matchedCat) {
      await loadMood(matchedCat.id, true);
      return;
    }
    const reqId = ++moodReqSeq;
    customStyleLabel = cleaned;
    customStyleQuery = `${cleaned} music hits official audio`;
    loading = true;
    err = "";
    try {
      const res = await api.searchOnline(customStyleQuery, "all", 20);
      if (reqId !== moodReqSeq) return;
      if (res.length > 0) {
        items = res.map((r) => ({
          ...r,
          reason: cleaned,
        }));
      }
    } catch (e) {
      if (reqId !== moodReqSeq) return;
      err = String(e);
    } finally {
      if (reqId === moodReqSeq) {
        loading = false;
      }
    }
  }

  async function loadMood(moodId: string, force = false) {
    const reqId = ++moodReqSeq;
    activeMood = moodId;
    customStyleLabel = null;
    customStyleQuery = null;
    err = "";

    // 1. Switch immediately to this style's own tracks (never mix another style's tracks!)
    const cached = !force ? getCachedMood(moodId) : null;
    const starter = STARTER_TRACKS[moodId] || [];
    let currentMoodItems: OnlineTrack[] = cached && cached.length > 0 ? cached : starter;
    items = currentMoodItems;
    loading = true;

    // 2. Actively search online for tracks matching this music style
    try {
      const cat = MOOD_CATEGORIES.find((m) => m.id === moodId);
      let fresh: OnlineTrack[] = [];
      if (!cat) {
        fresh = await api.getRecommendations(force);
      } else {
        const queries = cat.queries && cat.queries.length > 0 ? cat.queries : [cat.query];
        const prevIdx = moodVariantIndex[moodId] ?? 0;
        const nextIdx = force ? (prevIdx + 1) % queries.length : prevIdx % queries.length;
        moodVariantIndex[moodId] = nextIdx;
        const searchQuery = queries[nextIdx] || cat.query;

        if (moodId === "for-you" && !force) {
          const recs = await api.getRecommendations(false).catch(() => []);
          if (recs.length > 0) {
            fresh = recs;
          } else {
            const res = await api.searchOnline(searchQuery, "all", 20);
            fresh = res.map((r) => ({ ...r, reason: cat.label }));
          }
        } else {
          const res = await api.searchOnline(searchQuery, "all", 20);
          fresh = res.map((r) => ({
            ...r,
            reason: cat.label,
          }));
        }
      }
      if (reqId !== moodReqSeq) return;
      if (fresh.length > 0) {
        // Put newly searched songs of this style FIRST so the user sees them immediately
        items = mergeUnique(fresh, currentMoodItems);
        setCachedMood(moodId, items);
      }
    } catch (e) {
      if (reqId !== moodReqSeq) return;
      if (items.length === 0) {
        err = String(e);
      }
    } finally {
      if (reqId === moodReqSeq) {
        loading = false;
      }
    }
  }

  let loadingMore = $state(false);

  function mergeUnique(existing: OnlineTrack[], incoming: OnlineTrack[]): OnlineTrack[] {
    const urls = new Set(existing.map((x) => x.url));
    const added: OnlineTrack[] = [];
    for (const item of incoming) {
      if (!urls.has(item.url)) {
        urls.add(item.url);
        added.push(item);
      }
    }
    return [...existing, ...added];
  }

  async function loadMoreTracks() {
    if (loadingMore) return;
    loadingMore = true;
    try {
      const cat = MOOD_CATEGORIES.find((m) => m.id === activeMood);
      const query =
        customStyleQuery ||
        cat?.query ||
        "trending hits music vietnam international 2026";
      const targetCount = items.length + 14;
      const res = await api.searchOnline(query, "all", targetCount);
      if (res && res.length > 0) {
        const label = customStyleLabel || cat?.label || "Đề xuất";
        const enriched = res.map((r) => ({
          ...r,
          reason: r.reason || label,
        }));
        items = mergeUnique(items, enriched);
        if (!customStyleLabel) {
          setCachedMood(activeMood, items);
        }
      }
    } catch {
      // ignore
    } finally {
      loadingMore = false;
    }
  }

  function playAndInspect(item: OnlineTrack, index: number) {
    const isCurrent = player.current?.url === item.url;
    if (isCurrent) {
      player.toggle();
      return;
    }
    player.playOnlineList(items, index);
    onopen?.({
      id: -1 - index,
      path: item.url,
      title: item.title,
      artist: item.artist,
      album: item.album,
      duration: item.duration,
      cover_path: item.thumbnail,
      source: item.source,
      url: item.url,
    });
  }

  function openTrackDetail(item: OnlineTrack, index: number) {
    player.playOnlineList(items, index);
    onopen?.({
      id: -1 - index,
      path: item.url,
      title: item.title,
      artist: item.artist,
      album: item.album,
      duration: item.duration,
      cover_path: item.thumbnail,
      source: item.source,
      url: item.url,
    });
  }

  onMount(() => {
    loadMood("for-you", false);
  });
</script>

<section class="discovery-hub">
  <!-- Top Filter Bar (Spotify / SoundCloud Category Chips + Layout Switcher) -->
  <div class="hub-Controls-bar">
    <div class="mood-chips" role="tablist" aria-label="Chủ đề âm nhạc">
      {#each MOOD_CATEGORIES as cat (cat.id)}
        <button
          type="button"
          class="mood-pill"
          class:active={!customStyleLabel && activeMood === cat.id}
          onclick={() => selectMoodButton(cat.id)}
          title="Tìm kiếm các bài nhạc phong cách {cat.label}"
        >
          {cat.label}
        </button>
      {/each}
      {#if customStyleLabel}
        <button
          type="button"
          class="mood-pill active"
          onclick={() => searchByStyleTag(customStyleLabel!)}
          title="Đang lọc theo phong cách {customStyleLabel}"
        >
          #{customStyleLabel}
        </button>
      {/if}
    </div>

    <div class="right-tools">
      <div class="layout-switch" role="group" aria-label="Chế độ hiển thị đề xuất">
        <button
          type="button"
          class:active={viewLayout === "both"}
          onclick={() => (viewLayout = "both")}
          title="Hiển thị đầy đủ thẻ bài hát và luồng sóng nhạc"
        >
          Tất cả
        </button>
        <button
          type="button"
          class:active={viewLayout === "spotify"}
          onclick={() => (viewLayout = "spotify")}
          title="Hiển thị dạng thẻ bài hát"
        >
          Thẻ bài hát
        </button>
        <button
          type="button"
          class:active={viewLayout === "soundcloud"}
          onclick={() => (viewLayout = "soundcloud")}
          title="Hiển thị dạng luồng sóng nhạc"
        >
          Sóng nhạc
        </button>
      </div>

      <button
        type="button"
        class="refresh-pill"
        onclick={() =>
          customStyleLabel
            ? searchByStyleTag(customStyleLabel)
            : loadMood(activeMood, true)}
        disabled={loading}
        title="Tìm thêm danh sách nhạc mới cùng phong cách"
      >
        <RefreshCw size={13} class={loading ? "spin" : ""} />
        <span>{loading ? `Đang tìm ${activeCategoryLabel}…` : "Làm mới"}</span>
      </button>
    </div>
  </div>

  {#if err}
    <div class="notice error">{err}</div>
  {:else if loading && items.length === 0}
    <!-- Skeletons matching Spotify Shelf + SoundCloud Stream -->
    <div class="sk-shelf">
      {#each Array(5) as _}
        <div class="sk-vcard">
          <div class="sk-square"></div>
          <div class="sk-line w80"></div>
          <div class="sk-line w50"></div>
        </div>
      {/each}
    </div>
  {:else if items.length === 0}
    <div class="notice">
      Chưa có đề xuất. Hãy nghe thử vài bài nhạc hoặc bấm <strong>Làm mới</strong> để hệ thống phân tích gu của bạn.
    </div>
  {:else}
    <!-- 1. Spotify Home Top 6 Quick-Pick Tiles -->
    {#if quickPicks.length > 0 && viewLayout !== "soundcloud"}
      <div class="quick-picks-grid">
        {#each quickPicks as item, i (item.url + ":qp")}
          {@const isCurrent = player.current?.url === item.url}
          {@const isPlaying = isCurrent && player.playing}
          <div class="qp-tile" class:cur={isCurrent}>
            <button
              type="button"
              class="qp-main"
              onclick={() => openTrackDetail(item, i)}
              title="Mở & Phát {item.title}"
            >
              <div class="qp-art">
                <img
                  src={getTrackCover(item.thumbnail, item.url, item.title, item.artist)}
                  alt=""
                  loading="lazy"
                  onerror={(e) => handleImgError(e, item.title, item.artist, item.url)}
                />
              </div>
              <div class="qp-text">
                <span class="qp-title">{item.title}</span>
                <span class="qp-artist">{item.artist}</span>
              </div>
            </button>

            <button
              type="button"
              class="qp-play-fab"
              onclick={() => playAndInspect(item, i)}
              title={isPlaying ? "Tạm dừng" : "Phát ngay"}
            >
              {#if isPlaying}
                <Pause size={15} />
              {:else}
                <Play size={15} />
              {/if}
            </button>
          </div>
        {/each}
      </div>
    {/if}

    <!-- 2. Spotify "Made For You" Square Album Card Shelf -->
    {#if viewLayout === "both" || viewLayout === "spotify"}
      <div class="shelf-block">
        <header class="shelf-head">
          <div>
            <span class="kicker">PHONG CÁCH ÂM NHẠC</span>
            <h3>Tuyển tập: {activeCategoryLabel}</h3>
          </div>
          <button type="button" class="play-all-btn" onclick={() => openTrackDetail(items[0], 0)}>
            <Play size={13} />
            <span>Phát tất cả</span>
          </button>
        </header>

        <div class="spotify-shelf-grid">
          {#each (viewLayout === "spotify" ? items : shelfItems) as item, i (item.url + ":sp")}
            {@const pct = downloads[item.url]}
            {@const isDownloaded = downloadedUrls.has(item.url)}
            {@const isSaved = savedUrls.has(item.url) || isDownloaded}
            {@const isCurrent = player.current?.url === item.url}
            {@const isPlaying = isCurrent && player.playing}

            <article class="sp-card" class:cur={isCurrent}>
              <div class="sp-cover-wrap">
                <button
                  type="button"
                  class="sp-cover-btn"
                  onclick={() => openTrackDetail(item, i)}
                  title="Mở chi tiết {item.title}"
                >
                  <img
                    src={getTrackCover(item.thumbnail, item.url, item.title, item.artist)}
                    alt=""
                    loading="lazy"
                    onerror={(e) => handleImgError(e, item.title, item.artist, item.url)}
                  />
                </button>

                <span class="sp-source-tag" class:sc={item.source === "soundcloud"}>
                  {item.source === "soundcloud" ? "SOUNDCLOUD" : "YOUTUBE"}
                </span>

                <span class="sp-dur-tag">{fmtTime(item.duration)}</span>

                <!-- Floating Spotify Play FAB at bottom-right of cover -->
                <button
                  type="button"
                  class="sp-floating-play"
                  class:playing={isPlaying}
                  onclick={() => playAndInspect(item, i)}
                  title={isPlaying ? "Tạm dừng" : "Phát ngay"}
                  aria-label="Phát {item.title}"
                >
                  {#if isPlaying}
                    <Pause size={18} />
                  {:else}
                    <Play size={18} />
                  {/if}
                </button>
              </div>

              <div class="sp-meta">
                <button
                  type="button"
                  class="sp-title"
                  onclick={() => openTrackDetail(item, i)}
                  title={item.title}
                >
                  {item.title}
                </button>
                <div class="sp-artist" title={item.artist}>{item.artist}</div>

                {#if item.reason}
                  <button
                    type="button"
                    class="sp-reason"
                    onclick={() => searchByStyleTag(item.reason!)}
                    title="Tìm các bài nhạc cùng phong cách: {item.reason}"
                  >
                    {item.reason}
                  </button>
                {/if}
              </div>

              <div class="sp-footer">
                <button
                  type="button"
                  class="sp-mini-act"
                  onclick={() => player.enqueueOnline(item)}
                  title="Thêm vào hàng chờ"
                >
                  + Hàng chờ
                </button>

                <div class="sp-icons">
                  <button
                    type="button"
                    class="sp-icon-btn"
                    class:saved={isSaved}
                    onclick={() => onsave(item)}
                    disabled={isSaved}
                    title={isSaved ? "Đã lưu trong thư viện" : "Lưu vào thư viện"}
                  >
                    <Heart size={14} fill={isSaved ? "currentColor" : "none"} />
                  </button>

                  {#if isDownloaded}
                    <span class="mini-badge ok" title="Đã tải về máy"><Check size={12} /></span>
                  {:else if pct !== undefined}
                    <span class="mini-badge">{Math.round(pct)}%</span>
                  {:else}
                    <button
                      type="button"
                      class="sp-icon-btn"
                      onclick={() => ondownload(item)}
                      title="Tải về nghe Offline"
                    >
                      <Download size={14} />
                    </button>
                  {/if}
                </div>
              </div>
            </article>
          {/each}
        </div>
      </div>
    {/if}

    <!-- 3. SoundCloud Interactive Waveform Stream Feed -->
    {#if viewLayout === "both" || viewLayout === "soundcloud"}
      <div class="shelf-block">
        <header class="shelf-head">
          <div>
            <span class="kicker">LUỒNG SÓNG NHẠC</span>
            <h3>Luồng sóng nhạc: {activeCategoryLabel}</h3>
          </div>
        </header>

        <div class="sc-stream-list">
          {#each streamItems as item, i (item.url + ":sc")}
            {@const pct = downloads[item.url]}
            {@const isDownloaded = downloadedUrls.has(item.url)}
            {@const isSaved = savedUrls.has(item.url) || isDownloaded}
            {@const isCurrent = player.current?.url === item.url}
            {@const isPlaying = isCurrent && player.playing}

            <article class="sc-stream-card" class:cur={isCurrent}>
              <button
                type="button"
                class="sc-art"
                onclick={() => openTrackDetail(item, i)}
                title="Mở chi tiết {item.title}"
              >
                <img
                  src={getTrackCover(item.thumbnail, item.url, item.title, item.artist)}
                  alt=""
                  loading="lazy"
                  onerror={(e) => handleImgError(e, item.title, item.artist, item.url)}
                />
                <span class="sc-badge" class:sc={item.source === "soundcloud"}>
                  {item.source === "soundcloud" ? "SC" : "YT"}
                </span>
              </button>

              <div class="sc-body">
                <div class="sc-top-bar">
                  <button
                    type="button"
                    class="sc-play-circle"
                    onclick={() => playAndInspect(item, i)}
                    title={isPlaying ? "Tạm dừng" : "Phát"}
                  >
                    {#if isPlaying}
                      <Pause size={15} />
                    {:else}
                      <Play size={15} />
                    {/if}
                  </button>

                  <div class="sc-titles">
                    <div class="sc-sub-line">
                      <span class="sc-artist">{item.artist}</span>
                      {#if item.reason}
                        <button
                          type="button"
                          class="sc-reason-pill"
                          onclick={() => searchByStyleTag(item.reason!)}
                          title="Tìm các bài nhạc cùng phong cách: {item.reason}"
                        >
                          {item.reason}
                        </button>
                      {/if}
                    </div>
                    <button
                      type="button"
                      class="sc-main-title"
                      onclick={() => openTrackDetail(item, i)}
                    >
                      {item.title}
                    </button>
                  </div>

                  <span class="sc-time">{fmtTime(item.duration)}</span>
                </div>

                <div class="sc-wave-row">
                  <WaveformBar
                    seed="{item.url}:{item.title}:{item.duration}"
                    position={isCurrent ? player.position : 0}
                    duration={isCurrent && player.duration > 0 ? player.duration : item.duration}
                    playing={isPlaying}
                    height={34}
                    onseek={(sec) => {
                      if (isCurrent) {
                        player.seek(sec);
                      } else {
                        openTrackDetail(item, i);
                        setTimeout(() => player.seek(sec), 260);
                      }
                    }}
                  />
                </div>

                <div class="sc-actions-bar">
                  <button
                    type="button"
                    class="sc-act-btn"
                    class:saved={isSaved}
                    onclick={() => onsave(item)}
                    disabled={isSaved}
                  >
                    <Heart size={14} fill={isSaved ? "currentColor" : "none"} /> {isSaved ? "Đã thích" : "Thích & Lưu"}
                  </button>

                  <button
                    type="button"
                    class="sc-act-btn"
                    onclick={() => player.enqueueOnline(item)}
                  >
                    + Thêm vào hàng chờ
                  </button>

                  {#if isDownloaded}
                    <span class="sc-dl-status ok"><Check size={14} /> Đã lưu Offline</span>
                  {:else if pct !== undefined}
                    <span class="sc-dl-status">Đang tải {Math.round(pct)}%</span>
                  {:else}
                    <button
                      type="button"
                      class="sc-act-btn"
                      onclick={() => ondownload(item)}
                    >
                      <Download size={14} /> Tải xuống máy
                    </button>
                  {/if}
                </div>
              </div>
            </article>
          {/each}
        </div>
      </div>
    {/if}

    <!-- Progressive Continuous Feed / Load More -->
    <div class="feed-bottom-bar">
      {#if loadingMore}
        <div class="prog-loading-pill">
          <span class="mini-spin"></span>
          <span>Đang tìm nạp thêm nhiều bài hát cho bạn…</span>
        </div>
      {:else}
        <button type="button" class="prog-more-btn" onclick={loadMoreTracks} title="Tải thêm nhiều bài hát">
          <ChevronDown size={15} />
          <span>Khám phá thêm bài hát ({items.length} bài đã sẵn sàng)</span>
        </button>
      {/if}
    </div>
  {/if}
</section>

<style>
  
  .feed-bottom-bar {
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 24px 0 16px;
    margin-top: 10px;
  }
  .prog-loading-pill {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    padding: 10px 20px;
    border-radius: 999px;
    background: var(--bg-surface-2);
    border: 1px solid var(--border-subtle);
    color: var(--accent);
    font-size: 13px;
    font-weight: 600;
  }
  .prog-more-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    padding: 11px 24px;
    border-radius: 999px;
    border: 1px solid var(--border-strong);
    background: var(--bg-surface-2);
    color: var(--text-primary);
    font-size: 13.5px;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 4px 14px oklch(0 0 0 / 0.2);
    transition: all 0.2s ease;
  }
  .prog-more-btn:hover {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    border-color: var(--accent);
    transform: translateY(-2px);
    box-shadow: 0 6px 20px var(--accent-glow);
  }

  .discovery-hub {
    display: flex;
    flex-direction: column;
    gap: 22px;
    margin-top: 8px;
  }

  /* Top Mood & Layout Bar */
  .hub-Controls-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
    padding-bottom: 10px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .mood-chips {
    display: flex;
    align-items: center;
    gap: 7px;
    flex-wrap: wrap;
  }
  .mood-pill {
    height: 31px;
    padding: 0 13px;
    border-radius: 999px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-1);
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.16s ease;
  }
  .mood-pill:hover {
    background: var(--bg-surface-2);
    color: var(--text-primary);
  }
  .mood-pill.active {
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    border-color: var(--accent);
    font-weight: 700;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  .right-tools {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .layout-switch {
    display: inline-flex;
    background: var(--bg-surface-1);
    border: 1px solid var(--border-subtle);
    border-radius: 999px;
    padding: 3px;
    gap: 2px;
  }
  .layout-switch button {
    height: 26px;
    padding: 0 11px;
    border: none;
    border-radius: 999px;
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }
  .layout-switch button.active {
    background: var(--bg-surface-3);
    color: var(--accent);
  }

  .refresh-pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 13px;
    border-radius: 999px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-1);
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11.5px;
    font-weight: 600;
    cursor: pointer;
  }
  .refresh-pill:hover:not(:disabled) {
    color: var(--text-primary);
    border-color: var(--border-strong);
  }
  .refresh-pill :global(.spin) {
    animation: spin 0.85s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* 1. Spotify Top 6 Quick-Pick Tiles */
  .quick-picks-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(235px, 1fr));
    gap: 10px;
  }
  .qp-tile {
    display: flex;
    align-items: center;
    justify-content: space-between;
    background: linear-gradient(135deg, var(--bg-surface-2) 0%, var(--bg-surface-1) 100%);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius-md, 14px);
    overflow: hidden;
    padding: 5px 10px 5px 6px;
    transition: background 0.18s ease, border-color 0.18s ease;
  }
  .qp-tile:hover {
    background: var(--bg-surface-2);
    border-color: var(--border-strong);
  }
  .qp-tile.cur {
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .qp-main {
    display: flex;
    align-items: center;
    gap: 11px;
    flex: 1;
    min-width: 0;
    border: none;
    background: transparent;
    padding: 0;
    color: inherit;
    font-family: var(--font-sans);
    text-align: left;
    cursor: pointer;
  }
  .qp-art {
    width: 50px;
    height: 50px;
    border-radius: var(--ui-radius-sm, 10px);
    flex: none;
    background: var(--bg-surface-2);
    overflow: hidden;
  }
  .qp-art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .qp-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    padding-right: 6px;
  }
  .qp-title {
    font-size: 12.5px;
    font-weight: 700;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .qp-artist {
    font-size: 11px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .qp-play-fab {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: none;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    display: grid;
    place-items: center;
    cursor: pointer;
    flex: none;
    opacity: 0;
    transform: scale(0.88);
    box-shadow: 0 4px 12px var(--accent-glow);
    transition: all 0.18s var(--ease-out-expo);
  }
  .qp-tile:hover .qp-play-fab,
  .qp-tile.cur .qp-play-fab {
    opacity: 1;
    transform: scale(1);
  }

  /* Shelf Header */
  .shelf-block {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .shelf-head {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: 12px;
  }
  .kicker {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 700;
    letter-spacing: 0.1em;
    color: var(--accent);
  }
  .shelf-head h3 {
    margin: 2px 0 0;
    font-size: 18px;
    font-weight: 800;
    letter-spacing: -0.02em;
  }
  .play-all-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 32px;
    padding: 0 13px;
    border-radius: 999px;
    border: none;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 700;
    cursor: pointer;
    box-shadow: 0 4px 12px var(--accent-glow);
  }

  /* 2. Spotify Square Album Card Shelf */
  .spotify-shelf-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(168px, 1fr));
    gap: 14px;
  }
  .sp-card {
    background: linear-gradient(160deg, var(--bg-surface-2) 0%, var(--bg-surface-1) 100%);
    border: 1px solid var(--border-strong);
    border-radius: var(--ui-radius, 16px);
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 10px;
    transition: background 0.2s ease, transform 0.2s var(--ease-out-expo), border-color 0.2s ease;
  }
  .sp-card:hover {
    background: var(--bg-surface-2);
    border-color: var(--border-strong);
    transform: translateY(-3px);
  }
  .sp-card.cur {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .sp-cover-wrap {
    position: relative;
    width: 100%;
    aspect-ratio: 1 / 1;
    border-radius: calc(var(--ui-radius, 12px) - 2px);
    overflow: hidden;
    background: var(--bg-surface-2);
    box-shadow: 0 8px 22px oklch(0 0 0 / 0.45);
  }
  .sp-cover-btn {
    width: 100%;
    height: 100%;
    padding: 0;
    border: none;
    background: transparent;
    cursor: pointer;
    display: block;
  }
  .sp-cover-btn img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
    transition: transform 0.35s var(--ease-out-expo);
  }
  .sp-card:hover .sp-cover-btn img {
    transform: scale(1.05);
  }

  .sp-source-tag {
    position: absolute;
    top: 7px;
    left: 7px;
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 700;
    letter-spacing: 0.05em;
    padding: 2px 6px;
    border-radius: 4px;
    background: oklch(0.08 0.01 240 / 0.85);
    color: var(--yt-tone);
    backdrop-filter: blur(6px);
    pointer-events: none;
  }
  .sp-source-tag.sc {
    color: var(--accent);
  }
  .sp-dur-tag {
    position: absolute;
    bottom: 7px;
    left: 7px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    font-weight: 600;
    padding: 2px 5px;
    border-radius: 4px;
    background: oklch(0.08 0.01 240 / 0.85);
    color: var(--text-primary);
    pointer-events: none;
  }

  .sp-floating-play {
    position: absolute;
    right: 8px;
    bottom: 8px;
    width: 42px;
    height: 42px;
    border-radius: 50%;
    border: none;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    display: grid;
    place-items: center;
    cursor: pointer;
    opacity: 0;
    transform: translateY(8px);
    box-shadow: 0 8px 20px oklch(0 0 0 / 0.6);
    transition: all 0.22s var(--ease-out-expo);
  }
  .sp-card:hover .sp-floating-play,
  .sp-floating-play.playing {
    opacity: 1;
    transform: translateY(0);
  }
  .sp-floating-play:hover {
    transform: scale(1.07);
    background: var(--accent-hover);
  }

  .sp-meta {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
    flex: 1;
  }
  .sp-title {
    border: none;
    background: transparent;
    padding: 0;
    text-align: left;
    font-family: var(--font-sans);
    font-size: 13px;
    font-weight: 700;
    color: var(--text-primary);
    display: -webkit-box;
    -webkit-line-clamp: 2;
    line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
    line-height: 1.35;
    cursor: pointer;
  }
  .sp-title:hover {
    color: var(--accent);
  }
  .sp-artist {
    font-size: 11.5px;
    color: var(--text-secondary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .sp-reason {
    border: none;
    background: transparent;
    padding: 0;
    text-align: left;
    font-family: var(--font-sans);
    font-size: 10.5px;
    font-weight: 600;
    color: var(--accent);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    margin-top: 2px;
    cursor: pointer;
    transition: opacity 0.15s ease;
  }
  .sp-reason:hover {
    text-decoration: underline;
    opacity: 0.9;
  }

  .sp-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 6px;
    border-top: 1px solid var(--border-subtle);
  }
  .sp-mini-act {
    border: none;
    background: transparent;
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 600;
    padding: 2px 4px;
    cursor: pointer;
  }
  .sp-mini-act:hover {
    color: var(--accent);
  }
  .sp-icons {
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .sp-icon-btn {
    width: 26px;
    height: 26px;
    border-radius: 6px;
    border: none;
    background: transparent;
    color: var(--text-secondary);
    display: grid;
    place-items: center;
    cursor: pointer;
  }
  .sp-icon-btn:hover:not(:disabled) {
    color: var(--text-primary);
    background: var(--bg-surface-3);
  }
  .sp-icon-btn.saved {
    color: var(--accent);
  }
  .mini-badge {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--accent);
  }

  /* 3. SoundCloud Interactive Waveform Stream Feed */
  .sc-stream-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .sc-stream-card {
    display: grid;
    grid-template-columns: 94px 1fr;
    gap: 14px;
    padding: 12px 14px;
    border-radius: var(--ui-radius, 16px);
    background: linear-gradient(135deg, var(--bg-surface-2) 0%, var(--bg-surface-1) 100%);
    border: 1px solid var(--border-strong);
    transition: border-color 0.16s ease, background 0.16s ease;
  }
  .sc-stream-card:hover {
    background: var(--bg-surface-2);
    border-color: var(--border-strong);
  }
  .sc-stream-card.cur {
    border-color: var(--accent);
    background: var(--accent-soft);
  }

  .sc-art {
    position: relative;
    width: 94px;
    height: 94px;
    border-radius: var(--ui-radius-sm, 10px);
    overflow: hidden;
    padding: 0;
    border: 1px solid var(--border-subtle);
    background: var(--bg-surface-2);
    cursor: pointer;
  }
  .sc-art img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .sc-badge {
    position: absolute;
    bottom: 4px;
    right: 4px;
    font-family: var(--font-mono);
    font-size: 8.5px;
    font-weight: 700;
    padding: 1.5px 5px;
    border-radius: 4px;
    background: oklch(0.08 0.01 240 / 0.88);
    color: var(--yt-tone);
  }
  .sc-badge.sc {
    color: var(--accent);
  }

  .sc-body {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    gap: 6px;
    min-width: 0;
  }
  .sc-top-bar {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .sc-play-circle {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    border: none;
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
    display: grid;
    place-items: center;
    cursor: pointer;
    flex: none;
    box-shadow: 0 4px 12px var(--accent-glow);
  }
  .sc-play-circle:hover {
    background: var(--accent-hover);
  }
  .sc-titles {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .sc-sub-line {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 11.5px;
    color: var(--text-secondary);
  }
  .sc-reason-pill {
    border: 1px solid transparent;
    font-family: var(--font-sans);
    font-size: 10px;
    color: var(--accent);
    background: var(--accent-soft);
    padding: 1px 7px;
    border-radius: 999px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }
  .sc-reason-pill:hover {
    border-color: var(--accent);
    background: var(--accent);
    color: var(--accent-fg, oklch(0.09 0.015 240));
  }
  .sc-main-title {
    border: none;
    background: transparent;
    padding: 0;
    text-align: left;
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 700;
    color: var(--text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: pointer;
  }
  .sc-main-title:hover {
    color: var(--accent);
  }
  .sc-time {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-muted);
    flex: none;
  }

  .sc-wave-row {
    padding: 2px 0;
  }

  .sc-actions-bar {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }
  .sc-act-btn {
    height: 26px;
    padding: 0 11px;
    border-radius: 999px;
    border: 1px solid var(--border-subtle);
    background: var(--bg-canvas);
    color: var(--text-secondary);
    font-family: var(--font-sans);
    font-size: 11px;
    font-weight: 600;
    cursor: pointer;
  }
  .sc-act-btn:hover:not(:disabled) {
    color: var(--text-primary);
    border-color: var(--border-strong);
  }
  .sc-act-btn.saved {
    color: var(--accent);
    border-color: var(--accent);
    background: var(--accent-soft);
  }
  .sc-dl-status {
    font-family: var(--font-mono);
    font-size: 10.5px;
    color: var(--accent);
  }
  .sc-dl-status.ok {
    color: var(--signal-online);
  }

  /* Skeletons & Notices */
  .sk-shelf {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(168px, 1fr));
    gap: 14px;
  }
  .sk-vcard {
    padding: 12px;
    border-radius: 12px;
    background: var(--bg-surface-1);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .sk-square {
    width: 100%;
    aspect-ratio: 1/1;
    border-radius: 8px;
    background: var(--bg-surface-2);
    animation: shimmer 1.4s infinite ease-in-out;
  }
  .sk-line {
    height: 10px;
    border-radius: 5px;
    background: var(--bg-surface-2);
    animation: shimmer 1.4s infinite ease-in-out;
  }
  .w80 { width: 80%; }
  .w50 { width: 55%; }
  @keyframes shimmer {
    0%, 100% { opacity: 0.45; }
    50% { opacity: 0.9; }
  }
  .notice {
    padding: 20px;
    border-radius: 12px;
    background: var(--bg-surface-1);
    border: 1px solid var(--border-subtle);
    font-size: 13px;
    color: var(--text-secondary);
  }
  .notice.error {
    color: var(--danger);
  }
</style>
