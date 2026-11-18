import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { ask, open } from "@tauri-apps/plugin-dialog";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import {
  ERROR_TEXT,
  HAS_LEVEL,
  type InstalledSkills,
  type BusinessConnector,
  type BusinessConnectorInfo,
  type BusinessSelection,
  HAS_BUSINESS_LEVEL,
  type KeyStorage,
  type RevokeOutcome,
  type SaveOutcome,
  type Selection,
  type Service,
  type ServiceInfo,
  type Status,
} from "./core";
import {
  SegmentedControl,
  SegmentedControlIndicator,
  SegmentedControlItem,
} from "@ui-sdk/react/segmented-control";
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
import mediaIcon from "./assets/service-icons/media.png";
import meetIcon from "./assets/service-icons/meet.png";
import marketplaceIcon from "./assets/service-icons/marketplace.png";
import peopleIcon from "./assets/service-icons/people.png";
import securityIdentityIcon from "./assets/service-icons/security-identity.png";
import sheetsIcon from "./assets/service-icons/sheets.png";
import slidesIcon from "./assets/service-icons/slides.png";
import tasksIcon from "./assets/service-icons/tasks.png";
import webMobileIcon from "./assets/service-icons/web-mobile.png";

/** Google's "Third-party apps & services" page, where the person can also revoke access. */
const GOOGLE_CONNECTIONS_URL = "https://myaccount.google.com/connections";
const MCP_INSTALL_PROMPT = `Configura haulmer-connect como servidor MCP local, pero no modifiques ningún archivo sin pedirme confirmación.

La aplicación ya está instalada en este equipo. Usa su ejecutable y agrega:
- Claude Desktop: mcpServers.haulmer-connect con args ["mcp"] en claude_desktop_config.json.
- ChatGPT/Codex: [mcp_servers.haulmer-connect] con args = ["mcp"] en ~/.codex/config.toml.

Conserva las demás configuraciones, explícame qué cambiarías y dime qué cliente debo reiniciar.`;

