import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ask, save } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { useEffect, useRef, useState } from "react";
import adminIcon from "./assets/service-icons/admin.png";
import businessIntelligenceIcon from "./assets/service-icons/business-intelligence.png";
import calendarIcon from "./assets/service-icons/calendar.png";
import chatIcon from "./assets/service-icons/chat.png";
import dataAnalyticsIcon from "./assets/service-icons/data-analytics.png";
import docsIcon from "./assets/service-icons/docs.png";
import driveIcon from "./assets/service-icons/drive.png";
import formsIcon from "./assets/service-icons/forms.png";
import gmailIcon from "./assets/service-icons/gmail.png";
import mapsIcon from "./assets/service-icons/maps.png";
import marketplaceIcon from "./assets/service-icons/marketplace.png";
import mediaIcon from "./assets/service-icons/media.png";
import meetIcon from "./assets/service-icons/meet.png";
import peopleIcon from "./assets/service-icons/people.png";
import securityIdentityIcon from "./assets/service-icons/security-identity.png";
import sheetsIcon from "./assets/service-icons/sheets.png";
import slidesIcon from "./assets/service-icons/slides.png";
import tasksIcon from "./assets/service-icons/tasks.png";
import webMobileIcon from "./assets/service-icons/web-mobile.png";
import {
  type BusinessConnector,
  type BusinessConnectorInfo,
  type BusinessSelection,
  ERROR_TEXT,
  HAS_BUSINESS_LEVEL,
  HAS_LEVEL,
  type InstallPrompt,
  type RevokeOutcome,
  type SaveOutcome,
  type Selection,
  type Service,
  type ServiceInfo,
  type Status,
} from "./core";

/** The one action running right now; controls stay disabled while one runs. */
type Busy = "save" | "revoke" | "update" | null;

/** Feedback shown after an action. */
type Notice = { kind: "success" | "info" | "error"; text: string } | null;
type AccessChoice = "off" | "read" | "write";

const SERVICE_ICONS: Partial<Record<Service, string>> = {
  gmail: gmailIcon,
  calendar: calendarIcon,
  chat: chatIcon,
  meet: meetIcon,
  drive: driveIcon,
  docs: docsIcon,
  sheets: sheetsIcon,
  slides: slidesIcon,
  forms: formsIcon,
  tasks: tasksIcon,
  people: peopleIcon,
  "admin-reports": adminIcon,
  "admin-directory": adminIcon,
  "admin-groups": adminIcon,
  "cloud-identity": securityIdentityIcon,
};

const BUSINESS_ICONS: Record<BusinessConnector, string> = {
  youtube: mediaIcon,
  "google-ads": businessIntelligenceIcon,
  analytics: dataAnalyticsIcon,
  "search-console": webMobileIcon,
  "business-profile": mapsIcon,
  "merchant-center": marketplaceIcon,
};

function ServiceIcon({ src, label }: { src?: string; label: string }) {
  if (src !== undefined) {
    return (
      <img
        src={src}
        alt=""
        aria-hidden="true"
        className="h-7 w-7 shrink-0 object-contain"
      />
    );
  }
  return (
    <span
      aria-hidden="true"
      className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-slate-100 text-sm font-semibold text-slate-500 dark:bg-zinc-800 dark:text-zinc-300"
    >
      {label.charAt(0)}
    </span>
  );
}

function AvailabilityWarning({ message }: { message: string }) {
  return (
    <button
      type="button"
      title={message}
      aria-label={`Advertencia: ${message}`}
      className="ml-1 inline-flex h-6 w-6 items-center justify-center rounded-full text-base leading-none text-amber-600 hover:bg-amber-100 focus:outline-none focus:ring-2 focus:ring-amber-500 dark:text-amber-300 dark:hover:bg-amber-950"
      onClick={() => window.alert(message)}
    >
      ⚠
    </button>
  );
}

function serviceAccess(selection: Selection, service: Service): AccessChoice {
  if (HAS_LEVEL(selection, service, "write")) {
    return "write";
  }
  return HAS_LEVEL(selection, service, "read") ? "read" : "off";
}

function businessAccess(
  selection: BusinessSelection,
  connector: BusinessConnector,
): AccessChoice {
  if (HAS_BUSINESS_LEVEL(selection, connector, "write")) {
    return "write";
  }
  return HAS_BUSINESS_LEVEL(selection, connector, "read") ? "read" : "off";
}

