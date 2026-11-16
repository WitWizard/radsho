import { Download } from "lucide-react";
import { useApp } from "../state/AppContext";
import { ALL_TRANSPORTS, APP_VERSION, CORE_VERSION, SCAN_MODES, SHUFFLE_REGIONS, portsValid } from "../lib/config";
import { serverByRegion, serverLabel } from "../lib/servers";
import { transportLabelKey, type TKey } from "../i18n/translations";
import type { Lang, Ports, ScanMode, ThemeMode, TransportId } from "../types/vpn";
import { Page, Card } from "../components/Page";
import { Field, SettingRow, inputClass } from "../components/SettingRow";
import { Button, CustomSelect, Segmented, type SelectOption } from "../components/ui";
import { AboutCard } from "../components/AboutCard";
import { Flag } from "../components/Flag";

const PORT_FIELDS: Array<{ key: keyof Ports; label: TKey }> = [
  { key: "bind", label: "portBind" },
  { key: "http", label: "portHttp" },
  { key: "psiphon", label: "portPsiphon" },
  { key: "psiphonHttp", label: "portPsiphonHttp" },
];

export function SettingsView() {
  const { t, settings, patchSettings, updateInfo, setModal, isMobile } = useApp();
  const valid = portsValid(settings);

  const setPort = (key: keyof Ports, raw: string) => {
    const value = Number(raw.replace(/\D/g, ""));
    patchSettings({ ports: { ...settings.ports, [key]: value } });
  };

  const themes: Array<{ value: ThemeMode; label: string }> = [
    { value: "system", label: t("themeSystem") },
    { value: "light", label: t("themeLight") },
    { value: "dark", label: t("themeDark") },
  ];

  const languages: Array<{ value: Lang; label: string }> = [
    { value: "en", label: t("langEn") },
    { value: "fa", label: t("langFa") },
  ];

  const transportOptions: Array<SelectOption<TransportId>> = ALL_TRANSPORTS.map((id) => ({
    value: id,
    label: t(transportLabelKey[id]),
  }));

  const scanOptions: Array<SelectOption<ScanMode>> = SCAN_MODES.map((mode) => ({
    value: mode,
    label: mode,
  }));

  const regionOptions: Array<SelectOption<string>> = SHUFFLE_REGIONS.map((region) => {
    const srv = serverByRegion(region, "");
    return {
      value: region,
      label: serverLabel(srv, settings.lang),
      sublabel: region,
      icon: <Flag code={region} size={22} />,
    };
  });

  const currentRegion = SHUFFLE_REGIONS.includes(settings.psiphonRegion) ? settings.psiphonRegion : SHUFFLE_REGIONS[0];

  return (
    <Page title={t("settingsTitle")}>
      <div className={isMobile ? "flex flex-col gap-4" : "grid grid-cols-2 items-start gap-4"}>
        <div className="flex flex-col gap-4">
          <Card title={t("setAppearance")}>
            <div className="flex flex-wrap items-center justify-between gap-3 py-2">
              <span className="text-[15px] font-semibold">{t("setTheme")}</span>
              <Segmented value={settings.theme} options={themes} onChange={(theme) => patchSettings({ theme })} />
            </div>
            <div className="flex flex-wrap items-center justify-between gap-3 py-2">
              <span className="text-[15px] font-semibold">{t("setLanguage")}</span>
              <Segmented value={settings.lang} options={languages} onChange={(lang) => patchSettings({ lang })} />
            </div>
          </Card>

          <Card title={t("setTunnel")}>
            <Field label={t("setTransport")}>
              <CustomSelect
                value={settings.transport}
                options={transportOptions}
                onChange={(transport) => patchSettings({ transport })}
              />
            </Field>
            <Field label={t("setScanMode")}>
              <CustomSelect
                value={settings.scanMode}
                options={scanOptions}
                onChange={(scanMode) => patchSettings({ scanMode })}
              />
            </Field>
            <Field label={t("setPsiphonRegion")}>
              <CustomSelect
                value={currentRegion}
                options={regionOptions}
                onChange={(psiphonRegion) =>
                  patchSettings({
                    psiphonRegion,
                    serverId: serverByRegion(psiphonRegion, settings.serverId).id,
                  })
                }
              />
            </Field>
            <SettingRow
              title={t("protoPsiphon")}
              hint={t("protoPsiphonHint")}
              checked={settings.psiphon}
              onChange={(psiphon) => patchSettings({ psiphon })}
            />
            <SettingRow
              title={t("protoSanction")}
              hint={t("protoSanctionHint")}
              checked={settings.sanctionCheck}
              onChange={(sanctionCheck) => patchSettings({ sanctionCheck })}
            />
          </Card>

          <Card title={t("setPorts")}>
            <div className="grid grid-cols-2 gap-x-3">
              {PORT_FIELDS.map((field) => (
                <Field key={field.key} label={t(field.label)}>
                  <input
                    inputMode="numeric"
                    value={settings.ports[field.key] || ""}
                    onChange={(event) => setPort(field.key, event.target.value)}
                    className={`${inputClass} mono`}
                  />
                </Field>
              ))}
            </div>
            {!valid && <p className="m-0 mt-1 text-[13px] text-red-500">{t("portsInvalid")}</p>}
          </Card>
        </div>

        <div className="flex flex-col gap-4">
          <Card title={t("setSystem")}>
            <Field label={t("setDns")}>
              <input
                value={settings.dns}
                onChange={(event) => patchSettings({ dns: event.target.value })}
                placeholder="1.1.1.1"
                className={`${inputClass} mono`}
              />
            </Field>
            <Field label={t("setExtra")}>
              <input
                value={settings.extraArgs}
                onChange={(event) => patchSettings({ extraArgs: event.target.value })}
                placeholder="--example-flag value"
                className={`${inputClass} mono`}
              />
            </Field>
            <SettingRow
              title={t("setSysProxy")}
              hint={t("setSysProxyHint")}
              checked={settings.systemProxy}
              onChange={(systemProxy) => patchSettings({ systemProxy })}
            />
            <SettingRow
              title={t("setTray")}
              checked={settings.minimizeToTray}
              onChange={(minimizeToTray) => patchSettings({ minimizeToTray })}
            />
            <SettingRow
              title={t("setAutoUpdate")}
              checked={settings.autoUpdate}
              onChange={(autoUpdate) => patchSettings({ autoUpdate })}
            />
          </Card>

          <Card title={t("updaterTitle")}>
            <div className="grid grid-cols-2 gap-3">
              <div className="rounded-2xl border border-line bg-chip p-3">
                <div className="text-[12px] font-semibold text-fg2">{t("updaterApp")}</div>
                <div className="mono mt-1 text-[15px] font-bold">v{APP_VERSION}</div>
                {updateInfo && <div className="mono mt-0.5 text-[12px] font-bold text-[#00a870]">→ v{updateInfo.appLatest}</div>}
              </div>
              <div className="rounded-2xl border border-line bg-chip p-3">
                <div className="text-[12px] font-semibold text-fg2">{t("updaterCore")}</div>
                <div className="mono mt-1 text-[15px] font-bold">v{CORE_VERSION}</div>
                {updateInfo && <div className="mono mt-0.5 text-[12px] font-bold text-[#00a870]">→ v{updateInfo.coreLatest}</div>}
              </div>
            </div>
            <div className="mt-4 flex justify-end">
              <Button onClick={() => setModal("updater")}>
                <Download size={16} />
                {t("updaterOpen")}
              </Button>
            </div>
          </Card>

          <AboutCard />
        </div>
      </div>
    </Page>
  );
}