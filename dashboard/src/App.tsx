import { useState, useEffect } from "react";
import { invoke, convertFileSrc } from "@tauri-apps/api/core";
import { DisplayScreen, OverlayState, OverlayTextSettings } from "./types";
import { ScreenPreview } from "./components/ScreenPreview";
import { ControlPanel } from "./components/ControlPanel";
import "./App.css";

const AVAILABLE_SCREENS: DisplayScreen[] = [
  { id: "screen_1", name: "Display 1 (1920x1080)", width: 1920, height: 1080 },
  { id: "screen_2", name: "Display 2 (2560x1440)", width: 2560, height: 1440 },
  { id: "screen_3", name: "Display 3 (3840x2160)", width: 3840, height: 2160 },
];

export default function App() {
  const [selectedScreen, setSelectedScreen] = useState<DisplayScreen>(AVAILABLE_SCREENS[0]);
  const [assetSource, setAssetSource] = useState<"path" | "url">("path");
  const [assetValue, setAssetValue] = useState("");
  const [duration, setDuration] = useState(3000);
  const [status, setStatus] = useState<{ message: string; isError: boolean } | null>(null);

  const [naturalDimensions, setNaturalDimensions] = useState<{ width: number; height: number }>({
    width: 300,
    height: 300,
  });

  const [overlay, setOverlay] = useState<OverlayState>({
    x: 100,
    y: 100,
    width: 300,
    height: 300,
    widthMode: "custom",
    heightMode: "custom",
  });

  const [textSettings, setTextSettings] = useState<OverlayTextSettings>({
    enabled: false,
    content: "SAMPLE TEXT",
    position: "above",
    color: "#ffffff",
    size: 24,
  });

  useEffect(() => {
    if (!assetValue) return;

    const img = new Image();

    const handleLoad = () => {
      if (img.naturalWidth && img.naturalHeight) {
        setNaturalDimensions({
          width: img.naturalWidth,
          height: img.naturalHeight,
        });
      }
    };

    img.addEventListener("load", handleLoad);

    if (assetSource === "path") {
      img.src = convertFileSrc(assetValue);
    } else {
      img.src = assetValue;
    }

    return () => {
      img.removeEventListener("load", handleLoad);
    };
  }, [assetValue, assetSource]);

  let computedWidth = overlay.width;
  let computedHeight = overlay.height;

  if (overlay.widthMode === "auto" && overlay.heightMode === "auto") {
    computedWidth = naturalDimensions.width;
    computedHeight = naturalDimensions.height;
  } else if (overlay.widthMode === "auto") {
    const ratio = naturalDimensions.width / naturalDimensions.height;
    computedWidth = Math.round(overlay.height * ratio);
  } else if (overlay.heightMode === "auto") {
    const ratio = naturalDimensions.height / naturalDimensions.width;
    computedHeight = Math.round(overlay.width * ratio);
  }

  const handleScreenChange = (screen: DisplayScreen) => {
    setSelectedScreen(screen);
    setOverlay((prev) => ({
      ...prev,
      x: Math.min(prev.x, screen.width - computedWidth),
      y: Math.min(prev.y, screen.height - computedHeight),
    }));
  };

  const handleSend = async (e: React.FormEvent) => {
    e.preventDefault();
    setStatus({ message: "Transmitting...", isError: false });

    const payload = {
      image_path: assetValue,
      duration_ms: duration,
      anchor: "TopLeft",
      width: overlay.widthMode === "auto" ? "auto" : computedWidth,
      height: overlay.heightMode === "auto" ? "auto" : computedHeight,
      x: overlay.x,
      y: overlay.y,
      text: textSettings.enabled ? textSettings.content : null,
      text_position: textSettings.enabled ? textSettings.position : null,
      text_color: textSettings.enabled ? textSettings.color : null,
      text_size: textSettings.enabled ? textSettings.size : null,
    };

    try {
      await invoke("inject_meme", { event: payload });
      setStatus({ message: "Overlay successfully triggered", isError: false });
      setTimeout(() => setStatus(null), 3000);
    } catch (error) {
      setStatus({ message: `Error: ${error}`, isError: true });
    }
  };

  return (
    <div className="min-h-screen flex flex-col p-8">
      <header className="mb-8 flex items-center justify-between border-b border-slate-800 pb-4">
        <div>
          <h1 className="text-xl font-bold tracking-wider text-slate-100 uppercase">MemeBlink</h1>
          <p className="text-xs text-slate-500 tracking-wide">Dashboard</p>
        </div>
        {status && (
          <div className={`px-3 py-1.5 text-xs font-mono border ${status.isError ? "bg-red-950/40 text-red-400 border-red-900" : "bg-cyan-950/40 text-cyan-400 border-cyan-900"}`}>
            {status.message}
          </div>
        )}
      </header>

      <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
        <div className="lg:col-span-5">
          <ControlPanel
            screens={AVAILABLE_SCREENS}
            selectedScreen={selectedScreen}
            onScreenChange={handleScreenChange}
            assetSource={assetSource}
            setAssetSource={setAssetSource}
            assetValue={assetValue}
            setAssetValue={setAssetValue}
            duration={duration}
            setDuration={setDuration}
            overlay={overlay}
            setOverlay={setOverlay}
            textSettings={textSettings}
            setTextSettings={setTextSettings}
            onSubmit={handleSend}
          />
        </div>

        <div className="lg:col-span-7 bg-slate-900 border border-slate-800 p-6">
          <ScreenPreview
            screen={selectedScreen}
            overlay={overlay}
            setOverlay={setOverlay}
            textSettings={textSettings}
            computedWidth={computedWidth}
            computedHeight={computedHeight}
          />
        </div>
      </div>
    </div>
  );
}