function updateServiceAccess(
  selection: Selection,
  service: Service,
  access: AccessChoice,
): Selection {
  const withoutService = selection.filter((item) => item.service !== service);
  return access === "off"
    ? withoutService
    : [
        ...withoutService,
        { service, level: "read" },
        ...(access === "write" ? [{ service, level: "write" as const }] : []),
      ];
}

function updateBusinessAccess(
  selection: BusinessSelection,
  connector: BusinessConnector,
  access: AccessChoice,
): BusinessSelection {
  const withoutConnector = selection.filter(
    (item) => item.connector !== connector,
  );
  return access === "off"
    ? withoutConnector
    : [
        ...withoutConnector,
        { connector, level: "read" },
        ...(access === "write" ? [{ connector, level: "write" as const }] : []),
      ];
}

function AccessControl({
  value,
  onChange,
  disabled,
  writeDisabled = false,
  ariaLabel,
}: {
  value: AccessChoice;
  onChange: (value: AccessChoice) => void;
  disabled: boolean;
  writeDisabled?: boolean;
  ariaLabel: string;
}) {
  const options: Array<{
    value: AccessChoice;
    label: string;
    shortLabel: string;
  }> = [
    { value: "off", label: "Apagado", shortLabel: "Apagado" },
    { value: "read", label: "Solo Lectura", shortLabel: "Lectura" },
    { value: "write", label: "Acceso completo", shortLabel: "Completo" },
  ];

  return (
    <fieldset className="inline-flex rounded-lg border border-slate-200 p-0.5 dark:border-zinc-700">
      <legend className="sr-only">{ariaLabel}</legend>
      {options.map((option) => {
        const isSelected = value === option.value;
        const isDisabled =
          disabled || (option.value === "write" && writeDisabled);
        return (
          <button
            key={option.value}
            type="button"
            aria-pressed={isSelected}
            aria-label={option.label}
            disabled={isDisabled}
            onClick={() => onChange(option.value)}
            className={`rounded-md px-2 py-1 text-xs transition-colors sm:px-2.5 ${
              isSelected
                ? "bg-zinc-900 font-medium text-white shadow-sm dark:bg-zinc-100 dark:text-zinc-950"
                : "text-slate-600 hover:bg-slate-100 dark:text-zinc-300 dark:hover:bg-zinc-800"
            } disabled:cursor-not-allowed disabled:opacity-40`}
          >
            <span className="sm:hidden">{option.shortLabel}</span>
            <span className="hidden sm:inline">{option.label}</span>
          </button>
        );
      })}
    </fieldset>
  );
}

const IS_SKILLS_WINDOW =
  new URLSearchParams(window.location.search).get("window") === "skills";

function formatInstallCommand(command: string) {
  return command
    .replace("npx -y skills add ", "npx -y skills add \\\n  ")
    .replace(" -g -y -s ", " \\\n  -g -y \\\n  -s ");
}

/**
 * The skills window: a ZIP download of the granted services' skills, and the `npx skills add`
 * command for the core service skills (the same command as in the install prompt).
 *
 * @example
 * <SkillsWindow />
 */
