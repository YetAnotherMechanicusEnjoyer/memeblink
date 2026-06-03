import React from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { DisplayScreen, OverlayTextSettings } from "../types";

interface ControlPanelProps {
  screens: DisplayScreen[];
  selectedScreen: DisplayScreen;
  onScreenChange: (screen: DisplayScreen) => void;
  assetSource: "path" | "url";
  setAssetSource: (source: "path" | "url") => void;
  assetValue: string;
  setAssetValue: (val: string) => void;
  duration: number;
  setDuration: (val: number) => void;
  textSettings: OverlayTextSettings;
  setTextSettings: React.Dispatch<React.SetStateAction<OverlayTextSettings>>;
  onSubmit: (e: React.FormEvent) => void;
}

export function ControlPanel({
  screens,
  selectedScreen,
  onScreenChange,
  assetSource,
  setAssetSource,
  assetValue,
  setAssetValue,
  duration,
  setDuration,
  textSettings,
  setTextSettings,
  onSubmit,
}: ControlPanelProps) {

  const handleBrowseFiles = async () => {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{
        name: 'Images',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp']
      }]
    });

    if (selected && typeof selected === "string") {
      setAssetValue(selected);
    }
  };

  return (
    <form onSubmit={onSubmit} className="flex flex-col gap-6 bg-slate-900 border border-slate-800 p-6">
      <div className="flex flex-col gap-1.5">
        <label className="text-xs font-semibold uppercase tracking-wider text-slate-400">Target Display</label>
        <select
          className="w-full bg-slate-950 border border-slate-800 px-3 py-2 text-sm text-slate-200 focus:border-cyan-500 transition-colors cursor-pointer"
          value={selectedScreen.id}
          onChange={(e) => {
            const sc = screens.find((s) => s.id === e.target.value);
            if (sc) onScreenChange(sc);
          }}
        >
          {screens.map((s) => (
            <option key={s.id} value={s.id}>
              {s.name}
            </option>
          ))}
        </select>
      </div>

      <div className="flex flex-col gap-2">
        <label className="text-xs font-semibold uppercase tracking-wider text-slate-400">Asset Selection</label>
        <div className="grid grid-cols-2 gap-1 bg-slate-950 p-1 border border-slate-800">
          <button
            type="button"
            className={`py-1.5 text-xs font-medium transition-colors ${assetSource === "path" ? "bg-slate-800 text-cyan-400" : "text-slate-400 hover:text-slate-200"}`}
            onClick={() => {
              setAssetSource("path");
              setAssetValue("");
            }}
          >
            Local File Path
          </button>
          <button
            type="button"
            className={`py-1.5 text-xs font-medium transition-colors ${assetSource === "url" ? "bg-slate-800 text-cyan-400" : "text-slate-400 hover:text-slate-200"}`}
            onClick={() => {
              setAssetSource("url");
              setAssetValue("");
            }}
          >
            Web URL Link
          </button>
        </div>

        {assetSource === "path" ? (
          <div className="flex gap-2">
            <input
              type="text"
              readOnly
              className="flex-1 bg-slate-950 border border-slate-800 px-3 py-2 text-sm text-slate-400 select-all overflow-hidden text-ellipsis whitespace-nowrap"
              placeholder="No file selected"
              value={assetValue}
              required
            />
            <button
              type="button"
              onClick={handleBrowseFiles}
              className="bg-slate-800 hover:bg-slate-700 text-slate-200 px-4 py-2 text-xs font-semibold uppercase tracking-wider border border-slate-700 transition-colors cursor-pointer"
            >
              Browse
            </button>
          </div>
        ) : (
          <input
            type="text"
            className="w-full bg-slate-950 border border-slate-800 px-3 py-2 text-sm text-slate-200 focus:border-cyan-500 transition-colors"
            placeholder="https://example.com/meme.png"
            value={assetValue}
            onChange={(e) => setAssetValue(e.target.value)}
            required
          />
        )}
      </div>

      <div className="flex flex-col gap-1.5">
        <label className="text-xs font-semibold uppercase tracking-wider text-slate-400">Duration (ms)</label>
        <input
          type="number"
          min="100"
          max="60000"
          className="w-full bg-slate-950 border border-slate-800 px-3 py-2 text-sm text-slate-200 focus:border-cyan-500 transition-colors"
          value={duration}
          onChange={(e) => setDuration(Number(e.target.value))}
          required
        />
      </div>

      <div className="border-t border-slate-800 pt-4 flex flex-col gap-4">
        <label className="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            className="w-4 h-4 rounded border-slate-800 bg-slate-950 text-cyan-500 focus:ring-0 focus:ring-offset-0"
            checked={textSettings.enabled}
            onChange={(e) => setTextSettings((prev) => ({ ...prev, enabled: e.target.checked }))}
          />
          <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">Overlay Text Layer</span>
        </label>

        {textSettings.enabled && (
          <div className="flex flex-col gap-3 pl-6 border-l border-slate-800">
            <input
              type="text"
              className="w-full bg-slate-950 border border-slate-800 px-3 py-2 text-sm text-slate-200 focus:border-cyan-500 transition-colors"
              placeholder="Enter custom overlay text..."
              value={textSettings.content}
              onChange={(e) => setTextSettings((prev) => ({ ...prev, content: e.target.value }))}
            />
            <div className="grid grid-cols-3 gap-2">
              <div className="flex flex-col gap-1">
                <span className="text-[10px] uppercase text-slate-500">Position</span>
                <select
                  className="w-full bg-slate-950 border border-slate-800 px-2 py-1 text-xs text-slate-200 cursor-pointer"
                  value={textSettings.position}
                  onChange={(e) => setTextSettings((prev) => ({ ...prev, position: e.target.value as any }))}
                >
                  <option value="top">Top</option>
                  <option value="center">Center</option>
                  <option value="bottom">Bottom</option>
                </select>
              </div>
              <div className="flex flex-col gap-1">
                <span className="text-[10px] uppercase text-slate-500">Color</span>
                <input
                  type="color"
                  className="w-full h-7 bg-slate-950 border border-slate-800 p-0.5 cursor-pointer"
                  value={textSettings.color}
                  onChange={(e) => setTextSettings((prev) => ({ ...prev, color: e.target.value }))}
                />
              </div>
              <div className="flex flex-col gap-1">
                <span className="text-[10px] uppercase text-slate-500">Size (pt)</span>
                <input
                  type="number"
                  min="8"
                  max="120"
                  className="w-full bg-slate-950 border border-slate-800 px-2 py-1 text-xs text-slate-200"
                  value={textSettings.size}
                  onChange={(e) => setTextSettings((prev) => ({ ...prev, size: Number(e.target.value) }))}
                />
              </div>
            </div>
          </div>
        )}
      </div>

      <button
        type="submit"
        className="w-full bg-cyan-600 hover:bg-cyan-500 text-slate-950 font-bold uppercase tracking-wider text-xs py-3 transition-colors active:bg-cyan-700"
      >
        Trigger Overlay
      </button>
    </form>
  );
}
