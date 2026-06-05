import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface MatrixPanelProps {
  roomId: string;
  setRoomId: (value: string) => void;
  setSuccessMsg: (value: string | null) => void;
}

export const MatrixPanel: React.FC<MatrixPanelProps> = ({ roomId, setRoomId, setSuccessMsg }) => {
  const [homeserver, setHomeserver] = useState('https://matrix.org');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSSOConnect = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!homeserver || !roomId) {
      setError("Please fill in all fields.");
      return;
    }

    setLoading(true);
    setError(null);

    try {
      const msg = await invoke<string>('start_matrix_sso_auth', {
        homeserverUrl: homeserver.trim(),
        roomIdStr: roomId.trim(),
      });
      setSuccessMsg(msg);
    } catch (err) {
      console.error("SSO Init Error:", err);
      setError(typeof err === 'string' ? err : "Failed to launch authentication.");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="p-6 border border-slate-800 bg-slate-900 shadow-sm">
      <h2 className="text-xl font-bold mb-2 text-white">Matrix Connection (SSO)</h2>
      <p className="text-sm text-gray-400 mb-6">
        Authenticate securely via your homeserver's authentication protocol.
      </p>

      <form onSubmit={handleSSOConnect} className="space-y-4">
        <div>
          <label className="block text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">
            Matrix Server (Homeserver)
          </label>
          <input
            type="url"
            placeholder="https://matrix.org"
            value={homeserver}
            onChange={(e) => setHomeserver(e.target.value)}
            disabled={loading}
            className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white text-sm focus:outline-none focus:border-emerald-500 transition-colors"
          />
        </div>

        <div>
          <label className="block text-xs font-semibold text-gray-400 uppercase tracking-wider mb-1">
            Room ID
          </label>
          <input
            type="text"
            placeholder="!abcde:matrix.org"
            value={roomId}
            onChange={(e) => setRoomId(e.target.value)}
            disabled={loading}
            className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white text-sm focus:outline-none focus:border-emerald-500 transition-colors"
          />
        </div>

        {error && (
          <div className="p-3 text-xs text-red-400 bg-red-950/30 border border-red-900/50 rounded-lg">
            {error}
          </div>
        )}

        <button
          type="submit"
          disabled={loading}
          className={`w-full py-2.5 px-4 text-white font-medium rounded-lg transition-all flex items-center justify-center gap-2 text-sm ${loading
            ? 'bg-slate-800 cursor-not-allowed opacity-50'
            : 'bg-emerald-600 hover:bg-emerald-500'
            }`}
        >
          {loading ? "Waiting for browser..." : "Connect with Matrix"}
        </button>
      </form>
    </div>
  );
};