function SkillsWindow() {
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<Notice>(null);
  const [downloadNotice, setDownloadNotice] = useState<string | null>(null);
  const [commandCopied, setCommandCopied] = useState(false);
  const [installCommand, setInstallCommand] = useState<string | null>(null);

  useEffect(() => {
    invoke<string>("skills_install_command")
      .then(setInstallCommand)
      .catch((error) => setNotice({ kind: "error", text: ERROR_TEXT(error) }));
  }, []);

  async function downloadSkills() {
    setNotice(null);
    setDownloadNotice(null);
    const path = await save({
      defaultPath: "haulmer-googleworkspace-mcp-skills.zip",
      filters: [{ name: "ZIP", extensions: ["zip"] }],
    });
    if (path === null) {
      return;
    }
    setBusy(true);
    try {
      await invoke("export_skills_zip", { path });
      setDownloadNotice("Skills descargadas correctamente.");
    } catch (error) {
      setNotice({ kind: "error", text: ERROR_TEXT(error) });
    } finally {
      setBusy(false);
    }
  }

  /**
   * Copies the install command and shows "Copiado" for a moment.
   *
   * @example
   * await copyCommand();
   */
  async function copyCommand() {
    if (installCommand === null) {
      return;
    }
    try {
      await navigator.clipboard.writeText(installCommand);
      setCommandCopied(true);
      window.setTimeout(() => setCommandCopied(false), 1800);
      setNotice({ kind: "success", text: "Comando copiado al portapapeles." });
    } catch (error) {
      setNotice({ kind: "error", text: ERROR_TEXT(error) });
    }
  }

  return (
    <main className="min-h-screen bg-slate-50 p-6 text-slate-900 dark:bg-zinc-950 dark:text-zinc-100">
      <div className="mx-auto max-w-xl space-y-5">
        <header>
          <p className="text-sm font-medium text-slate-500 dark:text-zinc-400">
            haulmer-googleworkspace-mcp
          </p>
          <h1 className="text-2xl font-semibold">Skills</h1>
        </header>

        <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
          <h2 className="font-semibold">Descargar skills</h2>
          <p className="mt-1 text-sm text-slate-600 dark:text-zinc-400">
            Descarga un ZIP con las skills correspondientes a los permisos
            configurados.
          </p>
          <div className="mt-4 flex items-center gap-2">
            <button
              type="button"
              disabled={busy}
              onClick={downloadSkills}
              className="rounded-lg bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-50 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-300"
            >
              {busy ? "Preparando ZIP…" : "Descargar .zip"}
            </button>
            {downloadNotice !== null && (
              <span className="text-xs text-emerald-700 dark:text-emerald-400">
                {downloadNotice}
              </span>
            )}
          </div>
        </section>

        <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
          <h2 className="font-semibold">Avanzado</h2>
          <p className="mt-1 text-sm text-slate-600 dark:text-zinc-400">
            Instala las skills principales de Google Workspace con el gestor
            oficial.
          </p>
          {installCommand !== null && (
            <div className="mt-4 flex items-start gap-2 rounded-lg bg-slate-100 p-3 dark:bg-zinc-800">
              <pre className="min-w-0 flex-1 whitespace-pre-wrap break-words text-xs">
                {formatInstallCommand(installCommand)}
              </pre>
              <button
                type="button"
                onClick={copyCommand}
                className={`shrink-0 rounded-md border px-2 py-1 text-xs font-medium transition-colors ${
                  commandCopied
                    ? "border-emerald-500 bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300"
                    : "border-slate-300 hover:bg-white dark:border-zinc-600 dark:hover:bg-zinc-700"
                }`}
              >
                {commandCopied ? "Copiado" : "Copiar"}
              </button>
            </div>
          )}
        </section>

        {notice !== null && (
          <p
            role="status"
            className={
              notice.kind === "error"
                ? "text-sm text-red-600 dark:text-red-400"
                : "text-sm text-emerald-700 dark:text-emerald-400"
            }
          >
            {notice.text}
          </p>
        )}
      </div>
    </main>
  );
}

const IS_INSTALL_WINDOW =
  new URLSearchParams(window.location.search).get("window") === "install";

/**
 * The state of the one-click install button in the install window.
 *
 * @example
 * const state: InstallState = { kind: "error", text: "No hay una app para abrirlo." };
 */
type InstallState =
  | { kind: "idle" }
  | { kind: "running" }
  | { kind: "done" }
  | { kind: "error"; text: string };

/**
 * The install window: a one-click MCP extension install, and the install prompt that a person
 * pastes into an agent. It rebuilds the prompt on mount and each time the tray shows the window
 * again, and clears the result of the one-click install.
 *
 * @example
 * <InstallWindow />
 */