/** The one action running right now; controls stay disabled while one runs. */
type Busy = "save" | "revoke" | "install" | "mcp" | "advanced" | "update" | null;

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
    return <img src={src} alt="" aria-hidden="true" className="h-7 w-7 shrink-0 object-contain" />;
  }
  return (
    <span
      aria-hidden="true"
      className="flex h-7 w-7 shrink-0 items-center justify-center rounded-lg bg-slate-100 text-sm font-semibold text-slate-500 dark:bg-slate-800 dark:text-slate-300"
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

function businessAccess(selection: BusinessSelection, connector: BusinessConnector): AccessChoice {
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
    : [...withoutService, { service, level: access === "write" ? "write" : "read" }];
}

function updateBusinessAccess(
  selection: BusinessSelection,
  connector: BusinessConnector,
  access: AccessChoice,
): BusinessSelection {
  const withoutConnector = selection.filter((item) => item.connector !== connector);
  return access === "off"
    ? withoutConnector
    : [...withoutConnector, { connector, level: access === "write" ? "write" : "read" }];
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
  return (
    <SegmentedControl
      aria-label={ariaLabel}
      value={value}
      onValueChange={(next) => {
        if (next === "off" || next === "read" || next === "write") {
          onChange(next);
        }
      }}
      disabled={disabled}
      className="isolate inline-flex rounded-md border border-slate-200 bg-slate-100 p-1 dark:border-slate-700 dark:bg-slate-800"
    >
      <SegmentedControlIndicator className="rounded bg-white shadow-sm dark:bg-slate-600" />
      <SegmentedControlItem
        value="off"
        className="relative rounded px-1.5 py-1 text-xs text-slate-600 transition-colors data-[checked]:font-medium data-[checked]:text-slate-950 dark:text-slate-300 dark:data-[checked]:text-white sm:px-2"
      >
        Apagado
      </SegmentedControlItem>
      <SegmentedControlItem
        value="read"
        className="relative rounded px-1.5 py-1 text-xs text-slate-600 transition-colors data-[checked]:font-medium data-[checked]:text-slate-950 dark:text-slate-300 dark:data-[checked]:text-white sm:px-2"
      >
        <span className="sm:hidden">Lectura</span>
        <span className="hidden sm:inline">Solo Lectura</span>
      </SegmentedControlItem>
      <SegmentedControlItem
        value="write"
        disabled={writeDisabled}
        className="relative rounded px-1.5 py-1 text-xs text-slate-600 transition-colors data-[checked]:font-medium data-[checked]:text-slate-950 dark:text-slate-300 dark:data-[checked]:text-white sm:px-2"
      >
        <span className="sm:hidden">Completo</span>
        <span className="hidden sm:inline">Acceso completo</span>
      </SegmentedControlItem>
    </SegmentedControl>
  );
}

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
  const [businessCatalogue, setBusinessCatalogue] = useState<BusinessConnectorInfo[]>([]);
  const [businessSelection, setBusinessSelection] = useState<BusinessSelection>([]);
  const [catalogueError, setCatalogueError] = useState<string | null>(null);
  const [selection, setSelection] = useState<Selection>([]);
  const [busy, setBusy] = useState<Busy>(null);
  const [authUrl, setAuthUrl] = useState<string | null>(null);
  const [notice, setNotice] = useState<Notice>(null);
  const [update, setUpdate] = useState<Update | null>(null);
  const [showInstallPrompt, setShowInstallPrompt] = useState(false);

  useEffect(() => {
    const unlisten = listen<string>("auth-url", (event) => setAuthUrl(event.payload));
    loadAll();
    checkForUpdate();
    return () => {
      unlisten.then((stop) => stop());
    };
  }, []);

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
    const [statusResult, catalogueResult, businessResult, defaultsResult] = await Promise.allSettled([
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
    if (statusResult.status === "fulfilled" && statusResult.value.account !== null) {
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
        const replace = await ask("Ya tienes gws conectado con otra aplicación. ¿Quieres reemplazarlo?", {
          title: "Reemplazar conexión de gws",
          kind: "warning",
          okLabel: "Sí",
          cancelLabel: "No",
        });
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
        const text = account === null ? "Guardado. Se eliminó todo el acceso." : `Guardado. Conectado como ${account}.`;
        setNotice({ kind: "success", text });
      }
    } catch (error) {
      const text = ERROR_TEXT(error);
      if (text === "Google sign-in was cancelled") {
        setNotice({ kind: "info", text: "Se canceló el inicio de sesión de Google. No se realizaron cambios." });
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
      { title: "Revocar todo", kind: "warning", okLabel: "Revocar todo", cancelLabel: "Mantener acceso" },
    );
    if (!confirmed) {
      return;
    }
    setBusy("revoke");
    setNotice(null);
    try {
      let outcome = await invoke<RevokeOutcome>("revoke_all", { confirmForeign: false });
      if (outcome.kind === "needs_foreign_confirmation") {
        const revokeForeign = await ask(
          "Estas credenciales de gws fueron creadas por otra aplicación. ¿Revocarlas de todas formas?",
          { title: "Revocar todo", kind: "warning", okLabel: "Sí", cancelLabel: "No" },
        );
        if (!revokeForeign) {
          setNotice({ kind: "info", text: "No se revocó nada." });
          return;
        }
        outcome = await invoke<RevokeOutcome>("revoke_all", { confirmForeign: true });
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

  /** Copies non-invasive MCP setup instructions for the user's AI client. */
  async function handleCopyInstallPrompt() {
    setBusy("mcp");
    setNotice(null);
    try {
      await navigator.clipboard.writeText(MCP_INSTALL_PROMPT);
      setShowInstallPrompt(true);
      setNotice({ kind: "success", text: "Prompt copiado. Pégalo en Claude, ChatGPT o Codex." });
    } catch (error) {
      setNotice({ kind: "error", text: ERROR_TEXT(error) });
    } finally {
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
        setNotice({ kind: "info", text: "No se encontraron agentes en este equipo." });
      } else {
        try {
          await revealItemInDir(installed[0].target);
        } catch (error) {
          setNotice({
            kind: "error",
            text: `Las skills se instalaron, pero no se pudo abrir la carpeta: ${ERROR_TEXT(error)}`,
          });
          return;
        }
        const targets = installed.map((item) => item.target).join(", ");
        const skipped = [...new Set(installed.flatMap((item) => item.skipped))];
        let text = `Skills instaladas en: ${targets}.`;
        if (skipped.length > 0) {
          text += ` Omitidas (instaladas por otra aplicación): ${skipped.join(", ")}.`;
        }
        setNotice({ kind: "success", text });
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
      setNotice({ kind: "success", text: "Carpeta de configuración guardada." });
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
      setNotice({ kind: "success", text: "Almacenamiento de claves guardado." });
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
  const selectionKey = (items: Array<object>) =>
    JSON.stringify([...items].sort((a, b) => JSON.stringify(a).localeCompare(JSON.stringify(b))));
  const hasChanges =
    status !== null &&
    (status.account === null
      ? selection.length > 0 || businessSelection.length > 0
      : selectionKey(selection) !== selectionKey(status.granted) ||
        selectionKey(businessSelection) !== selectionKey(status.business_granted));
  const saveDisabled = disabled || !hasChanges;
  const statusDotClass =
    status === null || statusError !== null
      ? "bg-amber-400"
      : status.account !== null && status.token_valid
        ? "bg-emerald-500"
        : status.account !== null
          ? "bg-amber-400"
          : "bg-slate-400";

  let noticeClass = "border-red-200 bg-red-50 text-red-800 dark:border-red-900 dark:bg-red-950 dark:text-red-200";
  if (notice?.kind === "success") {
    noticeClass = "border-emerald-200 bg-emerald-50 text-emerald-800 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-200";
  } else if (notice?.kind === "info") {
    noticeClass = "border-slate-200 bg-slate-50 text-slate-700 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-200";
  }

  let keyStorageHint = "";
  if (status?.key_storage === "auto" && status.resolved_key_storage === "file") {
    keyStorageHint = "Automático usa un archivo en la carpeta de configuración.";
  } else if (status?.key_storage === "auto") {
    keyStorageHint = "Automático usa el llavero del sistema.";
  }

  return (
    <>
      <main className="mx-auto min-h-screen max-w-xl space-y-6 bg-white p-8 pb-32 font-sans text-slate-900 dark:bg-slate-950 dark:text-slate-100">
      {update !== null && (
        <div className="flex items-center justify-between gap-3 rounded-lg border border-slate-200 bg-slate-50 p-3 text-sm dark:border-slate-700 dark:bg-slate-900">
          <span className="text-slate-700 dark:text-slate-200">La versión {update.version} está disponible</span>
          <button
            type="button"
            className="rounded-md bg-slate-900 px-3 py-1.5 font-medium text-white hover:bg-slate-700 disabled:opacity-40 dark:bg-slate-100 dark:text-slate-900 dark:hover:bg-white"
            disabled={disabled}
            onClick={handleInstallUpdate}
          >
            {busy === "update" ? "Instalando…" : "Instalar y reiniciar"}
          </button>
        </div>
      )}

      <header>
        <h1 className="text-xl font-semibold">haulmer-connect</h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">Conecta tus agentes de IA a Google Workspace.</p>
      </header>

      <section className="space-y-2">
        {statusError !== null && <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">{statusError}</p>}
        {status !== null && status.account !== null && !status.token_valid && (
          <p className="mt-1 text-sm text-amber-700 dark:text-amber-300">La sesión expiró. Pulsa Guardar para iniciar sesión nuevamente.</p>
        )}
        {status !== null && !status.gws_available && (
          <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">gws se descargará al pulsar Guardar.</p>
        )}
      </section>

      <section className="space-y-3">
        <h2 className="text-sm font-semibold text-slate-700 dark:text-slate-200">Lo que pueden usar tus agentes</h2>
        {catalogueError !== null && (
          <p className="text-sm text-slate-500 dark:text-slate-400">Servicios no disponibles: {catalogueError}</p>
        )}
        <ul className="divide-y divide-slate-100 rounded-lg border border-slate-200 dark:divide-slate-800 dark:border-slate-700">
          {catalogue.map((info) => (
            <li key={info.service} className="flex items-center gap-3 px-3 py-2.5">
              <ServiceIcon src={SERVICE_ICONS[info.service]} label={info.name} />
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
                    setSelection(updateServiceAccess(selection, info.service, access))
                  }
                  disabled={disabled || !info.enabled}
                  writeDisabled={!info.write_enabled}
                />
              </div>
            </li>
          ))}
        </ul>

        <h2 className="pt-3 text-sm font-semibold text-slate-700 dark:text-slate-200">Conectores empresariales de Google</h2>
        <ul className="divide-y divide-slate-100 rounded-lg border border-slate-200 dark:divide-slate-800 dark:border-slate-700">
          {businessCatalogue.map((info) => (
            <li key={info.connector} className="flex items-center gap-3 px-3 py-2.5">
              <ServiceIcon src={BUSINESS_ICONS[info.connector]} label={info.name} />
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
                      updateBusinessAccess(businessSelection, info.connector, access),
                    )
                  }
                  disabled={disabled || !info.enabled}
                />
              </div>
            </li>
          ))}
        </ul>

        {busy === "save" && (
          <div className="flex items-center justify-between gap-3 rounded-lg bg-slate-50 p-3 text-sm dark:bg-slate-900">
            {authUrl === null && <span className="text-slate-600 dark:text-slate-300">Preparando…</span>}
            {authUrl !== null && (
              <span className="text-slate-600 dark:text-slate-300">
                Esperando el inicio de sesión de Google…{" "}
                <button type="button" className="underline" onClick={() => openUrl(authUrl)}>
                  Abrir nuevamente
                </button>
              </span>
            )}
            <button
              type="button"
              className="rounded-md border border-slate-300 px-3 py-1 hover:bg-white dark:border-slate-600 dark:hover:bg-slate-800"
              onClick={handleCancel}
            >
              Cancelar
            </button>
          </div>
        )}
      </section>

      {notice !== null && (
        <p className={`rounded-lg border p-3 text-sm ${noticeClass}`}>{notice.text}</p>
      )}

      <section className="flex flex-wrap items-center gap-3 border-t border-slate-200 pt-6 text-sm">
        <span className="font-medium">Usar desde aplicaciones de IA de escritorio</span>
        <button
          type="button"
          className="rounded-md border border-slate-300 px-3 py-1.5 hover:bg-slate-50 disabled:opacity-40 dark:border-slate-600 dark:hover:bg-slate-800"
          disabled={disabled}
          onClick={handleCopyInstallPrompt}
        >
          Copiar prompt de instalación
        </button>
        {showInstallPrompt && (
          <pre className="basis-full whitespace-pre-wrap rounded-md bg-slate-100 p-3 text-xs text-slate-600 dark:bg-slate-900 dark:text-slate-300">
            {MCP_INSTALL_PROMPT}
          </pre>
        )}
      </section>

      <section className="flex flex-wrap items-center gap-3 border-t border-slate-200 pt-6 text-sm">
        <button
          type="button"
          className="rounded-md border border-slate-300 px-3 py-1.5 hover:bg-slate-50 disabled:opacity-40 dark:border-slate-600 dark:hover:bg-slate-800"
          disabled={disabled}
          onClick={handleInstall}
        >
          Instalar y abrir carpeta de skills
        </button>
        <button
          type="button"
          className="rounded-md border border-red-200 px-3 py-1.5 text-red-700 hover:bg-red-50 disabled:opacity-40 dark:border-red-900 dark:text-red-300 dark:hover:bg-red-950"
          disabled={disabled}
          onClick={handleRevokeAll}
        >
          Revocar todo
        </button>
        <a
          href={GOOGLE_CONNECTIONS_URL}
          className="text-slate-500 underline hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200"
          onClick={(event) => {
            event.preventDefault();
            openUrl(GOOGLE_CONNECTIONS_URL);
          }}
        >
          Comprobar acceso en Google
        </a>
      </section>

      <details className="rounded-lg border border-slate-200 p-4 text-sm dark:border-slate-700">
        <summary className="cursor-pointer font-medium text-slate-700 dark:text-slate-200">Avanzado</summary>
        <div className="mt-4 space-y-4">
          <div>
            <p className="text-slate-500 dark:text-slate-400">Carpeta de configuración</p>
            <p className="mt-1 break-all font-mono text-xs">
              {status?.config_dir ?? "Desconocida"}
              {status?.config_dir_is_default && (
                <span className="font-sans text-slate-400 dark:text-slate-500"> (predeterminada)</span>
              )}
            </p>
            <div className="mt-2 flex gap-2">
              <button
                type="button"
                className="rounded-md border border-slate-300 px-3 py-1 hover:bg-slate-50 disabled:opacity-40 dark:border-slate-600 dark:hover:bg-slate-800"
                disabled={disabled}
                onClick={handleChooseFolder}
              >
                Elegir carpeta…
              </button>
              <button
                type="button"
                className="rounded-md border border-slate-300 px-3 py-1 hover:bg-slate-50 disabled:opacity-40 dark:border-slate-600 dark:hover:bg-slate-800"
                disabled={disabled || status === null || status.config_dir_is_default}
                onClick={() => changeConfigDir(null)}
              >
                Restablecer predeterminada
              </button>
            </div>
          </div>
          <label className="block">
            <span className="text-slate-500 dark:text-slate-400">Almacenamiento de claves</span>
            <select
              className="mt-1 block rounded-md border border-slate-300 bg-white px-2 py-1 dark:border-slate-600 dark:bg-slate-900"
              disabled={disabled}
              value={status?.key_storage ?? "auto"}
              onChange={(event) => handleKeyStorage(event.target.value as KeyStorage)}
            >
              <option value="auto">Automático</option>
              <option value="keyring">Llavero</option>
              <option value="file">Archivo</option>
            </select>
            {keyStorageHint !== "" && (
              <span className="mt-1 block text-xs text-slate-400 dark:text-slate-500">{keyStorageHint}</span>
            )}
          </label>
        </div>
      </details>
      </main>
      <div className="fixed inset-x-0 bottom-0 z-30 border-t border-slate-200/80 bg-white/95 px-4 py-3 shadow-[0_-8px_24px_rgba(15,23,42,0.08)] backdrop-blur dark:border-slate-700/80 dark:bg-slate-950/95 dark:shadow-[0_-8px_24px_rgba(0,0,0,0.25)]">
        <div className="mx-auto flex max-w-xl items-center gap-3">
          <div className="relative shrink-0">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-slate-200 text-sm font-semibold text-slate-700 dark:bg-slate-700 dark:text-slate-100">
              {accountInitials}
            </div>
            <span className={`absolute -bottom-0.5 -right-0.5 h-3 w-3 rounded-full border-2 border-white dark:border-slate-950 ${statusDotClass}`} />
          </div>
          <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-medium">{accountText}</p>
            <p className="text-xs text-slate-500 dark:text-slate-400">
              {hasChanges ? "Hay cambios sin guardar" : status?.account ? "Todo guardado" : "Selecciona servicios para conectar"}
            </p>
          </div>
          <button
            type="button"
            className="shrink-0 rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:cursor-not-allowed disabled:opacity-40 dark:bg-slate-100 dark:text-slate-900 dark:hover:bg-white"
            disabled={saveDisabled}
            onClick={handleSave}
          >
            {busy === "save" ? "Guardando…" : "Guardar"}
          </button>
        </div>
      </div>
    </>
  );
}
