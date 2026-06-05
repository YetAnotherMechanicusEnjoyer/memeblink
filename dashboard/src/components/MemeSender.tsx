import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface MemeSenderProps {
  currentRoomId: string;
}

export const MemeSender: React.FC<MemeSenderProps> = ({ currentRoomId }) => {
  const [text, setText] = useState('');
  const [imageUrl, setImageUrl] = useState('');
  const [duration, setDuration] = useState(5000);
  const [color, setColor] = useState('#ffffff');
  const [loading, setLoading] = useState(false);
  const [status, setStatus] = useState<{ msg: string; isError: boolean } | null>(null);

  const handleSendMeme = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!currentRoomId) {
      setStatus({ msg: "Please configure and connect to a Matrix room first.", isError: true });
      return;
    }
    if (!text || !imageUrl) {
      setStatus({ msg: "Both text and image URL are required.", isError: true });
      return;
    }

    setLoading(true);
    setStatus(null);

    try {
      await invoke('send_meme_to_matrix', {
        roomIdStr: currentRoomId.trim(),
        text: text.trim(),
        imageUrl: imageUrl.trim(),
        durationMs: duration,
        textColor: color,
      });
      setStatus({ msg: "✓ Meme successfully sent to Matrix!", isError: false });
      setText('');
    } catch (err) {
      console.error(err);
      setStatus({ msg: typeof err === 'string' ? err : "An error occurred while sending.", isError: true });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-6 border border-slate-800 bg-slate-900 shadow-sm">
      <h3 className="text-lg font-bold mb-4 text-white">Broadcast a Meme to Matrix</h3>

      <form onSubmit={handleSendMeme} className="space-y-4">
        <div>
          <label className="block text-xs font-semibold text-gray-400 uppercase mb-1">Meme Text</label>
          <input
            type="text"
            placeholder="e.g., When the code compiles"
            value={text}
            onChange={(e) => setText(e.target.value)}
            className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white text-sm focus:outline-none focus:border-cyan-400 transition-colors"
          />
        </div>

        <div>
          <label className="block text-xs font-semibold text-gray-400 uppercase mb-1">Image URL (Direct link)</label>
          <input
            type="url"
            placeholder="https://.../image.gif"
            value={imageUrl}
            onChange={(e) => setImageUrl(e.target.value)}
            className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white text-sm focus:outline-none focus:border-cyan-400 transition-colors"
          />
        </div>

        <div className="grid grid-cols-2 gap-4">
          <div>
            <label className="block text-xs font-semibold text-gray-400 uppercase mb-1">Duration (ms)</label>
            <input
              type="number"
              value={duration}
              onChange={(e) => setDuration(Number(e.target.value))}
              className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white text-sm focus:outline-none focus:border-cyan-400 transition-colors"
            />
          </div>
          <div>
            <label className="block text-xs font-semibold text-gray-400 uppercase mb-1">Text Color</label>
            <input
              type="color"
              value={color}
              onChange={(e) => setColor(e.target.value)}
              className="w-full h-9 bg-slate-950 border border-slate-800 rounded-lg cursor-pointer"
            />
          </div>
        </div>

        {status && (
          <div className={`p-3 text-xs rounded-lg border ${status.isError ? 'text-red-400 bg-red-950/30 border-red-900/50' : 'text-emerald-400 bg-emerald-950/30 border-emerald-900/50'
            }`}>
            {status.msg}
          </div>
        )}

        <button
          type="submit"
          disabled={loading}
          className={`w-full py-2 bg-cyan-500 hover:bg-cyan-400 text-white font-medium rounded-lg text-sm transition-colors ${loading ? 'opacity-50 cursor-not-allowed' : ''
            }`}
        >
          {loading ? "Sending..." : "Send Meme"}
        </button>
      </form>
    </div>
  );
};
