import { defineConfig } from "vitepress";

export default defineConfig({
  title: "netinject",
  description:
    "Lightweight API security testing orchestrator",
  themeConfig: {
    logo: "/logo.svg",
    nav: [
      { text: "Guide", link: "/guide/getting-started" },
      { text: "CLI Reference", link: "/cli/run" },
      { text: "Config", link: "/config/" },
    ],
    sidebar: [
      {
        text: "Guide",
        items: [
          { text: "Getting Started", link: "/guide/getting-started" },
          { text: "Installation", link: "/guide/installation" },
          { text: "Quick Start", link: "/guide/quick-start" },
        ],
      },
      {
        text: "CLI Reference",
        items: [
          { text: "run", link: "/cli/run" },
          { text: "recon", link: "/cli/recon" },
          { text: "scan", link: "/cli/scan" },
          { text: "fuzz", link: "/cli/fuzz" },
          { text: "baseline", link: "/cli/baseline" },
          { text: "regress", link: "/cli/regress" },
          { text: "sessions", link: "/cli/sessions" },
          { text: "replay", link: "/cli/replay" },
          { text: "init", link: "/cli/init" },
          { text: "check", link: "/cli/check" },
        ],
      },
      {
        text: "Configuration",
        items: [
          { text: "Overview", link: "/config/" },
          { text: "Auth Profiles", link: "/config/auth" },
          { text: "Scope Rules", link: "/config/scope" },
          { text: "Adapter Config", link: "/config/adapters" },
          { text: "Pipelines", link: "/config/pipelines" },
          { text: "Regression", link: "/config/regression" },
        ],
      },
      {
        text: "Architecture",
        items: [
          { text: "Overview", link: "/architecture/" },
          { text: "Adapters", link: "/architecture/adapters" },
          { text: "Findings", link: "/architecture/findings" },
          { text: "Session Store", link: "/architecture/sessions" },
          { text: "Reports", link: "/architecture/reports" },
        ],
      },
    ],
    socialLinks: [
      { icon: "github", link: "https://github.com/QaidVoid/netinject" },
    ],
    search: {
      provider: "local",
    },
  },
});
