import React, { useState, useRef, useEffect } from "react";
import { DisplayScreen, OverlayState, OverlayTextSettings } from "../types";

interface ScreenPreviewProps {
  screen: DisplayScreen;
  overlay: OverlayState;
  setOverlay: React.Dispatch<React.SetStateAction<OverlayState>>;
  textSettings: OverlayTextSettings;
}

export function ScreenPreview({ screen, overlay, setOverlay, textSettings }: ScreenPreviewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const [previewWidth, setPreviewWidth] = useState(0);
  const [interaction, setInteraction] = useState<{
    type: "drag" | "resize" | null;
    startX: number;
    startY: number;
    startOverlayX: number;
    startOverlayY: number;
    startWidth: number;
    startHeight: number;
  }>({
    type: null,
    startX: 0,
    startY: 0,
    startOverlayX: 0,
    startOverlayY: 0,
    startWidth: 0,
    startHeight: 0,
  });

  const scale = previewWidth / screen.width;
  const previewHeight = screen.height * scale;

  useEffect(() => {
    if (!containerRef.current) return;
    const updateSize = () => {
      setPreviewWidth(containerRef.current?.getBoundingClientRect().width || 0);
    };
    updateSize();
    window.addEventListener("resize", updateSize);
    return () => window.removeEventListener("resize", updateSize);
  }, []);

  const handlePointerDown = (e: React.PointerEvent, type: "drag" | "resize") => {
    e.stopPropagation();
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    setInteraction({
      type,
      startX: e.clientX,
      startY: e.clientY,
      startOverlayX: overlay.x,
      startOverlayY: overlay.y,
      startWidth: overlay.width,
      startHeight: overlay.height,
    });
  };

  const handlePointerMove = (e: React.PointerEvent) => {
    if (!interaction.type) return;

    const deltaX = (e.clientX - interaction.startX) / scale;
    const deltaY = (e.clientY - interaction.startY) / scale;

    if (interaction.type === "drag") {
      const nextX = Math.max(0, Math.min(screen.width - overlay.width, interaction.startOverlayX + deltaX));
      const nextY = Math.max(0, Math.min(screen.height - overlay.height, interaction.startOverlayY + deltaY));
      setOverlay((prev) => ({ ...prev, x: Math.round(nextX), y: Math.round(nextY) }));
    } else if (interaction.type === "resize") {
      const nextW = Math.max(40, Math.min(screen.width - overlay.x, interaction.startWidth + deltaX));
      const nextH = Math.max(40, Math.min(screen.height - overlay.y, interaction.startHeight + deltaY));
      setOverlay((prev) => ({ ...prev, width: Math.round(nextW), height: Math.round(nextH) }));
    }
  };

  const handlePointerUp = (e: React.PointerEvent) => {
    if (!interaction.type) return;
    (e.target as HTMLElement).releasePointerCapture(e.pointerId);
    setInteraction((prev) => ({ ...prev, type: null }));
  };

  return (
    <div className="flex flex-col gap-2 w-full">
      <span className="text-xs font-semibold uppercase tracking-wider text-slate-400">Interactive Display Preview</span>
      <div
        ref={containerRef}
        className="relative w-full border border-slate-800 bg-slate-950 preview-grid-bg overflow-hidden select-none"
        style={{ height: `${previewHeight}px` }}
      >
        <div
          className="absolute border border-cyan-500 bg-cyan-950/20 shadow-[0_0_10px_rgba(6,182,212,0.15)] cursor-move flex flex-col items-center justify-center transition-shadow duration-200"
          style={{
            left: `${overlay.x * scale}px`,
            top: `${overlay.y * scale}px`,
            width: `${overlay.width * scale}px`,
            height: `${overlay.height * scale}px`,
          }}
          onPointerDown={(e) => handlePointerDown(e, "drag")}
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
        >
          <svg className="w-5 h-5 text-cyan-400/60" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>

          {textSettings.enabled && textSettings.content && (
            <span
              className="absolute font-bold text-center w-full truncate px-1"
              style={{
                color: textSettings.color,
                fontSize: `${Math.max(8, textSettings.size * scale)}px`,
                top: textSettings.position === "above" ? "4px" : "auto",
                bottom: textSettings.position === "below" ? "4px" : "auto",
              }}
            >
              {textSettings.content}
            </span>
          )}

          <div
            className="absolute bottom-0 right-0 w-3 h-3 bg-cyan-500 cursor-se-resize border border-slate-950"
            onPointerDown={(e) => handlePointerDown(e, "resize")}
            onPointerMove={handlePointerMove}
            onPointerUp={handlePointerUp}
          />
        </div>
      </div>
    </div>
  );
}
