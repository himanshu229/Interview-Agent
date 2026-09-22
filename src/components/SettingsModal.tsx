import SettingsTab from "@/tabs/SettingsTab";

interface SettingsModalProps {
  open: boolean;
  onClose: () => void;
}

export default function SettingsModal({ open, onClose }: SettingsModalProps) {
  if (!open) return null;
  return (
    <div className="modal-backdrop" onClick={onClose}>
      <div className="modal settings-modal glass" onClick={(e) => e.stopPropagation()}>
        <header className="modal__head">
          <h2>Settings</h2>
          <button className="toolbar__icon modal__close" onClick={onClose} title="Close">
            ✕
          </button>
        </header>
        <div className="modal__body">
          <SettingsTab />
        </div>
      </div>
    </div>
  );
}
