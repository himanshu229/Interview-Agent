import type { TabKey } from "@/types";

interface SidebarProps {
  activeTab: TabKey;
  onChange: (tab: TabKey) => void;
}

const TABS: { key: TabKey; label: string; icon: string }[] = [
  { key: "chat", label: "Chat", icon: "💬" },
  { key: "transcript", label: "Transcript", icon: "🎙" },
  { key: "screenshot", label: "Screenshot", icon: "🖼" },
  { key: "history", label: "History", icon: "🕑" },
  { key: "settings", label: "Settings", icon: "⚙" },
];

export default function Sidebar({ activeTab, onChange }: SidebarProps) {
  return (
    <nav className="sidebar">
      {TABS.map((tab) => (
        <button
          key={tab.key}
          className={
            "sidebar__item" +
            (activeTab === tab.key ? " sidebar__item--active" : "")
          }
          onClick={() => onChange(tab.key)}
          title={tab.label}
        >
          <span className="sidebar__icon" aria-hidden>
            {tab.icon}
          </span>
          <span className="sidebar__label">{tab.label}</span>
        </button>
      ))}
    </nav>
  );
}
