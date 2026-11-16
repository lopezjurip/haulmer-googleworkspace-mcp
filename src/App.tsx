import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ask, open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  ERROR_TEXT,
  HAS_LEVEL,
  TOGGLE_LEVEL,
  type InstalledSkills,
  type KeyStorage,
  type SaveOutcome,
  type Selection,
  type ServiceInfo,
  type Status,
} from "./core";

/** Google's "Third-party apps & services" page, where the person can also revoke access. */
const GOOGLE_CONNECTIONS_URL = "https://myaccount.google.com/connections";

/** The one action running right now; controls stay disabled while one runs. */
type Busy = "save" | "revoke" | "install" | "advanced" | null;

/** Feedback shown after an action. */
type Notice = { kind: "success" | "info" | "error"; text: string } | null;

/**
 * The one screen of haulmer-connect: account, services, Save, Revoke all,
 * Install in my agents and the Advanced settings.
 *
 * @example
 * <App />
 */
export default function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);
  const [catalogue, setCatalogue] = useState<ServiceInfo[]>([]);
  const [catalogueError, setCatalogueError] = useState<string | null>(null);
  const [selection, setSelection] = useState<Selection>([]);
  const [busy, setBusy] = useState<Busy>(null);
  const [authUrl, setAuthUrl] = useState<string | null>(null);
  const [notice, setNotice] = useState<Notice>(null);

  useEffect(() => {
    const unlisten = listen<string>("auth-url", (event) => setAuthUrl(event.payload));
    loadAll();
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  /**
   * Loads the status, the service labels and the starting selection
   * (what Google granted when connected, the safe default otherwise).
   *
   * @example
   * await loadAll();
   */
  async function loadAll() {
    const [statusResult, catalogueResult, defaultsResult] = await Promise.allSettled([
      invoke<Status>("status"),
      invoke<ServiceInfo[]>("catalogue"),
      invoke<Selection>("default_selection"),
    ]);
    if (catalogueResult.status === "fulfilled") {
      setCatalogue(catalogueResult.value);
      setCatalogueError(null);
    } else {
      setCatalogueError(ERROR_TEXT(catalogueResult.reason));
    }
    if (statusResult.status === "fulfilled") {
      setStatus(statusResult.value);
      setStatusError(null);
    } else {
      setStatusError(ERROR_TEXT(statusResult.reason));
    }
    if (statusResult.status === "fulfilled" && statusResult.value.account !== null) {
      setSelection(statusResult.value.granted);
    } else if (defaultsResult.status === "fulfilled") {
      setSelection(defaultsResult.value);
    }
  }

  /**
   * Reloads only the status (after Revoke all or an Advanced change).
   *
   * @example
   * await refreshStatus();
   */
  async function refreshStatus() {
    try {
      setStatus(await invoke<Status>("status"));
      setStatusError(null);
    } catch (error) {
      setStatusError(ERROR_TEXT(error));
    }
  }

  /**
   * Saves the selection: runs the Google sign-in, asks before replacing foreign
   * gws credentials, then shows the result.
   *
   * @example
   * <button onClick={handleSave}>Save</button>
   */
  async function handleSave() {
    setBusy("save");
    setNotice(null);
    setAuthUrl(null);
    try {
      let outcome = await invoke<SaveOutcome>("save", { selection, confirmReplace: false });
      if (outcome.kind === "needs_replace_confirmation") {
        const replace = await ask("You already have gws connected with another app. Replace?", {
          title: "Replace gws connection",
          kind: "warning",
          okLabel: "Yes",
          cancelLabel: "No",
        });
        if (!replace) {
          return;
        }
        outcome = await invoke<SaveOutcome>("save", { selection, confirmReplace: true });
      }
      if (outcome.kind === "connected") {
        const account = outcome.status.account ?? "your account";
        setStatus(outcome.status);
        setStatusError(null);
        setSelection(outcome.status.granted);
        setNotice({ kind: "success", text: `Saved. Connected as ${account}.` });
      }
    } catch (error) {
      const text = ERROR_TEXT(error);
      if (text === "Google sign-in was cancelled") {
        setNotice({ kind: "info", text: "Google sign-in was cancelled. Nothing changed." });
      } else {
        setNotice({ kind: "error", text });
      }
    } finally {
      setBusy(null);
      setAuthUrl(null);
    }
  }

  /**
   * Stops the Google sign-in that Save is waiting on.
   *
   * @example
   * <button onClick={handleCancel}>Cancel</button>
   */
  async function handleCancel() {
    try {
      await invoke("cancel_login");
    } catch (error) {
      setNotice({ kind: "error", text: ERROR_TEXT(error) });
    }
  }

  /**
   * Revokes every access at Google and clears local credentials, after a confirm.
   *
   * @example
   * <button onClick={handleRevokeAll}>Revoke all</button>
   */
  async function handleRevokeAll() {
    const confirmed = await ask(
      "Revoke all access? Your agents lose access to Google until you connect again.",
      { title: "Revoke all", kind: "warning", okLabel: "Revoke all", cancelLabel: "Keep access" },
    );
    if (!confirmed) {
      return;
    }
    setBusy("revoke");
    setNotice(null);
    try {
      await invoke("revoke_all");
      setNotice({ kind: "success", text: "All access was revoked." });
    } catch (error) {
      setNotice({ kind: "error", text: ERROR_TEXT(error) });
    } finally {
      await refreshStatus();
      setBusy(null);
    }
  }

  /**
   * Installs the gws skills for the enabled services in every agent found.
   *
   * @example
   * <button onClick={handleInstall}>Install in my agents</button>
   */
  async function handleInstall() {
    setBusy("install");
    setNotice(null);
    try {
      const installed = await invoke<InstalledSkills[]>("install_skills");
      if (installed.length === 0) {
        setNotice({ kind: "info", text: "No agents were found on this computer." });
      } else {
        const targets = installed.map((item) => item.target).join(", ");
        setNotice({ kind: "success", text: `Skills installed in: ${targets}.` });
      }
    } catch (error) {
      setNotice({ kind: "error", text: ERROR_TEXT(error) });
    } finally {
      setBusy(null);
    }
  }

  /**
   * Sets the gws config folder; `null` resets to the gws default folder.
   *
   * @example
   * await changeConfigDir("/Users/ana/.config/haulmer-gws");
   * await changeConfigDir(null);
   */
  async function changeConfigDir(dir: string | null) {
    setBusy("advanced");
    setNotice(null);
    try {
      await invoke("set_config_dir", { dir });
      setNotice({ kind: "success", text: "Config folder saved." });
    } catch (error) {
      setNotice({ kind: "error", text: ERROR_TEXT(error) });
    } finally {
      await refreshStatus();
      setBusy(null);
    }
  }

  /**
   * Opens a folder picker and saves the chosen config folder.
   *
   * @example
   * <button onClick={handleChooseFolder}>Choose folder…</button>
   */
  async function handleChooseFolder() {
    const chosen = await open({ directory: true, defaultPath: status?.config_dir });
    if (typeof chosen === "string") {
      await changeConfigDir(chosen);
    }
  }

  /**
   * Saves where gws keeps its encryption key.
   *
   * @example
   * <select onChange={(event) => handleKeyStorage(event.target.value as KeyStorage)} />
   */
  async function handleKeyStorage(storage: KeyStorage) {
    setBusy("advanced");
    setNotice(null);
    try {
      await invoke("set_key_storage", { storage });
      setNotice({ kind: "success", text: "Key storage saved." });
    } catch (error) {
      setNotice({ kind: "error", text: ERROR_TEXT(error) });
    } finally {
      await refreshStatus();
      setBusy(null);
    }
  }

  let accountText: string;
  if (status === null && statusError === null) {
    accountText = "Checking…";
  } else if (status === null) {
    accountText = "Status unavailable";
  } else if (status.account !== null) {
    accountText = status.account;
  } else {
    accountText = "Not connected";
  }

  let noticeClass = "border-red-200 bg-red-50 text-red-800";
  if (notice?.kind === "success") {
    noticeClass = "border-emerald-200 bg-emerald-50 text-emerald-800";
  } else if (notice?.kind === "info") {
    noticeClass = "border-slate-200 bg-slate-50 text-slate-700";
  }

  let keyStorageHint = "";
  if (status?.key_storage === "auto" && status.resolved_key_storage === "file") {
    keyStorageHint = "Auto uses a file in the config folder.";
  } else if (status?.key_storage === "auto") {
    keyStorageHint = "Auto uses the system keyring.";
  }

  const disabled = busy !== null;

  return (
    <main className="mx-auto max-w-xl space-y-6 p-8 font-sans text-slate-900">
      <header>
        <h1 className="text-xl font-semibold">haulmer-connect</h1>
        <p className="mt-1 text-sm text-slate-500">Connect your AI agents to Google Workspace.</p>
      </header>

      <section className="rounded-lg border border-slate-200 p-4">
        <p className="text-xs uppercase tracking-wide text-slate-500">Google account</p>
        <p className="mt-1 font-medium">{accountText}</p>
        {statusError !== null && <p className="mt-1 text-sm text-slate-500">{statusError}</p>}
        {status !== null && status.account !== null && !status.token_valid && (
          <p className="mt-1 text-sm text-amber-700">The sign-in expired. Click Save to sign in again.</p>
        )}
        {status !== null && !status.gws_available && (
          <p className="mt-1 text-sm text-slate-500">gws will be downloaded when you click Save.</p>
        )}
      </section>

      <section className="space-y-3">
        <h2 className="text-sm font-semibold text-slate-700">What your agents can use</h2>
        {catalogueError !== null && (
          <p className="text-sm text-slate-500">Services unavailable: {catalogueError}</p>
        )}
        <ul className="divide-y divide-slate-100 rounded-lg border border-slate-200">
          {catalogue.map((info) => (
            <li key={info.service} className="p-4">
              <p className="font-medium">{info.name}</p>
              <label className="mt-2 flex items-start gap-2 text-sm text-slate-700">
                <input
                  type="checkbox"
                  className="mt-0.5"
                  disabled={disabled}
                  checked={HAS_LEVEL(selection, info.service, "read")}
                  onChange={(event) =>
                    setSelection(TOGGLE_LEVEL(selection, info.service, "read", event.target.checked))
                  }
                />
                {info.read}
              </label>
              <label className="mt-1 flex items-start gap-2 text-sm text-slate-700">
                <input
                  type="checkbox"
                  className="mt-0.5"
                  disabled={disabled}
                  checked={HAS_LEVEL(selection, info.service, "write")}
                  onChange={(event) =>
                    setSelection(TOGGLE_LEVEL(selection, info.service, "write", event.target.checked))
                  }
                />
                {info.write}
              </label>
            </li>
          ))}
        </ul>

        {busy === "save" && (
          <div className="flex items-center justify-between gap-3 rounded-lg bg-slate-50 p-3 text-sm">
            {authUrl === null && <span className="text-slate-600">Getting ready…</span>}
            {authUrl !== null && (
              <span className="text-slate-600">
                Waiting for Google sign-in…{" "}
                <button type="button" className="underline" onClick={() => openUrl(authUrl)}>
                  Open it again
                </button>
              </span>
            )}
            <button
              type="button"
              className="rounded-md border border-slate-300 px-3 py-1 hover:bg-white"
              onClick={handleCancel}
            >
              Cancel
            </button>
          </div>
        )}
        {busy !== "save" && (
          <div>
            <button
              type="button"
              className="rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-40"
              disabled={disabled || selection.length === 0}
              onClick={handleSave}
            >
              Save
            </button>
            <p className="mt-2 text-xs text-slate-400">Removing a service needs a new Google sign-in.</p>
          </div>
        )}
      </section>

      {notice !== null && (
        <p className={`rounded-lg border p-3 text-sm ${noticeClass}`}>{notice.text}</p>
      )}

      <section className="flex flex-wrap items-center gap-3 border-t border-slate-200 pt-6 text-sm">
        <button
          type="button"
          className="rounded-md border border-slate-300 px-3 py-1.5 hover:bg-slate-50 disabled:opacity-40"
          disabled={disabled}
          onClick={handleInstall}
        >
          Install in my agents
        </button>
        <button
          type="button"
          className="rounded-md border border-red-200 px-3 py-1.5 text-red-700 hover:bg-red-50 disabled:opacity-40"
          disabled={disabled}
          onClick={handleRevokeAll}
        >
          Revoke all
        </button>
        <a
          href={GOOGLE_CONNECTIONS_URL}
          className="text-slate-500 underline hover:text-slate-700"
          onClick={(event) => {
            event.preventDefault();
            openUrl(GOOGLE_CONNECTIONS_URL);
          }}
        >
          Check access on Google
        </a>
      </section>

      <details className="rounded-lg border border-slate-200 p-4 text-sm">
        <summary className="cursor-pointer font-medium text-slate-700">Advanced</summary>
        <div className="mt-4 space-y-4">
          <div>
            <p className="text-slate-500">Config folder</p>
            <p className="mt-1 break-all font-mono text-xs">
              {status?.config_dir ?? "Unknown"}
              {status?.config_dir_is_default && (
                <span className="font-sans text-slate-400"> (default)</span>
              )}
            </p>
            <div className="mt-2 flex gap-2">
              <button
                type="button"
                className="rounded-md border border-slate-300 px-3 py-1 hover:bg-slate-50 disabled:opacity-40"
                disabled={disabled}
                onClick={handleChooseFolder}
              >
                Choose folder…
              </button>
              <button
                type="button"
                className="rounded-md border border-slate-300 px-3 py-1 hover:bg-slate-50 disabled:opacity-40"
                disabled={disabled || status === null || status.config_dir_is_default}
                onClick={() => changeConfigDir(null)}
              >
                Reset to default
              </button>
            </div>
          </div>
          <label className="block">
            <span className="text-slate-500">Key storage</span>
            <select
              className="mt-1 block rounded-md border border-slate-300 px-2 py-1"
              disabled={disabled}
              value={status?.key_storage ?? "auto"}
              onChange={(event) => handleKeyStorage(event.target.value as KeyStorage)}
            >
              <option value="auto">Auto</option>
              <option value="keyring">Keyring</option>
              <option value="file">File</option>
            </select>
            {keyStorageHint !== "" && (
              <span className="mt-1 block text-xs text-slate-400">{keyStorageHint}</span>
            )}
          </label>
        </div>
      </details>
    </main>
  );
}