function InstallWindow() {
  const [prompt, setPrompt] = useState<InstallPrompt | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);
  const [copyError, setCopyError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [installState, setInstallState] = useState<InstallState>({
    kind: "idle",
  });

  const latestRequest = useRef(0);

  useEffect(() => {
    /**
     * Asks the backend for a fresh prompt. A failure replaces the prompt with its error text. A
     * response applies only if no newer request started meanwhile.
     *
     * @example
     * await loadPrompt();
     */
    async function loadPrompt() {
      latestRequest.current += 1;
      const request = latestRequest.current;
      try {
        const result = await invoke<InstallPrompt>("install_prompt");
        if (request === latestRequest.current) {
          setPrompt(result);
          setLoadError(null);
        }
      } catch (error) {
        if (request === latestRequest.current) {
          setPrompt(null);
          setLoadError(ERROR_TEXT(error));
        }
      }
    }

    /**
     * Back to the loading state, so an old prompt or error never shows, then fetches again.
     *
     * @example
     * onShown(); // when the tray shows the window again
     */
    function onShown() {
      setPrompt(null);
      setLoadError(null);
      setCopyError(null);
      setCopied(false);
      setInstallState({ kind: "idle" });
      loadPrompt();
    }

    loadPrompt();
    const unlisten = listen("install-window-shown", onShown);
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  /**
   * Copies the prompt text and shows "Copiado" for a moment.
   *
   * @example
   * await copyPrompt();
   */
  async function copyPrompt() {
    if (prompt === null) {
      return;
    }
    try {
      await navigator.clipboard.writeText(prompt.text);
      setCopyError(null);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1800);
    } catch (error) {
      setCopyError(ERROR_TEXT(error));
    }
  }

  /**
   * Builds the MCP extension and opens it with the default app. Shows the result or the error.
   * The `/Volumes/` warning that the command returns is ignored: the prompt's warning already
   * shows it.
   *
   * @example
   * await installMcp();
   */
  async function installMcp() {
    setInstallState({ kind: "running" });
    try {
      await invoke<string | null>("install_mcp_extension");
      setInstallState({ kind: "done" });
    } catch (error) {
      setInstallState({ kind: "error", text: ERROR_TEXT(error) });
    }
  }

  return (
    <main className="min-h-screen bg-slate-50 p-6 text-slate-900 dark:bg-zinc-950 dark:text-zinc-100">
      <div className="mx-auto max-w-xl space-y-5">
        <header>
          <p className="text-sm font-medium text-slate-500 dark:text-zinc-400">
            haulmer-googleworkspace-mcp
          </p>
          <h1 className="text-2xl font-semibold">Conectar un agente</h1>
        </header>

        {loadError !== null && (
          <p
            role="alert"
            className="rounded-2xl border border-red-200 bg-red-50 p-5 text-sm text-red-700 dark:border-red-900 dark:bg-red-950 dark:text-red-300"
          >
            {loadError}
          </p>
        )}

        {prompt !== null && (
          <>
            {prompt.warning !== null && (
              <p
                role="status"
                className="rounded-2xl border border-amber-200 bg-amber-50 p-4 text-sm text-amber-800 dark:border-amber-900 dark:bg-amber-950 dark:text-amber-300"
              >
                {prompt.warning}
              </p>
            )}

            <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
              <h2 className="font-semibold">Instalación en un clic</h2>
              <div className="mt-4 flex items-center gap-2">
                <button
                  type="button"
                  disabled={installState.kind === "running"}
                  onClick={installMcp}
                  className="rounded-lg bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-50 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-zinc-300"
                >
                  {installState.kind === "running"
                    ? "Abriendo…"
                    : "Instalar MCP"}
                </button>
                {installState.kind === "done" && (
                  <span
                    role="status"
                    className="text-xs text-emerald-700 dark:text-emerald-400"
                  >
                    Listo. Sigue los pasos en tu app de IA.
                  </span>
                )}
                {installState.kind === "error" && (
                  <span
                    role="alert"
                    className="text-xs text-red-600 dark:text-red-400"
                  >
                    {installState.text}
                  </span>
                )}
              </div>
              <p className="mt-2 text-sm text-slate-600 dark:text-zinc-400">
                Se abre tu app de IA (por ejemplo, Claude Desktop). Ahí pulsa
                "Instalar".
              </p>
            </section>

            <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-zinc-800 dark:bg-zinc-900">
              <h2 className="font-semibold">
                Con un prompt (Claude Code, Codex y otros agentes)
              </h2>
              <ol className="mt-3 list-decimal space-y-1 pl-5 text-sm text-slate-600 dark:text-zinc-400">
                <li>Copia el prompt.</li>
                <li>
                  Pégalo en tu agente (Claude Code, Codex, Hermes, Cursor…) o en
                  ChatGPT de escritorio.
                </li>
                <li>
                  Sigue sus pasos. No funciona en claude.ai ni en chatgpt.com.
                </li>
              </ol>
              <pre className="mt-4 max-h-80 overflow-y-auto whitespace-pre-wrap rounded-lg bg-slate-100 p-3 font-mono text-xs dark:bg-zinc-800">
                {prompt.text}
              </pre>
              <div className="mt-4 flex items-center gap-2">
                <button
                  type="button"
                  onClick={copyPrompt}
                  className={`rounded-lg border px-4 py-2 text-sm font-medium transition-colors ${
                    copied
                      ? "border-emerald-500 bg-emerald-50 text-emerald-700 dark:bg-emerald-950 dark:text-emerald-300"
                      : "border-slate-300 hover:bg-slate-50 dark:border-zinc-600 dark:hover:bg-zinc-800"
                  }`}
                >
                  {copied ? "Copiado" : "Copiar"}
                </button>
                {copyError !== null && (
                  <span
                    role="alert"
                    className="text-xs text-red-600 dark:text-red-400"
                  >
                    {copyError}
                  </span>
                )}
              </div>
            </section>
          </>
        )}
      </div>
    </main>
  );
}

/**
 * The settings window of haulmer-googleworkspace-mcp: account, services, Save, Revoke all and the Advanced
 * settings. It also routes the skills and install windows to their own components.
 *
 * @example
 * <App />
 */
export default function App() {
  const [status, setStatus] = useState<Status | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);
  const [catalogue, setCatalogue] = useState<ServiceInfo[]>([]);
  const [businessCatalogue, setBusinessCatalogue] = useState<
    BusinessConnectorInfo[]
  >([]);
  const [businessSelection, setBusinessSelection] = useState<BusinessSelection>(
    [],
  );
  const [catalogueError, setCatalogueError] = useState<string | null>(null);
  const [selection, setSelection] = useState<Selection>([]);
  const [busy, setBusy] = useState<Busy>(null);
  const [authUrl, setAuthUrl] = useState<string | null>(null);
  const [notice, setNotice] = useState<Notice>(null);
  const [update, setUpdate] = useState<Update | null>(null);

  // biome-ignore lint/correctness/useExhaustiveDependencies: this startup effect intentionally runs once.
  useEffect(() => {
    if (IS_SKILLS_WINDOW || IS_INSTALL_WINDOW) {
      return;
    }
    const unlisten = listen<string>("auth-url", (event) =>
      setAuthUrl(event.payload),
    );
    loadAll();
    checkForUpdate();
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

  if (IS_SKILLS_WINDOW) {
    return <SkillsWindow />;
  }

  if (IS_INSTALL_WINDOW) {
    return <InstallWindow />;
  }

  /**
   * Looks for a newer release on start. A failed check is silent (console only).
   *
   * @example
   * await checkForUpdate();
   */
  async function checkForUpdate() {
    try {
      setUpdate(await check());
    } catch (error) {
      console.error("Update check failed:", error);
    }
  }

  /**
   * Downloads and installs the available update, then restarts the app.
   *
   * @example
   * <button onClick={handleInstallUpdate}>Install and restart</button>
   */
  async function handleInstallUpdate() {
    if (update === null) {
      return;
    }
    setBusy("update");
    setNotice(null);
    try {
      await update.downloadAndInstall();
      await relaunch();
    } catch (error) {
      setNotice({ kind: "error", text: ERROR_TEXT(error) });
    } finally {
      setBusy(null);
    }
  }

  /**
   * Loads the status, the service labels and the starting selection
   * (what Google granted when connected, the safe default otherwise).
   *
   * @example
   * await loadAll();
   */
  async function loadAll() {
    const [statusResult, catalogueResult, businessResult, defaultsResult] =
      await Promise.allSettled([
        invoke<Status>("status"),
        invoke<ServiceInfo[]>("catalogue"),
        invoke<BusinessConnectorInfo[]>("business_catalogue"),
        invoke<Selection>("default_selection"),
      ]);
    if (catalogueResult.status === "fulfilled") {
      setCatalogue(catalogueResult.value);
      setCatalogueError(null);
    } else {
      setCatalogueError(ERROR_TEXT(catalogueResult.reason));
    }
    if (businessResult.status === "fulfilled") {
      setBusinessCatalogue(businessResult.value);
    }
    if (statusResult.status === "fulfilled") {
      setStatus(statusResult.value);
      setStatusError(null);
    } else {
      setStatusError(ERROR_TEXT(statusResult.reason));
    }
    if (
      statusResult.status === "fulfilled" &&
      statusResult.value.account !== null
    ) {
      setSelection(statusResult.value.granted);
      setBusinessSelection(statusResult.value.business_granted);
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
      let outcome = await invoke<SaveOutcome>("save", {
        selection,
        businessSelection,
        confirmReplace: false,
      });
      if (outcome.kind === "needs_replace_confirmation") {
        const replace = await ask(
          "Ya tienes gws conectado con otra aplicación. ¿Quieres reemplazarlo?",
          {
            title: "Reemplazar conexión de gws",
            kind: "warning",
            okLabel: "Sí",
            cancelLabel: "No",
          },
        );
        if (!replace) {
          return;
        }
        outcome = await invoke<SaveOutcome>("save", {
          selection,
          businessSelection,
          confirmReplace: true,
        });
      }
      if (outcome.kind === "connected") {
        const account = outcome.status.account;
        setStatus(outcome.status);
        setStatusError(null);
        setSelection(outcome.status.granted);
        setBusinessSelection(outcome.status.business_granted);
        const text =
          account === null
            ? "Guardado. Se eliminó todo el acceso."
            : `Guardado. Conectado como ${account}.`;
        setNotice({ kind: "success", text });
      }
    } catch (error) {
      const text = ERROR_TEXT(error);
      if (text === "Google sign-in was cancelled") {
        setNotice({
          kind: "info",
          text: "Se canceló el inicio de sesión de Google. No se realizaron cambios.",
        });
      } else {
        setNotice({ kind: "error", text });
      }
      // A failed save may already have removed access (e.g. a reduced selection): show the truth.
      await refreshStatus();
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
      "¿Revocar todo el acceso? Tus agentes perderán acceso a Google hasta que vuelvas a conectar la cuenta.",
      {
        title: "Revocar todo",
        kind: "warning",
        okLabel: "Revocar todo",
        cancelLabel: "Mantener acceso",
      },
    );
    if (!confirmed) {
      return;
    }
    setBusy("revoke");
    setNotice(null);
    try {
      let outcome = await invoke<RevokeOutcome>("revoke_all", {
        confirmForeign: false,
      });
      if (outcome.kind === "needs_foreign_confirmation") {
        const revokeForeign = await ask(
          "Estas credenciales de gws fueron creadas por otra aplicación. ¿Revocarlas de todas formas?",
          {
            title: "Revocar todo",
            kind: "warning",
            okLabel: "Sí",
            cancelLabel: "No",
          },
        );
        if (!revokeForeign) {
          setNotice({ kind: "info", text: "No se revocó nada." });
          return;
        }
        outcome = await invoke<RevokeOutcome>("revoke_all", {
          confirmForeign: true,
        });
      }
      if (outcome.kind === "revoked") {
        setNotice({ kind: "success", text: "Se revocó todo el acceso." });
      }
    } catch (error) {
      setNotice({ kind: "error", text: ERROR_TEXT(error) });
    } finally {
      await refreshStatus();
      setBusy(null);
    }
  }

  let accountText: string;
  if (status === null && statusError === null) {
    accountText = "Comprobando…";
  } else if (status === null) {
    accountText = "Estado no disponible";
  } else if (status.account !== null) {
    accountText = status.account;
  } else {
    accountText = "Sin conexión";
  }

  const accountInitials =
    status?.account
      ?.split("@")[0]
      .split(/[._-]/)
      .filter(Boolean)
      .slice(0, 2)
      .map((part) => part.charAt(0).toUpperCase())
      .join("") || "?";
  const disabled = busy !== null;
  const selectionKey = (
    items: Array<{
      level: "read" | "write";
      service?: string;
      connector?: string;
    }>,
  ) => {
    const effectivePermissions = new Set<string>();
    for (const item of items) {
      const resource = item.service ?? item.connector;
      if (resource === undefined) {
        continue;
      }
      effectivePermissions.add(`${resource}:read`);
      if (item.level === "write") {
        effectivePermissions.add(`${resource}:write`);
      }
    }
    return [...effectivePermissions].sort().join("|");
  };
  const hasChanges =
    status !== null &&
    (status.account === null
      ? selection.length > 0 || businessSelection.length > 0
      : selectionKey(selection) !== selectionKey(status.granted) ||
        selectionKey(businessSelection) !==
          selectionKey(status.business_granted));
  const saveDisabled = disabled || !hasChanges;
  const statusDotClass =
    status === null || statusError !== null
      ? "bg-amber-400"
      : status.account !== null && status.token_valid
        ? "bg-emerald-500"
        : status.account !== null
          ? "bg-amber-400"
          : "bg-slate-400";

  let noticeClass =
    "border-red-200 bg-red-50 text-red-800 dark:border-red-900 dark:bg-red-950 dark:text-red-200";
  if (notice?.kind === "success") {
    noticeClass =
      "border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-200";
  } else if (notice?.kind === "info") {
    noticeClass =
      "border-slate-200 bg-slate-50 text-slate-700 dark:border-zinc-700 dark:bg-zinc-900 dark:text-zinc-200";
  }

  return (
    <>
      <main className="mx-auto min-h-screen max-w-xl space-y-6 bg-white p-8 pb-32 font-sans text-slate-900 dark:bg-zinc-950 dark:text-zinc-100">
        {update !== null && (
          <div className="flex items-center justify-between gap-3 rounded-lg border border-slate-200 bg-slate-50 p-3 text-sm dark:border-zinc-700 dark:bg-zinc-900">
            <span className="text-slate-700 dark:text-zinc-200">
              La versión {update.version} está disponible
            </span>
            <button
              type="button"
              className="rounded-md bg-slate-900 px-3 py-1.5 font-medium text-white hover:bg-slate-700 disabled:opacity-40 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-white"
              disabled={disabled}
              onClick={handleInstallUpdate}
            >
              {busy === "update" ? "Instalando…" : "Instalar y reiniciar"}
            </button>
          </div>
        )}

        <header>
          <h1 className="text-xl font-semibold">haulmer-googleworkspace-mcp</h1>
          <p className="mt-1 text-sm text-slate-500 dark:text-zinc-400">
            Conecta tus agentes de IA a Google Workspace.
          </p>
        </header>

        <section className="space-y-2">
          {statusError !== null && (
            <p className="mt-1 text-sm text-slate-500 dark:text-zinc-400">
              {statusError}
            </p>
          )}
          {status !== null &&
            status.account !== null &&
            !status.token_valid && (
              <p className="mt-1 text-sm text-amber-700 dark:text-amber-300">
                La sesión expiró. Pulsa Guardar para iniciar sesión nuevamente.
              </p>
            )}
          {status !== null && !status.gws_available && (
            <p className="mt-1 text-sm text-slate-500 dark:text-zinc-400">
              gws se descargará al pulsar Guardar.
            </p>
          )}
        </section>

        <section className="space-y-3">
          <h2 className="text-sm font-semibold text-slate-700 dark:text-zinc-200">
            Lo que pueden usar tus agentes
          </h2>
          {catalogueError !== null && (
            <p className="text-sm text-slate-500 dark:text-zinc-400">
              Servicios no disponibles: {catalogueError}
            </p>
          )}
          <ul className="divide-y divide-slate-100 rounded-lg border border-slate-200 dark:divide-zinc-800 dark:border-zinc-700">
            {catalogue.map((info) => (
              <li
                key={info.service}
                className="flex items-center gap-3 px-3 py-2.5"
              >
                <ServiceIcon
                  src={SERVICE_ICONS[info.service]}
                  label={info.name}
                />
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{info.name}</p>
                </div>
                <div className="flex shrink-0 items-center gap-1">
                  {info.availability_reason !== null && (
                    <AvailabilityWarning message={info.availability_reason} />
                  )}
                  <AccessControl
                    ariaLabel={`${info.name} nivel de acceso`}
                    value={serviceAccess(selection, info.service)}
                    onChange={(access) =>
                      setSelection(
                        updateServiceAccess(selection, info.service, access),
                      )
                    }
                    disabled={disabled || !info.enabled}
                    writeDisabled={!info.write_enabled}
                  />
                </div>
              </li>
            ))}
          </ul>

          <h2 className="pt-3 text-sm font-semibold text-slate-700 dark:text-zinc-200">
            Conectores empresariales de Google
          </h2>
          <ul className="divide-y divide-slate-100 rounded-lg border border-slate-200 dark:divide-zinc-800 dark:border-zinc-700">
            {businessCatalogue.map((info) => (
              <li
                key={info.connector}
                className="flex items-center gap-3 px-3 py-2.5"
              >
                <ServiceIcon
                  src={BUSINESS_ICONS[info.connector]}
                  label={info.name}
                />
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-medium">{info.name}</p>
                </div>
                <div className="flex shrink-0 items-center gap-1">
                  {info.availability_reason !== null && (
                    <AvailabilityWarning message={info.availability_reason} />
                  )}
                  <AccessControl
                    ariaLabel={`${info.name} nivel de acceso`}
                    value={businessAccess(businessSelection, info.connector)}
                    onChange={(access) =>
                      setBusinessSelection(
                        updateBusinessAccess(
                          businessSelection,
                          info.connector,
                          access,
                        ),
                      )
                    }
                    disabled={disabled || !info.enabled}
                  />
                </div>
              </li>
            ))}
          </ul>

          {busy === "save" && (
            <div className="flex items-center justify-between gap-3 rounded-lg bg-slate-50 p-3 text-sm dark:bg-zinc-900">
              {authUrl === null && (
                <span className="text-slate-600 dark:text-zinc-300">
                  Preparando…
                </span>
              )}
              {authUrl !== null && (
                <span className="text-slate-600 dark:text-zinc-300">
                  Esperando el inicio de sesión de Google…{" "}
                  <button
                    type="button"
                    className="underline"
                    onClick={() => openUrl(authUrl)}
                  >
                    Abrir nuevamente
                  </button>
                </span>
              )}
            </div>
          )}
        </section>

        {notice !== null && (
          <p className={`rounded-lg border p-3 text-sm ${noticeClass}`}>
            {notice.text}
          </p>
        )}

        <section className="border-t border-slate-200 pt-6 text-sm dark:border-zinc-800">
          <button
            type="button"
            className="font-medium text-red-600 underline-offset-4 hover:underline disabled:cursor-not-allowed disabled:opacity-40 dark:text-red-400"
            disabled={disabled}
            onClick={handleRevokeAll}
          >
            Revocar todo
          </button>
        </section>
      </main>
      <div className="fixed bottom-3 left-1/2 z-30 w-[calc(100%-1.5rem)] max-w-xl -translate-x-1/2 rounded-2xl border border-slate-200/80 bg-white/95 px-4 py-3 shadow-[0_8px_30px_rgba(15,23,42,0.14)] backdrop-blur dark:border-zinc-700/80 dark:bg-zinc-950/95 dark:shadow-[0_8px_30px_rgba(0,0,0,0.35)]">
        <div className="flex items-center gap-3">
          <div className="relative shrink-0">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-slate-200 text-sm font-semibold text-slate-700 dark:bg-zinc-700 dark:text-zinc-100">
              {accountInitials}
            </div>
            <span
              className={`absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-white dark:border-zinc-950 ${statusDotClass}`}
            />
          </div>
          <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-medium">{accountText}</p>
            <p className="text-xs text-slate-500 dark:text-zinc-400">
              {status?.account === null && hasChanges
                ? "Esperando tu permiso"
                : hasChanges
                  ? "Hay cambios sin guardar"
                  : status?.account
                    ? "Todo guardado"
                    : "Selecciona servicios para conectar"}
            </p>
          </div>
          <div className="flex shrink-0 items-center gap-2">
            {busy === "save" && (
              <button
                type="button"
                className="rounded-md border border-slate-300 px-3 py-2 text-sm hover:bg-slate-50 dark:border-zinc-600 dark:hover:bg-zinc-800"
                onClick={handleCancel}
              >
                Cancelar
              </button>
            )}
            <button
              type="button"
              className="rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-40 dark:bg-zinc-100 dark:text-zinc-900 dark:hover:bg-white"
              disabled={saveDisabled}
              onClick={handleSave}
            >
              {busy === "save"
                ? status?.account
                  ? "Guardando…"
                  : "Conectando…"
                : status?.account
                  ? "Guardar"
                  : "Conectar"}
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
