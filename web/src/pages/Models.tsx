import { useEffect, useState } from "react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardContent,
} from "../components/ui/card";
import { Badge } from "@/components/ui/badge";
import { fetchModels } from "@/api/model";

type Model = {
  id: string;
  name: string;
  userId?: string;
  description?: string;
  vectorFormat?: string;
  createdAt?: string;
  updatedAt?: string;
  [key: string]: any;
};

export default function Models() {
  const [models, setModels] = useState<Model[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedId(text);
      setTimeout(() => setCopiedId((cur) => (cur === text ? null : cur)), 1500);
    } catch (e) {
      console.error("Copy failed", e);
    }
  };

  useEffect(() => {
    const loadModels = async () => {
      setLoading(true);
      setError(null);
      try {
        const res: any = await fetchModels();
        const list: Model[] = Array.isArray(res) ? res : res?.models ?? [];
        setModels(list);
      } catch (err: any) {
        console.error("Error loading models:", err);
        setError(err?.message ?? "Failed to load models");
        setModels([]);
      } finally {
        setLoading(false);
      }
    };

    loadModels();
  }, []);

  return (
    <div className="flex flex-col items-center justify-start min-h-[60vh] mt-8">
      <div className="w-full max-w-4xl p-8 rounded-2xl glass shadow-lg">
        <div className="accent-stripe rounded-t-2xl mb-4" />
        <h1 className="text-3xl font-bold">Models</h1>
        <p className="mt-2 text-gray-300">List of models</p>

        <div className="mt-6">
          {loading && <div>Loading models…</div>}

          {error && <div className="text-red-400">Error: {error}</div>}

          {!loading && !error && models.length === 0 && (
            <div className="text-gray-400">No models available.</div>
          )}

          {!loading && !error && models.length > 0 && (
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              {models.map((m, idx) => (
                <Card key={m.id ?? idx} className="glass">
                  <CardHeader className="px-4">
                    <CardTitle className="text-lg text-white">
                      {m.name ?? "Unnamed Model"}
                    </CardTitle>
                  </CardHeader>

                  <div className="px-4">
                    <div className="flex items-center gap-2">
                      {m.createdAt && (
                        <Badge className="text-[12px]">
                          {new Date(m.createdAt).toLocaleDateString()}
                        </Badge>
                      )}
                      {m.vectorFormat && (
                        <Badge className="text-[12px]">{m.vectorFormat}</Badge>
                      )}
                    </div>


                    <div className="border border-white/6 rounded-md p-2 mt-3 bg-white/2 flex items-center justify-between">
                      <div className="text-xs text-gray-300">{m.id}</div>

                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => copyToClipboard(m.id)}
                          aria-label="Copy ID"
                          className="p-1 rounded hover:bg-white/5"
                          title="Copy ID"
                        >
                          <svg
                            xmlns="http://www.w3.org/2000/svg"
                            width="16"
                            height="16"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            strokeWidth="2"
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            className="text-gray-300"
                          >
                            <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
                            <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
                          </svg>
                        </button>

                        {copiedId === m.id && (
                          <span className="text-xs text-green-400">Copied</span>
                        )}
                      </div>
                    </div>
                  </div>

                  <CardContent className="px-4 pb-4">
                    <details className="text-xs text-gray-200">
                      <summary className="cursor-pointer">More</summary>
                      <pre className="text-[10px] mt-2">{JSON.stringify(m, null, 2)}</pre>
                    </details>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
