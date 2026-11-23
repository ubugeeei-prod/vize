export function installMisskeyDevResponses(): void {
  const originalFetch = window.fetch;
  window.fetch = function (input, init) {
    const url =
      typeof input === "string" ? input : input instanceof URL ? input.toString() : input.url;
    if (url.includes("/api/")) {
      let body = "{}";
      if (url.includes("/api/meta")) {
        body = JSON.stringify({
          name: "Misskey",
          uri: "http://localhost:3000",
          version: "2024.11.0",
          description: "A Misskey instance",
          disableRegistration: false,
          federation: "all",
          iconUrl: null,
          backgroundImageUrl: null,
          defaultDarkTheme: null,
          defaultLightTheme: null,
          clientOptions: {},
          policies: { ltlAvailable: true, gtlAvailable: true },
          maxNoteTextLength: 3000,
          features: {
            registration: true,
            localTimeline: true,
            globalTimeline: true,
            miauth: true,
          },
        });
      } else if (url.includes("/api/emojis")) {
        body = JSON.stringify({ emojis: [] });
      }
      return Promise.resolve(
        new Response(body, {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      );
    }
    if (url.includes("/assets/locales/")) {
      return Promise.resolve(
        new Response(
          JSON.stringify({
            _lang_: "English",
            headlineMisskey: "A network connected by notes",
            introMisskey:
              "Welcome! Misskey is an open source, decentralized microblogging platform.",
            monthAndDay: "{month}/{day}",
            search: "Search",
            notifications: "Notifications",
            username: "Username",
            password: "Password",
            forgotPassword: "Forgot password",
            fetchingAsAp498: "Fetching...",
            login: "Sign In",
            loggingIn: "Signing In",
            signup: "Sign Up",
            uploading: "Uploading...",
            save: "Save",
            users: "Users",
            notes: "Notes",
            following: "Following",
            followers: "Followers",
            ok: "OK",
            gotIt: "Got it!",
            cancel: "Cancel",
            enterUsername: "Enter username",
            renotedBy: "Boosted by {user}",
            noNotes: "No notes",
            noNotifications: "No notifications",
            instance: "Instance",
            settings: "Settings",
            basicSettings: "General",
            otherSettings: "Other Settings",
            openInWindow: "Open in window",
            profile: "Profile",
            timeline: "Timeline",
            noAccountDescription: "No description",
            loginFailed: "Sign in failed",
            showMore: "Show More",
            youGotNewFollower: "followed you",
            explore: "Explore",
            favorited: "Favorited",
            unfavorite: "Unfavorite",
            pinnedNote: "Pinned note",
            somethingHappened: "Something went wrong",
            retry: "Retry",
            pageLoadError: "An error occurred while loading the page.",
            pageLoadErrorDescription:
              "This is usually caused by a network error or the browser's cache.",
            serverIsDead: "Server is not responding. Please wait a moment and try again.",
            youShouldUpgradeClient: "Please refresh the page to use the updated client.",
            enterListName: "Enter list name",
            privacy: "Privacy",
            makeFollowManuallyApprove: "Follow requests require approval",
            defaultNavigationBehaviour: "Default navigation behavior",
            editProfile: "Edit profile",
            noteOfThisUser: "Notes by this user",
            joinThisServer: "Sign up at this instance",
            exploreOtherServers: "Look for another instance",
            letsLookAtTimeline: "Have a look at the timeline",
            invitationRequiredToRegister: "This instance is invite-only.",
          }),
          {
            status: 200,
            headers: { "Content-Type": "application/json" },
          },
        ),
      );
    }
    return originalFetch.call(window, input, init);
  } as typeof window.fetch;
}

export function installMisskeyVisualMocks(apiFixtures: Record<string, unknown>): void {
  const originalFetch = window.fetch.bind(window);

  window.fetch = function (input, init) {
    const url =
      typeof input === "string"
        ? new URL(input, window.location.href)
        : input instanceof URL
          ? input
          : new URL(input.url, window.location.href);

    if (!url.pathname.startsWith("/api/")) {
      return originalFetch(input, init);
    }

    const endpoint = url.pathname.slice("/api/".length);
    let body: unknown = {};

    if (Object.prototype.hasOwnProperty.call(apiFixtures, endpoint)) {
      body = apiFixtures[endpoint];
    } else if (endpoint.startsWith("notes/")) {
      body = [];
    } else if (endpoint.startsWith("charts/")) {
      body = {
        read: [],
        write: [],
      };
    } else if (
      endpoint.endsWith("/list") ||
      endpoint.endsWith("/search") ||
      endpoint.includes("timeline") ||
      endpoint.includes("notifications") ||
      endpoint.includes("requests") ||
      endpoint.includes("following") ||
      endpoint.includes("followers") ||
      endpoint.includes("files") ||
      endpoint.includes("folders") ||
      endpoint.includes("featured") ||
      endpoint.includes("owned") ||
      endpoint.includes("joined") ||
      endpoint.includes("liked")
    ) {
      body = [];
    }

    return Promise.resolve(
      new Response(JSON.stringify(body), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      }),
    );
  } as typeof window.fetch;

  const serviceWorkerRegistration = {
    active: {
      postMessage() {},
    },
    unregister: async () => true,
    update: async () => {},
    scope: "/",
  };

  if ("serviceWorker" in navigator) {
    try {
      Object.assign(navigator.serviceWorker, {
        register: async () => serviceWorkerRegistration,
        getRegistrations: async () => [serviceWorkerRegistration],
      });
      Object.defineProperty(navigator.serviceWorker, "controller", {
        configurable: true,
        value: serviceWorkerRegistration.active,
      });
      Object.defineProperty(navigator.serviceWorker, "ready", {
        configurable: true,
        value: Promise.resolve(serviceWorkerRegistration),
      });
    } catch {
      // Network-level sw.js routing still keeps the VRT environment deterministic.
    }
  }
}
